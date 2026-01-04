type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Palette<'a> {
    colors: &'a [[u8; 3]; 256],
}

impl<'a> Palette<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        if data.len() < 768 {
            return Err("Palette data too short".into());
        }

        const PALETTE_BYTES: usize = 3 * 256;
        let raw = data
            .first_chunk::<{ PALETTE_BYTES }>()
            .ok_or("Palette data too short")?;
        let (chunks, remainder) = raw.as_chunks::<3>();
        debug_assert!(remainder.is_empty());
        let colors: &[[u8; 3]; 256] = chunks
            .try_into()
            .map_err(|_| "Palette data malformed")?;

        Ok(Self { colors })
    }

    pub fn get_rgb(&self, index: usize) -> Option<&[u8; 3]> {
        self.colors.get(index)
    }

    pub fn get_rgba(&self, index: usize) -> Option<[u8; 4]> {
        self.colors.get(index).map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
    }

}

impl<'a> TryFrom<&'a [u8]> for Palette<'a> {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        Palette::from_bytes(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_can_be_created_from_bytes() {
        let data: Vec<u8> = (0..768).map(|val: u16| (val % 256) as u8).collect();
        let palette = Palette::from_bytes(&data).unwrap();
        assert_eq!(palette.colors.len(), 256);
    }

    #[test]
    fn palette_creation_fails_with_short_data() {
        let data: Vec<u8> = (0..500).map(|val: u16| (val % 256) as u8).collect();
        let result = Palette::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn palette_can_get_rgb_by_index() {
        let data: Vec<u8> = (0..768).map(|val: u16| (val % 256) as u8).collect();
        let palette = Palette::try_from(data.as_slice()).unwrap();
        assert_eq!(palette.get_rgb(0), Some(&[0, 1, 2]));
        assert_eq!(palette.get_rgb(255), Some(&[253, 254, 255]));
    }

    #[test]
    fn palette_can_get_rgba_by_index() {
        let data: Vec<u8> = (0..768).map(|val: u16| (val % 256) as u8).collect();
        let palette = Palette::from_bytes(&data).unwrap();
        assert_eq!(palette.get_rgba(0), Some([0, 1, 2, 255]));
        assert_eq!(palette.get_rgba(255), Some([253, 254, 255, 255]));
    }
}