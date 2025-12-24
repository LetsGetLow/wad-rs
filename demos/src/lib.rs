use rodio::{OutputStream, OutputStreamBuilder, Sink, StreamError};
use rodio::buffer::SamplesBuffer;
use wad_rs::audio::{MusicSample, SoundSample};

pub struct AudioStream {
    _stream: OutputStream, // Keep the stream alive
    sink: Sink,
}

impl AudioStream {
    pub fn new() -> Result<Self, StreamError> {
        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream.mixer());

        Ok(AudioStream {
            _stream: stream,
            sink,
        })
    }

    pub fn append_sound(&self, audio: SoundSample) {
        let source = SamplesBuffer::new(1, audio.sample_rate(), audio.sample());
        self.sink.append(source);
    }

    pub fn append_music(&self, audio: MusicSample) {
        let source = SamplesBuffer::new(audio.channels(), audio.sample_rate(), audio.sample());
        self.sink.append(source);
    }

    pub fn play(&self) {
        self.sink.play();
        self.sink.sleep_until_end();
    }
}
