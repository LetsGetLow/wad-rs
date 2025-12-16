use std::sync::Arc;
use wad_rs::LumpCollection;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad").to_vec();
    let wad_data = Arc::from(wad_data);
    let wad = wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
    let lumps = wad.get_lumps();
    print_lump_names(lumps, 0);
}

fn print_lump_names(lumps: &LumpCollection, level: usize) {
    for (name, collection) in lumps.collection_iter() {
        println!("{}{}", "  ".repeat(level), name);
        print_lump_names(collection, level + 1);
    }
    for (name, _) in lumps.iter() {
        println!("{}{}", "  ".repeat(level), name);
    }
}