use wad_rs::index::LumpNode;
use wad_rs::WadIndex;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom2.wad");
    let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data).unwrap();

    for (idx, (name, map_node)) in wad.get_maps().unwrap().iter().enumerate() {
        let idx = idx + 1;
        println!("Found map {idx}: {name}");
        match map_node {
            LumpNode::Namespace { children, .. } => for (child_name, ..) in children {
                println!("\t- {child_name}");
            },
            _ => continue,
        };
    }
}
