extern crate core;

use demos::AudioStream;
use wad_rs::audio::MidiSynthesizer;

fn main() {

    let mut synthesizer = MidiSynthesizer::new(include_bytes!("../../assets/microgm.sf2"), 16_000).unwrap();
    let wad_data = include_bytes!("../../assets/wad/freedoom1.wad");
    let mut wad = wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), wad_data, &mut synthesizer).unwrap();

    let index = wad.get_lump_index();
    let audio_stream = AudioStream::new();
    let audio_stream = match audio_stream {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create audio stream: {}", e);
            return;
        }
    };
    let lump_names = index
        .iter()
        .filter(|&(name, _)| name.starts_with("D_"))
        .map(|(name, _)| name.to_string())
        .collect::<Vec<String>>();

    for name in lump_names {
        let sample = wad.get_music_sample(&name).unwrap();
        if let Some(sample) = sample {
            println!(
                "Lump {name} : {} seconds (pcm size: {} bytes)",
                sample.sample().len() as f32 / sample.sample_rate() as f32,
                sample.sample().len() * size_of::<f32>()
            );
            audio_stream.append_music(sample);
        };
    }
    println!("Playing all samples");
    audio_stream.play();
}
