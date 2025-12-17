pub fn is_map_lump(name: &String) -> bool {
    matches!(
        name.as_str(),
        "THINGS"
            | "LINEDEFS"
            | "SIDEDEFS"
            | "VERTEXES"
            | "SECTORS"
            | "SSECTORS"
            | "SEGS"
            | "NODES"
            | "REJECT"
            | "BLOCKMAP"
            | "BEHAVIOR"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_map_lump_identifies_map_lumps() {
        let map_lumps = vec![
            "THINGS",
            "LINEDEFS",
            "SIDEDEFS",
            "VERTEXES",
            "SECTORS",
            "SSECTORS",
            "SEGS",
            "NODES",
            "REJECT",
            "BLOCKMAP",
            "BEHAVIOR",
        ];

        for lump in map_lumps {
            assert!(is_map_lump(&lump.to_string()));
        }

        let non_map_lumps = vec!["TEXTURE1", "FLAT1", "SOUND1", "GRAPHICS", "LEVEL1"];
        for lump in non_map_lumps {
            assert!(!is_map_lump(&lump.to_string()));
        }
    }
}
