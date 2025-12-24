use std::rc::Rc;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad").to_vec();
    let wad_data = Rc::from(wad_data);
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_data)).unwrap();

    let palette_lump = wad.get_lump(Vec::new(), "PLAYPAL").unwrap();
    let palette_data = &wad_data[palette_lump.start()..palette_lump.end()];
    let palette = wad_rs::palette::Palette::try_from(palette_data).unwrap();

    let index = wad.get_lump_index();
    let mut count = 0usize;
    for (name, lump_ref) in index.iter() {
        if lump_ref.end() > wad_data.len() {
            println!("Lump {} has invalid end offset, skipping", name);
            continue;
        }

        if lump_ref.start() > wad_data.len() {
            println!("Lump {} has invalid start offset, skipping", name);
            continue;
        }

        if name.starts_with("S/") {
            count += 1;
            let sprite = wad_rs::sprite::Sprite::new(&wad_data, lump_ref.start(), lump_ref.end()).unwrap();
            println!(
                "Lump {name}:\n\tSize {} Bytes\n\tWidth: {}\n\tHeight: {}\n\tLeft Offset: {}\n\tTop Offset: {}",
                sprite.size(),
                sprite.width(),
                sprite.height(),
                sprite.left_offset(),
                sprite.top_offset(),
            );

            let file = std::fs::File::create(format!("assets/img/{}.png", name.replace("/", "_"))).unwrap();
            let data = sprite.rgba_image(&wad_data, &palette).unwrap();
            let mut encoder = png::Encoder::new(file, sprite.width() as u32, sprite.height() as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&data).unwrap();
        }
    }
    println!("Extracted {} sprites", count);
}