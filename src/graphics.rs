use crate::error::WadError;

type Error = WadError;
type Result<T> = std::result::Result<T, Error>;

/// Palette represents a 256-color palette, where each color is represented by 3 bytes (RGB).
#[derive(Debug, Clone)]
pub struct Palette<'a> {
    colors: &'a [[u8; 3]; 256],
}

impl<'a> Palette<'a> {
    const SIZE: usize = 3 * 256;

    /// Creates a Palette from a byte slice.
    /// Expects the data to be at least 768 bytes (256 colors * 3 bytes per color).
    /// # Errors
    /// Returns an error if the data is too short or malformed.
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        let raw = data
            .first_chunk::<{ Palette::SIZE }>()
            .ok_or(WadError::PaletteDataTooShort)?;
        let (chunks, _) = raw.as_chunks::<3>();
        let colors: &[[u8; 3]; 256] = chunks.try_into().map_err(|_| WadError::PaletteDataMalformed)?;

        Ok(Self { colors })
    }

    /// Retrieves the RGB color at the specified index.
    pub fn get_rgb(&self, index: usize) -> Option<&[u8; 3]> {
        self.colors.get(index)
    }

    /// Retrieves the RGBA color at the specified index, with alpha set to 255.
    pub fn get_rgba(&self, index: usize) -> Option<[u8; 4]> {
        self.colors
            .get(index)
            .map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
    }
}

impl<'a> TryFrom<&'a [u8]> for Palette<'a> {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        Palette::from_bytes(value)
    }
}

/// A ColorMap contains multiple color maps, each mapping 256 palette indices.
pub struct ColorMap<'a> {
    map: &'a [[u8; 256]; 34],
}

impl<'a> ColorMap<'a> {
    pub const NUM_MAPS: usize = 34;
    pub const NUM_COLORS_PER_MAP: usize = 256;
    pub const SIZE: usize = ColorMap::NUM_COLORS_PER_MAP * ColorMap::NUM_MAPS;

    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        let raw = data
            .first_chunk::<{ ColorMap::SIZE }>()
            .ok_or(WadError::ColorMapDataTooShort)?;
        let (chunks, _) = raw.as_chunks::<256>();
        let map: &[[u8; 256]; 34] = chunks.try_into().map_err(|_| WadError::ColorMapDataMalformed)?;

        Ok(Self { map })
    }

    pub fn get_map_by(&self, index: usize) -> Option<&[u8; 256]> {
        self.map.get(index)
    }
}

impl<'a> TryFrom<&'a [u8]> for ColorMap<'a> {
    type Error = Error;

    /// Creates a ColorMap from a byte slice.
    /// Expects the data to be at least 8704 bytes (34 maps * 256 bytes per map).
    /// # Errors
    /// Returns an error if the data is too short or malformed.
    fn try_from(value: &'a [u8]) -> Result<Self> {
        ColorMap::from_bytes(value)
    }
}

/// Map Palette indices by ColorMap  then to RGB colors
pub trait PaletteMapper<'a> {
    fn remap_color(&self, index: u8) -> Option<&'a [u8; 3]>;
}

/// A mapper that uses a palette and a colormap to remap colors
/// from one palette index to another.
/// Given an input index, it looks up the corresponding index in the colormap,
/// then retrieves the RGB color from the palette using that index.
pub struct DefaultPaletteMapper<'a> {
    palette: &'a Palette<'a>,
    colormap: &'a [u8; 256],
}

impl<'a> DefaultPaletteMapper<'a> {
    /// Creates a new DefaultPaletteMapper with the given palette, colormap, and map index.
    /// Returns an error if the map index is out of bounds.
    ///
    /// # Arguments
    /// * `palette` - A reference to the Palette to use for color lookup.
    /// * `colormap` - A reference to the ColorMap to use for index mapping.
    /// * `map_index` - The index of the colormap to use.
    ///
    /// # Errors
    /// Returns an error if the map index is out of bounds.
    pub fn new(
        palette: &'a Palette<'a>,
        colormap: &'a ColorMap<'_>,
        map_index: usize,
    ) -> Result<Self> {
        let colormap = colormap
            .get_map_by(map_index)
            .ok_or(WadError::ColorIndexOutOfBounds {index: map_index})?;
        Ok(Self { palette, colormap })
    }
}

impl<'a> PaletteMapper<'a> for DefaultPaletteMapper<'a> {
    /// Remaps a color index using the colormap and retrieves the RGB color from the palette.
    /// Returns None if the index is out of bounds.
    ///
    /// # Arguments
    /// * `index` - The input color index to remap.
    ///
    /// # Returns
    /// the RGB color corresponding to the remapped index,
    /// or None if the index is out of bounds.
    fn remap_color(&self, index: u8) -> Option<&'a [u8; 3]> {
        let mapped_index = *self.colormap.get(index as usize)?;
        self.palette.get_rgb(mapped_index as usize)
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

    #[test]
    fn colormap_can_be_created_from_bytes() {
        let data: Vec<u8> = (0..(34 * 256)).map(|val: u16| (val % 256) as u8).collect();
        let colormap = ColorMap::from_bytes(&data).unwrap();
        assert_eq!(colormap.map.len(), 34);
    }

    #[test]
    fn colormap_creation_fails_with_short_data() {
        let data: Vec<u8> = (0..500).map(|val: u16| (val % 256) as u8).collect();
        let result = ColorMap::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn colormap_can_get_map_by_index() {
        let data: Vec<u8> = (0..(34 * 256)).map(|val: u16| (val % 256) as u8).collect();
        let colormap = ColorMap::from_bytes(&data).unwrap();
        let expected_map: [u8; 256] = (0..=255)
            .map(|val: u8| val)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        assert_eq!(colormap.get_map_by(0), Some(&expected_map));
        assert_eq!(colormap.get_map_by(33), Some(&expected_map));
    }

    #[test]
    fn palette_colormap_mapper_can_remap_color() {
        let palette_data: Vec<u8> = (0..768).map(|val: u16| (val % 256) as u8).collect();
        let palette = Palette::from_bytes(&palette_data).unwrap();
        let colormap_data: Vec<u8> = (0..(34 * 256))
            .rev()
            .map(|val: u16| (val % 256) as u8)
            .collect();
        let colormap = ColorMap::from_bytes(&colormap_data).unwrap();
        let mapper = DefaultPaletteMapper::new(&palette, &colormap, 0).unwrap();
        assert_eq!(mapper.remap_color(0), Some(&[253, 254, 255]));
        assert_eq!(mapper.remap_color(255), Some(&[0, 1, 2]));
    }
}
