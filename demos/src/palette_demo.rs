use wad_rs::graphics::ColorMap;
use wad_rs::index::LumpNode;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad");
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data).unwrap();

    let palette_node = wad.get_lump(Vec::new(), "PLAYPAL").unwrap();
    let palette_lump = match palette_node {
        LumpNode::Lump { lump, .. } => lump,
        _ => panic!("PLAYPAL not a lump"),
    };
    let palette_data = palette_lump.data();
    let palette = wad_rs::graphics::Palette::try_from(palette_data).unwrap();
    for i in 0..256 {
        let rgb = palette.get_rgb(i).unwrap();
        let rgba = palette.get_rgba(i).unwrap();
        println!("Color {}: R={}, G={}, B={}", i, rgb[0], rgb[1], rgb[2]);
        println!("Color {}: R={}, G={}, B={}, A={}", i, rgba[0], rgba[1], rgba[2], rgba[3]);
    }

    let colormap_node = wad.get_lump(Vec::new(), "COLORMAP").unwrap();
    let colormap_lump = match colormap_node {
        LumpNode::Lump { lump, .. } => lump,
        _ => panic!("COLORMAP not a lump"),
    };
    let colormap_data = colormap_lump.data();
    let colormap = ColorMap::from_bytes(colormap_data).unwrap();
    for i in 0..ColorMap::NUM_MAPS {
        let map = colormap.get_map_by(i).unwrap();
        println!("Colormap {}: {:?}", i, map)
    }
}