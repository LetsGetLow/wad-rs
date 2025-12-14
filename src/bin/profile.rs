use std::sync::Arc;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad").to_vec();
    let wad_data = Arc::from(wad_data);
    let wad = wad_rs::WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
    for e  in wad.get_directory().iter() {
        println!("{:?}", e.name(Arc::clone(&wad_data)));
    }
}