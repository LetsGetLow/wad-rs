use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamBuilder, Sink, StreamError};
use std::rc::Rc;
use wad_rs::audio::SoundSample;

fn main() {
    let wad_data = include_bytes!("../../assets/wad/freedoom2.wad").to_vec();
    let wad_data = Rc::from(wad_data);
    let wad =
        wad_rs::WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_data)).unwrap();

    let index = wad.get_lump_index();
    let audio_stream = AudioStream::new();
    let audio_stream = match audio_stream {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create audio stream: {}", e);
            return ;
        }
    };

    for (name, lump_ref) in index.iter() {
        if lump_ref.end() > wad_data.len() {
            println!("Lump {} has invalid end offset, skipping", name);
            continue;
        }

        if lump_ref.start() > wad_data.len() {
            println!("Lump {} has invalid start offset, skipping", name);
            continue;
        }

        if name.starts_with("DS") {
            assert!(wad_data.len() >= 8);
            let data = wad_data[lump_ref.start()..lump_ref.end()].as_ref();
            let sample = wad_rs::audio::SoundSample::try_from(data).unwrap();
            audio_stream.append_sound(sample);
        }
    }

    println!("Playing all samples");
    audio_stream.play();
}

pub struct AudioStream {
    stream: OutputStream,
    sink: Sink,
}

impl AudioStream {
    pub fn new() -> Result<Self, StreamError> {
        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream.mixer());

        Ok(AudioStream { stream, sink })
    }

    pub fn append_sound(&self, audio: SoundSample) {
        let source = SamplesBuffer::new(1, audio.sample_rate(), audio.sample());
        self.sink.append(source);
    }

    pub fn play(&self) {
        self.sink.play();
        self.sink.sleep_until_end();
    }
}