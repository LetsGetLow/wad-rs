use std::rc::Rc;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad").to_vec();
    let wad_data = Rc::from(wad_data);
    let wad = wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_data)).unwrap();
    let lumps = wad.get_lump_index();
    for (name, _) in lumps.iter() {
        println!("{name}");
    }
}