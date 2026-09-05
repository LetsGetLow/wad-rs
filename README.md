# wad-rs library

This library provides a higher-level abstraction for WAD files. It indexes the wad file with almost no heap allocation.
It converts old data formats to modern ones. i.e. 8-bit sound samples and midi data to f32 audio buffers or sprites to image buffer with alpha channel.

**Status:** under active development — the public API is not yet stable.

## Features
- No heap allocation when indexing the wad file
- Converts 8-bit sound samples to f32 audio buffers
- Converts midi data to f32 audio buffers
- Converts sprites to image buffers with alpha channel 
- Sprites can use different PaletteMapper implementations
- Supports Doom, Doom2, Heretic, Hexen wad formats
- Easy to use API
- High performance
- Lightweight

## Example

```rust
use wad_rs::WadIndex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wad_data = include_bytes!("assets/wad/freedoom1.wad");

    let mut synthesizer = MidiSynthesizer::new(include_bytes!("assets/microgm.sf2"), 16_000).unwrap();
    let wad = WadIndex::from_bytes("wadfile.wad".to_string(), wad_data, &mut synthesizer).unwrap();

    // List all lumps in the wad file
    
    for lump in wad.wad.get_lump_index().values() {
        println!("Lump name: {}", lump.name());
    }

    // Extract a specific lump (e.g., a sound sample)
    if let Some(sound_buffer) = wad.get_sound_sample("SOUND1") {
        println!("Extracted audio buffer with {} samples", sound_buffer.len());
    }
    
    // Extract a MIDI lump
    if let Some(music_buffer) = wad.get_music_sample("MUSIC1") {
        println!("Extracted MIDI audio buffer with {} samples", music_buffer.len());
        
    }
    
    // Extract a sprite lump
    if let Some(sprite) = wad.get_sprite("SPRITE1") {
        let sprite = sprite.unwrap();
        println!("Converted sprite to image buffer with dimensions: {}x{}", sprite.width(), sprite.height());
    }

    Ok(())
}
```