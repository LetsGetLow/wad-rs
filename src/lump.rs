pub const LUMP_NAME_LENGTH: usize = 8;
pub const LUMP_ENTRY_LENGTH: usize = 16;

/// A refence to a lump data and it's name
/// this struct does not own any data, it just stores references inside the WAD file
///
/// # Fields
/// - `name`: The name of the lump
/// - `data`: The raw data of the lump. empty data indicates a marker lump
#[derive(Debug, Clone, PartialEq)]
pub struct LumpRef<'a> {
    name: &'a str,
    data: &'a [u8],
}

impl<'a> LumpRef<'a> {
    /// Creates a new LumpRef
    pub fn new(data: &'a [u8], name: &'a str) -> Self {
        Self { data, name }
    }

    // Determines if the lump is a marker (has no data)
    pub fn is_marker(&self) -> bool {
        self.data.len() == 0
    }

    // Returns the lump name
    pub fn name(&self) -> &'a str {
        self.name
    }

    // Returns the lump data
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lump_ref_can_store_the_name() {
        let lump_data: &[u8] = &[1, 2, 3, 4, 5];
        let lump_name = "TESTLUMP";
        let lump_ref = LumpRef::new(lump_data, lump_name);
        assert_eq!(lump_ref.name(), lump_name);
    }

    #[test]
    fn lump_ref_can_store_the_data() {
        let lump_data: &[u8] = &[1, 2, 3, 4, 5];
        let lump_name = "TESTLUMP";
        let lump_ref = LumpRef::new(lump_data, lump_name);
        assert_eq!(lump_ref.data(), lump_data);
    }

    #[test]
    fn lump_ref_identifies_marker_lump() {
        let marker_lump_data: &[u8] = &[];
        let marker_lump_name = "MARKER";
        let marker_lump_ref = LumpRef::new(marker_lump_data, marker_lump_name);
        assert!(marker_lump_ref.is_marker());
    }
}
