extern crate core;

use demos::AudioStream;
use std::rc::Rc;
use wad_rs::audio::{MidiSynthesizer, MusicSample};

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad").to_vec();
    let wad_data = Rc::from(wad_data);
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_data)).unwrap();

    let index = wad.get_lump_index();
    let audio_stream = AudioStream::new();
    let audio_stream = match audio_stream {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create audio stream: {}", e);
            return;
        }
    };


    let mut synthesizer =
        MidiSynthesizer::new(include_bytes!("../../assets/microgm.sf2"), 16_000).unwrap();

    for (name, lump_ref) in index.iter() {
        if lump_ref.end() > wad_data.len() {
            println!("Lump {} has invalid end offset, skipping", name);
            continue;
        }

        if lump_ref.start() > wad_data.len() {
            println!("Lump {} has invalid start offset, skipping", name);
            continue;
        }

        if name.starts_with("D_") {
            assert!(wad_data.len() >= 8);
            let data = wad_data[lump_ref.start()..lump_ref.end()].as_ref();
            let sample = MusicSample::from_bytes(&mut synthesizer, data, true).unwrap();
            audio_stream.append_music(sample.clone());
            println!(
                "Lump {name} : {} seconds (pcm size: {} bytes)",
                sample.sample().len() as f32 / sample.sample_rate() as f32,
                sample.sample().len() * size_of::<f32>()
            );
        }
    }
    println!("Playing all samples");
    audio_stream.play();
}
