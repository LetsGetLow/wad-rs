extern crate core;

pub mod header;
pub mod directory;
pub mod wad;
pub mod lump;
pub mod tokenizer;
pub mod index;
pub mod map;
pub mod audio;
pub mod palette;

pub use wad::WadIndex;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}