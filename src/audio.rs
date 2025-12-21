use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const DEFAULT_MIDI_SAMPLE_RATE: i32 = 16000;

/// A structure representing a sound sample with its sample rate and audio data.
/// The audio data is stored as a vector of f32 samples normalized between -1.0 and 1.0.
///
/// # Format Description
/// The sound sample can be created from a byte slice that follows a specific format.
/// 8 bytes header followed by audio sample data as 8-bit unsigned integers.
///
/// # Header Format
/// - The first 2 bytes represent the magic number (u16, little-endian, always 768).
/// - The next 2 bytes represent the sample rate (u16, little-endian).
/// - The next 4 bytes represent the number of samples (u32, little-endian).
#[derive(Debug, Clone)]
pub struct SoundSample {
    sample_rate: u32,
    samples: Vec<f32>,
}

impl SoundSample {
    pub fn new(sample_rate: u32, samples: Vec<f32>) -> Self {
        SoundSample {
            sample_rate,
            samples,
        }
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample(&self) -> &[f32] {
        &self.samples
    }

    pub fn is_sound_sample(data: &[u8]) -> bool {
        data.starts_with(&[0x03, 0x00])
    }
}

impl TryFrom<&[u8]> for SoundSample {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err("Data too short to contain valid sound sample header".into());
        }

        if data[0] != 0x03 || data[1] != 0x00 {
            return Err("Invalid sound sample magic number".into());
        }

        let sample_rate = u16::from_le_bytes([data[2], data[3]]) as u32;
        let sample_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let sample_end = 8 + sample_count;
        if sample_end > data.len() {
            return Err("Data too short to contain declared number of samples".into());
        }

        let sample = data[8..sample_end]
            .iter()
            .map(|&b| (b as f32 - 128.0) / 128.0)
            .collect();

        Ok(SoundSample {
            sample_rate,
            samples: sample,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicType {
    Mus,
    Midi,
    Unknown,
}

/// A structure representing a music file.
#[derive(Debug, Clone)]
pub struct MusicSample {
    sample_rate: u32,
    sample_channels: u16,
    sample: Vec<f32>,
}

impl MusicSample {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.sample_channels
    }

    pub fn sample(&self) -> &[f32] {
        &self.sample
    }

    pub fn determine_type(data: &[u8]) -> MusicType {
        match data.get(..4) {
            Some(b"MUS\x1A") => MusicType::Mus,
            Some(b"MThd") => MusicType::Midi,
            _ => MusicType::Unknown,
        }
    }
}

impl TryFrom<&[u8]> for MusicSample {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        let format = MusicSample::determine_type(data);
        match format {
            MusicType::Mus => {
                // TODO: need to get hands on WAD with MUS files to implement parser
                todo!()
            }
            MusicType::Midi => {
                let sample = midi_to_pcm(data, DEFAULT_MIDI_SAMPLE_RATE);
                Ok(MusicSample {
                    sample_rate: DEFAULT_MIDI_SAMPLE_RATE as u32,
                    sample_channels: 1,
                    sample,
                })
            }
            MusicType::Unknown => Err("Unknown music format".into()),
        }
    }
}

fn midi_to_pcm(mid: &[u8], sample_rate: i32) -> Vec<f32> {
    let mut sf2 = File::open("../assets/microgm.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    // Load the MIDI file.
    let mut mid = Cursor::new(mid);
    let midi_file = Arc::new(MidiFile::new(&mut mid).unwrap());

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(sample_rate);
    let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file.
    sequencer.play(&midi_file, false);

    // The output buffer.
    let sample_count = (settings.sample_rate as f64 * midi_file.get_length()) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    sequencer.render(&mut left[..], &mut right[..]);

    // Write the waveform to the file.
    let mut sample = Vec::with_capacity(left.len() + right.len());
    for t in 0..left.len() {
        // Mix down to mono
        sample.push((left[t] + right[t]) * 0.5);
    }

    sample
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_sample_conversion_fails_on_to_short_data() {
        let data = vec![0u8; 4];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_fails_on_invalid_magic_number() {
        let data = vec![0u8; 10];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_fails_on_invalid_sample_count() {
        let data = vec![
            0x03, 0x00, // Magic number
            0x40, 0x1F, // Sample rate (8000)
            0xFF, 0xFF, 0xFF, 0xFF, // Sample count (4294967295)
        ];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn sound_sample_conversion_succeeds_on_valid_data() {
        let data = vec![
            0x03, 0x00, // Magic number
            0x40, 0x1F, // Sample rate (8000)
            0x04, 0x00, 0x00, 0x00, // Sample count (4)
            0x00, 0x80, 0xFF, 0x7F, // Sample data
        ];
        let result = SoundSample::try_from(data.as_slice());
        assert!(result.is_ok());
        let sound_sample = result.unwrap();
        assert_eq!(sound_sample.sample_rate(), 8000);
        assert_eq!(sound_sample.sample(), &[-1.0, 0.0, 0.9921875, -0.0078125]);
    }

    #[test]
    fn sound_sample_detects_valid_magic_number() {
        let valid_magic = [0x03, 0x00];
        let invalid_magic = [0x04, 0x00];
        assert!(SoundSample::is_sound_sample(&valid_magic));
        assert!(!SoundSample::is_sound_sample(&invalid_magic));
    }

    #[test]
    fn music_sample_detects_types() {
        let mus_data = b"MUS\x1Arest of the data";
        let midi_data = b"MThdrest of the data";
        let unknown_data = b"XXXXrest of the data";
        let too_short_data = b"MU";

        assert_eq!(MusicSample::determine_type(mus_data), MusicType::Mus);
        assert_eq!(MusicSample::determine_type(midi_data), MusicType::Midi);
        assert_eq!(
            MusicSample::determine_type(unknown_data),
            MusicType::Unknown
        );
        assert_eq!(
            MusicSample::determine_type(too_short_data),
            MusicType::Unknown
        );
    }
}
