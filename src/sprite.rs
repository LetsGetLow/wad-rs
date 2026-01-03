use crate::graphics::Palette;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const HEADER_SIZE: usize = 8;

/// Header of a Doom-Patch-/Sprite-Lump
#[derive(Debug, Clone, Copy)]
pub struct SpriteHeader {
    pub width: u16,
    pub height: u16,
    pub left_offset: i16,
    pub top_offset: i16,
}

impl SpriteHeader {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err("Sprite lump too small for header".into());
        }

        let width = u16::from_le_bytes([data[0], data[1]]);
        let height = u16::from_le_bytes([data[2], data[3]]);
        let left_offset = i16::from_le_bytes([data[4], data[5]]);
        let top_offset = i16::from_le_bytes([data[6], data[7]]);

        Ok(SpriteHeader {
            width,
            height,
            left_offset,
            top_offset,
        })
    }
}

/// Represents a Doom sprite/patch lump.
///
/// Contains metadata and methods to extract the sprite image data and turn it into
/// an RGBA pixel buffer.
///
/// # Structure of a Sprite Lump
/// A sprite lump starts with an 8-byte header:
/// - Bytes 0-1: Width (u16, little-endian)
/// - Bytes 2-3: Height (u16, little-endian)
/// - Bytes 4-5: Left offset (i16, little-endian)
/// - Bytes 6-7: Top offset (i16, little-endian)
/// Following the header is a column offset table, which contains
/// 4-byte little-endian offsets for each column of the sprite image data.
/// Each column consists of a series of "posts", where each post has:
/// - 1 byte: Top delta (y offset where the post starts)
/// - 1 byte: Length of the post (number of pixels)
/// - 1 byte: Dummy byte (unused)
/// - N bytes: Pixel data (palette indices)
/// - 1 byte: 0xFF (end of column marker)
///
#[derive(Debug, Clone)]
pub struct Sprite<'a> {
    lump_data: &'a [u8],
    header: SpriteHeader,
}

impl<'a> Sprite<'a> {
    /// Creates a `Sprite` from the complete lump slice.
    ///
    /// # Arguments
    /// - `data`: The complete WAD data slice.
    /// - `start`: The start offset of the sprite lump within `data`.
    /// - `end`: The end offset of the sprite lump within `data`.
    /// # Returns
    /// - `Ok(Sprite)` if the sprite lump is valid.
    /// - `Err` if the sprite lump is invalid or out of bounds.
    pub fn new(lump_data: &'a [u8]) -> Result<Self> {
        let header = SpriteHeader::from_bytes(lump_data)?;
        Self::check_size(header.width as usize, lump_data)?;

        Ok(Self {
            lump_data,
            header,
        })
    }

    pub fn header(&self) -> SpriteHeader {
        self.header
    }

    pub fn width(&self) -> u16 {
        self.header.width
    }

    pub fn height(&self) -> u16 {
        self.header.height
    }

    pub fn left_offset(&self) -> i16 {
        self.header.left_offset
    }

    pub fn top_offset(&self) -> i16 {
        self.header.top_offset
    }

    // Size of the sprite image data in bytes
    pub fn size(&self) -> usize {
        self.lump_data.len()
    }

    pub fn rgba_pixel_buffer(&self, palette: &Palette) -> Result<Vec<u8>> {
        let w = self.width() as usize;
        let h = self.height() as usize;

        // sanity check
        if w == 0 || h == 0 {
            return Err("sprite has zero width or height".into());
        }

        let lump = self.lump_data;

        Self::check_size(w, lump)?;

        // init RGBA pixel buffer with transparent pixels
        let mut pixel_buffer = vec![0u8; w * h * 4];

        for row in 0..w {
            let offset_index = HEADER_SIZE + row * 4;
            let column_offset = u32::from_le_bytes([
                lump[offset_index],
                lump[offset_index + 1],
                lump[offset_index + 2],
                lump[offset_index + 3],
            ]) as usize;

            if column_offset >= lump.len() {
                return Err("column offset out of range".into());
            }

            let mut cursor = column_offset;
            loop {
                let topdelta = lump
                    .get(cursor)
                    .copied()
                    .ok_or("unexpected end of post header")?;
                cursor += 1;

                // 0xFF marks the end of the column
                if topdelta == 0xFF {
                    break;
                }

                let length = lump
                    .get(cursor)
                    .copied()
                    .ok_or("unexpected end of post length")? as usize;
                cursor += 2;

                let data_start = cursor;
                let data_end = data_start
                    .checked_add(length)
                    .ok_or("post length overflow")?;

                if data_end >= lump.len() {
                    return Err("post data out of range".into());
                }

                if data_end + 1 > lump.len() {
                    return Err("post trailing byte missing".into());
                }

                let row_start = topdelta as usize;
                if row_start >= h || row_start + length > h {
                    return Err("post writes beyond sprite height".into());
                }

                let pixel_data = &lump[data_start..data_end];
                for (dy, &index) in pixel_data.iter().enumerate() {
                    let y = row_start + dy;
                    let buffer_pos = (y * w + row) * 4;
                    pixel_buffer[buffer_pos..buffer_pos + 4].copy_from_slice(
                        palette
                            .get_rgba(index as usize)
                            .ok_or("palette index out of bounds")?
                            .as_ref(),
                    );
                }
                cursor = data_end + 1;
            }
        }

        Ok(pixel_buffer)
    }

    fn check_size(w: usize, lump: &[u8]) -> Result<()> {
        let column_table_bytes = w.checked_mul(4).ok_or("column table size overflow")?;
        if lump.len() < HEADER_SIZE + column_table_bytes {
            Err("sprite lump too small for column table".into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_header_from_bytes_rejects_too_small_data() {
        let data = [0u8; 4];
        let result = SpriteHeader::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn sprite_header_can_extract_header_data() {
        let data = [0x10, 0x00, 0x20, 0x00, 0xFF, 0xFF, 0xEE, 0xFF];
        let header = SpriteHeader::from_bytes(&data).unwrap();
        assert_eq!(header.width, 16);
        assert_eq!(header.height, 32);
        assert_eq!(header.left_offset, -1);
        assert_eq!(header.top_offset, -18);
    }
}
