use demos::AudioStream;

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

    for (name, lump_ref) in index.iter() {
        if name.starts_with("DS") {
            assert!(wad_data.len() >= 8);
            let data = lump_ref.data();
            let sample = wad_rs::audio::SoundSample::try_from(data).unwrap();
            audio_stream.append_sound(sample);
            println!("Lump {name} appended to audio stream");
        }
    }

    println!("Playing all sound samples");
    audio_stream.play();
}