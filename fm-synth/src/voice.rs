use crate::envelop::{Envelop, EnvelopStage};
use crate::wave_table::{
    WaveTable, WaveTableCollection, WaveTableSize, WaveTableType, generate_wave_tables,
};
const DOOM_ATTACK: f32 = 0.005;
const DOOM_RELEASE: f32 = 0.1;

#[derive(Debug, Clone)]
pub struct VoiceManager {
    wave_tables: WaveTableCollection,
    voices: Vec<Voice>,
}

impl VoiceManager {
    pub fn new(num_voices: usize, wave_table_size: WaveTableSize) -> Self {
        let mut voices = Vec::with_capacity(num_voices);
        let wave_tables = generate_wave_tables(wave_table_size);
        for _ in 0..num_voices {
            voices.push(Voice::new());
        }
        Self {
            wave_tables,
            voices,
        }
    }

    pub fn note_on(
        &mut self,
        wave_table_type: WaveTableType,
        frequency: f32,
        sample_rate: u32,
        amplitude: f32,
    ) -> Option<usize> {
        if let Some((index, voice)) = self
            .voices
            .iter_mut()
            .enumerate()
            .find(|(_, voice)| !voice.is_active())
        {
            let phase_increment = (frequency / sample_rate as f32)
                * self.wave_tables.get(&wave_table_type)?.len() as f32;
            voice.initialize(wave_table_type, sample_rate, phase_increment, amplitude);
            Some(index)
        } else {
            None
        }
    }

