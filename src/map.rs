use crate::directory::DirectoryRef;

#[derive(Debug, Clone, PartialEq)]
pub struct MapIndex {
    things: Option<DirectoryRef>,
    line_defs: Option<DirectoryRef>,
    side_defs: Option<DirectoryRef>,
    vertices: Option<DirectoryRef>,
    sectors: Option<DirectoryRef>,
    sub_sectors: Option<DirectoryRef>,
    segs: Option<DirectoryRef>,
    nodes: Option<DirectoryRef>,
    reject: Option<DirectoryRef>,
    block_map: Option<DirectoryRef>,
}

impl MapIndex {
    pub fn new() -> Self {
        Self {
            things: None,
            line_defs: None,
            side_defs: None,
            vertices: None,
            sectors: None,
            sub_sectors: None,
            segs: None,
            nodes: None,
            reject: None,
            block_map: None,
        }
    }

    pub fn is_map_lump(name: &str) -> bool {
        matches!(
            name,
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
        )
    }

    pub fn add_lump(&mut self, name: &str, dir_ref: DirectoryRef) {
        match name {
            "THINGS" => self.things = Some(dir_ref),
            "LINEDEFS" => self.line_defs = Some(dir_ref),
            "SIDEDEFS" => self.side_defs = Some(dir_ref),
            "VERTEXES" => self.vertices = Some(dir_ref),
            "SECTORS" => self.sectors = Some(dir_ref),
            "SSECTORS" => self.sub_sectors = Some(dir_ref),
            "SEGS" => self.segs = Some(dir_ref),
            "NODES" => self.nodes = Some(dir_ref),
            "REJECT" => self.reject = Some(dir_ref),
            "BLOCKMAP" => self.block_map = Some(dir_ref),
            _ => {}
        }
    }
}