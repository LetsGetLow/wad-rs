use once_cell::sync::Lazy;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::sync::Arc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub type SampleRate = u32;

pub type ChannelCount = u16;

pub type PcmSamples = Vec<f32>;

const SOUNDFONT: Lazy<Arc<SoundFont>> = Lazy::new(|| {
    let mut cursor = Cursor::new(include_bytes!("../assets/microgm.sf2"));
    Arc::new(SoundFont::new(&mut cursor).expect("Failed to load SoundFont"))
});
/// A structure representing a sound sample with its sample rate and audio data.
/// The audio data is stored as a vector of f32 samples normalized between -1.0 and 1.0.
/// SoundSamples are typically mono audio samples.
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
    sample_rate: SampleRate,
    samples: PcmSamples,
}

impl SoundSample {
    /// Returns the sample rate of the sound sample.
    ///
    /// # Returns
    /// - `SampleRate`: The sample rate in Hz.
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns a reference to the PCM sample data.
    ///
    /// # Returns
    /// - `&[f32]`: A slice of PCM samples normalized between -1.0 and 1.0.
    pub fn sample(&self) -> &PcmSamples {
        &self.samples
    }

    /// Checks if the provided data slice starts with the expected magic number for a sound sample.
    /// # Arguments
    /// - `data`: A byte slice to check.
    /// # Returns
    /// - `bool`: `true` if the data starts with the sound sample magic number, `false` otherwise.
    pub fn is_sound_sample(data: &[u8]) -> bool {
        data.starts_with(&[0x03, 0x00])
    }

    /// Creates a SoundSample from a byte slice following the specified format.
    /// # Arguments
    /// - `data`: A byte slice containing the sound sample data.
    /// # Returns
    /// - `Result<SoundSample>`: Ok(SoundSample) if successful, Err otherwise.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err("Data too short to contain valid sound sample header".into());
        }

        if !Self::is_sound_sample(data) {
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

        Ok(Self {
            sample_rate,
            samples: sample,
        })
    }
}

/// Implement TryFrom<&[u8]> for SoundSample to allow easy conversion from byte slices.
impl TryFrom<&[u8]> for SoundSample {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        Self::from_bytes(data)
    }
}

/// Enum representing the type of music file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicType {
    Mus,
    Midi,
    Unknown,
}

/// A structure representing a music file.
#[derive(Debug, Clone)]
pub struct MusicSample {
    sample_rate: SampleRate,
    sample_channels: ChannelCount,
    sample: PcmSamples,
}

impl MusicSample {
    /// Returns the sample rate of the music sample.
    ///
    /// # Returns
    /// - `SampleRate`: The sample rate in Hz.
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns the number of channels in the music sample.
    ///
    /// #  Returns
    /// - `ChannelCount`: The number of channels (1 for mono, 2 for stereo).
    pub fn channels(&self) -> ChannelCount {
        self.sample_channels
    }

    /// Returns a reference to the PCM sample data.
    ///
    /// # Returns
    /// - `&[f32]`: A slice of PCM samples normalized between -1.0 and 1.0.
    pub fn sample(&self) -> &PcmSamples {
        &self.sample
    }

    /// Determines the type of music file based on its header bytes.
    ///
    /// # Arguments
    /// - `data`: A byte slice containing the music file data.
    /// # Returns
    /// - `MusicType`: The determined music file type.
    fn determine_type(data: &[u8]) -> MusicType {
        match data.get(..4) {
            Some(b"MUS\x1A") => MusicType::Mus,
            Some(b"MThd") => MusicType::Midi,
            _ => MusicType::Unknown,
        }
    }

    /// Creates a MusicSample from a byte slice, sample rate, and channel configuration.
    /// # Arguments
    /// - `data`: A byte slice the music file data.
    /// - `sample_rate`: The desired sample rate for the output PCM samples.
    /// - `is_stereo`: A boolean indicating whether to output stereo samples.
    /// # Returns
    /// - `Result<MusicSample>`: Ok(MusicSample) if successful, Err otherwise.
    pub fn from_bytes(
        synthesizer: &mut MidiSynthesizer,
        midi_data: &[u8],
        is_stereo: bool,
    ) -> Result<Self> {
        let format = Self::determine_type(midi_data);
        match format {
            MusicType::Mus => {
                // TODO: need to get hands on WAD with MUS files to implement parser
                Err("MUS format not supported yet".into())
            }
            MusicType::Midi => Ok(Self {
                sample_rate: synthesizer.get_sample_rate(),
                sample_channels: if is_stereo { 2 } else { 1 },
                sample: synthesizer.synth(midi_data, is_stereo),
            }),
            MusicType::Unknown => Err("Unknown music format".into()),
        }
    }
}

/// Implement TryFrom<&[u8]> for MusicSample to allow easy conversion from byte slices.
/// with default sample rate of 16000 Hz and mono output.
impl TryFrom<&[u8]> for MusicSample {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        let mut synthesizer = MidiSynthesizer::new(
            include_bytes!("../assets/microgm.sf2"),
            MidiSynthesizer::DEFAULT_SAMPLE_RATE,
        )?;
        Self::from_bytes(&mut synthesizer, data, false)
    }
}

pub struct MidiSynthesizer {
    sample_rate: SampleRate,
    sequencer: MidiFileSequencer,
}

