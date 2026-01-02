use wad_rs::index::LumpNode;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad");
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data).unwrap();

    let palette_node = wad.get_lump(Vec::new(), "PLAYPAL").unwrap();
    let palette_lump = match palette_node {
        LumpNode::Namespace { .. } => {
            panic!("PLAYPAL lump is a namespace, expected a lump");
        }
        LumpNode::Lump { lump, .. } => lump,
    };
    let palette_data = palette_lump.data();
    let palette = wad_rs::graphics::Palette::try_from(palette_data).unwrap();
    for i in 0..256 {
        let rgb = palette.get_rgb(i).unwrap();
        let rgba = palette.get_rgba(i).unwrap();
        println!("Color {}: R={}, G={}, B={}", i, rgb[0], rgb[1], rgb[2]);
        println!("Color {}: R={}, G={}, B={}, A={}", i, rgba[0], rgba[1], rgba[2], rgba[3]);
    }
}