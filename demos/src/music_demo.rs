extern crate core;

use demos::AudioStream;
use wad_rs::audio::{MidiSynthesizer, MusicSample};
use wad_rs::index::LumpNode;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad");
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data).unwrap();

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
        MidiSynthesizer::new(include_bytes!("../../assets/microgm.sf2"), 44_100).unwrap();

    for (name, lump_node) in index.iter() {
        if name.starts_with("D_") {
            let lump_ref = match lump_node {
                LumpNode::Lump {lump, .. } => lump,
                _ => continue,
            };
            assert!(wad_data.len() >= 8);
            let data = lump_ref.data();
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
