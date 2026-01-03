use wad_rs::index::LumpNode;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad");
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data).unwrap();

    let palette_lump = wad.get_lump(Vec::new(), "PLAYPAL").unwrap();
    let lump_ref = match palette_lump {
        LumpNode::Lump { lump, .. } => lump,
        _ => panic!("PLAYPAL is not a lump"),
    };
    let palette_data = lump_ref.data();
    let palette = wad_rs::graphics::Palette::try_from(palette_data).unwrap();

    let index = {
        let idx = wad.get_lump(Vec::new(), "S_START").unwrap();
        match idx {
            LumpNode::Namespace { children, .. } => children,
            _ => panic!("S_START is not a namespace"),
        }
    };

    let mut count = 0usize;

    for (name, lump_node) in index.iter() {
        count += 1;
        let lump_ref = match lump_node {
            LumpNode::Lump {lump, .. } => lump,
            _ => continue,
        };

        let sprite = wad_rs::sprite::Sprite::new(lump_ref.data()).unwrap();
        // println!(
        //     "Lump {name}:\n\tSize {} Bytes\n\tWidth: {}\n\tHeight: {}\n\tLeft Offset: {}\n\tTop Offset: {}",
        //     sprite.size(),
        //     sprite.width(),
        //     sprite.height(),
        //     sprite.left_offset(),
        //     sprite.top_offset(),
        // );

        let file = std::fs::File::create(format!("assets/img/{}.png", name.replace("/", "_"))).unwrap();
        let data = sprite.rgba_pixel_buffer(&palette).unwrap();
        let mut encoder = png::Encoder::new(file, sprite.width() as u32, sprite.height() as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&data).unwrap();
    }
    println!("Extracted {} sprites", count);
}