impl MidiSynthesizer {
    const DEFAULT_SAMPLE_RATE: SampleRate = 16_000;
    const MIN_SAMPLE_RATE: SampleRate = 16_000;
    const MAX_SAMPLE_RATE: SampleRate = 44_100;

    pub fn new(sound_font: &[u8], sample_rate: SampleRate) -> Result<Self> {
        if sample_rate < Self::MIN_SAMPLE_RATE || sample_rate > Self::MAX_SAMPLE_RATE {
            return Err("Sample rate out of bounds".into());
        }
        let sound_font = {
            let mut cursor = Cursor::new(sound_font);
            Arc::new(SoundFont::new(&mut cursor)?)
        };

        let sequencer = {
            // Create the MIDI file sequencer.
            let settings = SynthesizerSettings::new(sample_rate as i32);
            let synthesizer = Synthesizer::new(&sound_font, &settings)?;
            MidiFileSequencer::new(synthesizer)
        };

        Ok(Self {
            sample_rate,
            sequencer,
        })
    }

    pub fn get_sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// synth MIDI data to PCM samples
    pub fn synth(&mut self, midi_data: &[u8], is_stereo: bool) -> PcmSamples {
        let midi_data = &mut Cursor::new(midi_data);
        let midi_file = Arc::new(MidiFile::new(midi_data).unwrap());

        // initialize the output buffer.
        let sample_count = (self.sample_rate as f64 * midi_file.get_length()) as usize;
        let mut left: PcmSamples = vec![0_f32; sample_count];
        let mut right: PcmSamples = vec![0_f32; sample_count];

        // Play the MIDI file.
        self.sequencer.play(&midi_file, false);
        // Render the waveform.
        self.sequencer.render(&mut left[..], &mut right[..]);

        // Write the waveform to final buffer.
        if is_stereo {
            let mut sample = Vec::with_capacity(sample_count * 2);
            for t in 0..left.len() {
                sample.push(left[t]);
                sample.push(right[t]);
            }
            sample
        } else {
            let mut sample = Vec::with_capacity(sample_count);
            for t in 0..left.len() {
                // Mix down to mono
                sample.push((left[t] + right[t]) * 0.5);
            }
            sample
        }
    }
}

/// Convert MIDI data to PCM samples using an embedded SoundFont.
fn midi_to_pcm(midi_data: &[u8], sample_rate: SampleRate, is_stereo: bool) -> PcmSamples {
    // Load the MIDI file.
    let midi_data = &mut Cursor::new(midi_data);
    let midi_file = Arc::new(MidiFile::new(midi_data).unwrap());

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(sample_rate as i32);
    let synthesizer = Synthesizer::new(&SOUNDFONT, &settings).unwrap();
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file.
    sequencer.play(&midi_file, false);

    // The output buffer.
    let sample_count = (settings.sample_rate as f64 * midi_file.get_length()) as usize;
    let mut left: PcmSamples = vec![0_f32; sample_count];
    let mut right: PcmSamples = vec![0_f32; sample_count];

    // Render the waveform.
    sequencer.render(&mut left[..], &mut right[..]);

    // Write the waveform to the file.
    if is_stereo {
        let mut sample = Vec::with_capacity(sample_count * 2);
        for t in 0..left.len() {
            sample.push(left[t]);
            sample.push(right[t]);
        }
        sample
    } else {
        let mut sample = Vec::with_capacity(sample_count);
        for t in 0..left.len() {
            // Mix down to mono
            sample.push((left[t] + right[t]) * 0.5);
        }
        sample
    }
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

    #[test]
    fn music_sample_conversion_fails_on_unsupported_format() {
        let mus_data = b"MUS\x1Arest of the data";
        let mut synthesizer =
            MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 44_100).unwrap();
        let result = MusicSample::from_bytes(&mut synthesizer, mus_data, false);
        assert!(result.is_err());
    }

    #[test]
    fn music_sample_conversion_fails_on_unknown_format() {
        let unknown_data = b"XXXXrest of the data";
        let mut synthesizer =
            MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 44_100).unwrap();
        let result = MusicSample::from_bytes(&mut synthesizer, unknown_data, false);
        assert!(result.is_err());
    }

    #[test]
    fn music_sample_converts_midi_to_mono() {
        let midi_data = include_bytes!("../assets/midi/test.mid").as_slice();
        let music_sample = MusicSample::try_from(midi_data).unwrap();
        assert_eq!(music_sample.sample_rate(), 16000);
        assert_eq!(music_sample.channels(), 1);
        assert!(!music_sample.sample().is_empty());
    }

    #[test]
    fn music_sample_converts_midi_to_stereo() {
        let midi_data = include_bytes!("../assets/midi/test.mid").as_slice();
        let mut synthesizer =
            MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 44_100).unwrap();
        let music_sample = MusicSample::from_bytes(&mut synthesizer, midi_data, true).unwrap();
        assert_eq!(music_sample.sample_rate(), 44_100);
        assert_eq!(music_sample.channels(), 2);
        assert!(!music_sample.sample().is_empty());
    }

    #[test]
    fn midi_synthesizer_creation_fails_on_too_low_sample_rate() {
        let midi_data = b"MThdrest of the data";
        let result = MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 8_000);
        assert!(result.is_err());
    }

    #[test]
    fn midi_synthesizer_creation_fails_on_too_high_sample_rate() {
        let result = MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 100_000);
        assert!(result.is_err());
    }
}
