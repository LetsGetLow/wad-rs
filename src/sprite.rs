use crate::palette::Palette;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const HEADER_SIZE: usize = 8;


/// Header eines Doom-Patch-/Sprite-Lumps
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

/// Repräsentiert einen Doom-Sprite/Patch-Lump.
///
/// Hält einen Verweis auf die zugrunde liegenden WAD-Daten über `Rc<[u8]>`
/// und speichert lediglich Offsets/Metadaten, aber keine kopierten Pixel.
#[derive(Debug, Clone)]
pub struct Sprite {
    lump_start: usize,
    lump_end: usize,
    header: SpriteHeader,
}

impl Sprite {
    /// Erzeugt einen `Sprite` aus dem kompletten Lumpslice in einem `Rc<[u8]>`.
    ///
    /// Erwartet, dass `rc_data` genau den Sprite-Lump enthält (kein globaler WAD-Puffer).
    pub fn new(data: &[u8],  start: usize, end: usize) -> Result<Self> {
        if start >= end || end > data.len() {
            return Err("sprite lump range out of bounds".into());
        }

        let lump = &data[start..end];
        let header = SpriteHeader::from_bytes(lump)?;
        let column_table_bytes = (header.width as usize)
            .checked_mul(4)
            .ok_or("column table size overflow")?;

        if lump.len() < HEADER_SIZE + column_table_bytes {
            return Err("sprite lump too small for column table".into());
        }

        Ok(Sprite {
            lump_start: start,
            lump_end: end,
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
        self.lump_end - self.lump_start
    }

    pub fn image_data<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.lump_start..self.lump_end]
    }

    pub fn rgba_image(&self, data: &[u8], palette: &Palette) -> Result<Vec<u8>> {
        let w = self.width() as usize;
        let h = self.height() as usize;

        if w == 0 || h == 0 {
            return Err("sprite has zero width or height".into());
        }

        let column_table_bytes = w
            .checked_mul(4)
            .ok_or("column table size overflow")?;

        let lump = self.image_data(data);
        if lump.len() < HEADER_SIZE + column_table_bytes {
            return Err("sprite lump too small for column table".into());
        }

        let mut rgba = vec![0u8; w * h * 4];

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
                if cursor >= lump.len() {
                    return Err("unexpected end of column data".into());
                }

                let topdelta = lump[cursor];
                cursor += 1;

                if topdelta == 0xFF {
                    break;
                }

                if cursor + 1 >= lump.len() {
                    return Err("unexpected end of post header".into());
                }

                let length = lump[cursor] as usize;
                let _dummy = lump[cursor + 1];
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

                for (dy, &index) in lump[data_start..data_end].iter().enumerate() {
                    let y = row_start + dy;
                    let dest = (y * w + row) * 4;
                    if let Some(color) = palette.get_rgba(index as usize) {
                        rgba[dest..dest + 4].copy_from_slice(&color);
                    } else {
                        return Err("palette index out of bounds".into());
                    }
                }

                cursor = data_end + 1;
            }
        }

        Ok(rgba)
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