    pub fn note_off(&mut self, voice_index: usize) {
        if let Some(voice) = self.voices.get_mut(voice_index) {
            voice.note_off();
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut mixed_sample = 0.0;
        for voice in &mut self.voices {
            if let Some(wave_table_type) = voice.wave_table_type {
                if let Some(wave_table) = self.wave_tables.get(&wave_table_type) {
                    let sample = voice.next_sample(wave_table);
                    mixed_sample += sample;
                }
            }
        }
        mixed_sample
    }
}

#[derive(Default, Debug, Clone)]
pub struct Voice {
    wave_table_type: Option<WaveTableType>,
    envelop: Envelop,
    phase: f32,
    phase_increment: f32,
    amplitude: f32,
    active: bool,
}

impl Voice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(
        &mut self,
        wave_table_type: WaveTableType,
        sample_rate: u32,
        phase_increment: f32,
        amplitude: f32,
    ) {
        self.wave_table_type = Some(wave_table_type);
        self.phase = 0.0;
        self.phase_increment = phase_increment;
        self.amplitude = amplitude;
        self.envelop.initialize(sample_rate, DOOM_ATTACK, DOOM_RELEASE);
        self.envelop.note_on();
        self.set_active(true);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn next_sample(&mut self, wave_table: &WaveTable) -> f32 {
        let (el, es) = (self.envelop.next_sample(), self.envelop.current_state());
        if el <= 0.0 && es == EnvelopStage::Idle {
            self.active = false;
            0.0
        } else {
            self.process_sample(wave_table) * el
        }
    }

    fn process_sample(&mut self, wave_table: &WaveTable) -> f32 {
            let index = self.phase as usize % wave_table.len();
            let sample = wave_table[index] * self.amplitude;

            self.phase += self.phase_increment;
            if self.phase >= wave_table.len() as f32 {
                // prevent phase overflow
                self.phase -= wave_table.len() as f32;
            }

            sample
    }

    fn note_off(&mut self) {
        self.envelop.note_off();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_initialization_works() {
        let mut voice = Voice::new();
        voice.initialize(WaveTableType::Sine, 44100, 100.0, 0.5);
        assert_eq!(voice.wave_table_type, Some(WaveTableType::Sine));
        assert_eq!(voice.phase, 0.0);
        assert_eq!(voice.phase_increment, 100.0);
        assert_eq!(voice.amplitude, 0.5);
        assert_eq!(voice.envelop.current_state(), EnvelopStage::Attack);
        assert!(voice.is_active());
    }

    #[test]
    fn voice_next_sample_returns_zero_when_inactive() {
        let mut voice = Voice::new();
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let mut sample = 0.0;
        for _ in 0..256 {
            sample = voice.next_sample(wave_table.get(&WaveTableType::Sine).unwrap());
        };
        assert_eq!(sample, 0.0);
    }

    #[test]
    fn voice_next_sample_processes_sample_when_active() {
        let mut voice = Voice::new();
        voice.initialize(WaveTableType::Sine, 44100, 1.0, 1.0);
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        let mut sample = 0.0;
        for _ in 0..256 {
            sample = voice.next_sample(wave_table.get(&WaveTableType::Sine).unwrap());
        };
        assert_ne!(sample, 0.0);
    }

    #[test]
    fn voice_note_off_triggers_envelop_release() {
        let mut voice = Voice::new();
        voice.initialize(WaveTableType::Sine, 44100, 1.0, 1.0);
        assert_eq!(voice.envelop.current_state(), EnvelopStage::Attack);
        voice.note_off();
        assert_eq!(voice.envelop.current_state(), EnvelopStage::Release);
    }

    #[test]
    fn voice_deactivates_when_envelop_reaches_zero() {
        let mut voice = Voice::new();
        voice.initialize(WaveTableType::Sine, 44100, 1.0, 1.0);
        let wave_table = generate_wave_tables(WaveTableSize::B256);
        for _ in 0..10000 {
            voice.next_sample(wave_table.get(&WaveTableType::Sine).unwrap());
        }
        assert!(voice.is_active());
        voice.note_off();
        for _ in 0..10000 {
            voice.next_sample(wave_table.get(&WaveTableType::Sine).unwrap());
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn voice_manager_note_on_and_off_works() {
        let mut vm = VoiceManager::new(4, WaveTableSize::B256);
        let voice_index = vm.note_on(WaveTableType::Sine, 440.0, 44100, 0.5);
        assert!(voice_index.is_some());
        let index = voice_index.unwrap();
        assert!(vm.voices[index].is_active());
        vm.note_off(index);
        assert_eq!(vm.voices[index].envelop.current_state(), EnvelopStage::Release);
    }

    #[test]
    fn voice_manager_can_handle_multiple_voices() {
        let mut vm = VoiceManager::new(2, WaveTableSize::B256);
        let index1 = vm.note_on(WaveTableType::Sine, 440.0, 44100, 0.5).unwrap();
        let index2 = vm.note_on(WaveTableType::Square, 550.0, 44100, 0.5).unwrap();

        assert_ne!(index1, index2);
        assert!(vm.voices[index1].is_active());
        assert!(vm.voices[index2].is_active());
        assert!(vm.note_on(WaveTableType::Sawtooth, 660.0, 44100, 0.5).is_none());
    }

    #[test]
    fn voice_manager_next_reuses_voices_correctly() {
        let mut vm = VoiceManager::new(2, WaveTableSize::B256);
        let index1 = vm.note_on(WaveTableType::Sine, 440.0, 44100, 0.5).unwrap();
        let index2 = vm.note_on(WaveTableType::Sine, 440.0, 44100, 0.5).unwrap();
        assert!(vm.voices[index1].is_active());
        assert!(vm.voices[index2].is_active());

        // Simulate enough samples to let the voice finish
        for _ in 0..10000 {
            vm.next_sample();
        }
        vm.note_off(index1);
        for _ in 0..10000 {
            vm.next_sample();
        }
        assert!(!vm.voices[index1].is_active());

        // Now we should be able to reuse the voice
        let new_index = vm.note_on(WaveTableType::Square, 550.0, 44100, 0.5);
        assert_eq!(new_index, Some(index1));
        assert!(vm.voices[index1].is_active());
    }
}
