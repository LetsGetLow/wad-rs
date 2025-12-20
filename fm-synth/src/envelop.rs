#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EnvelopStage {
    #[default]
    Idle,
    Attack,
    Release,
}

/// Simple ADSR envelope with only Attack and Release stages
/// level goes from 0.0 to 1.0 during Attack
/// level goes from 1.0 to 0.0 during Release
/// level is 0.0 during Idle
/// Note: This is a simplified version and does not include Decay and Sustain stages
/// Also, the envelope immediately goes to Idle after reaching the peak in Attack stage
/// and after reaching 0.0 in Release stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelop {
    level: f32,
    attack_inc: f32,
    release_inc: f32,
    state: EnvelopStage,
}

impl Default for Envelop {
    fn default() -> Self {
        Envelop {
            level: 0.0,
            attack_inc: 0.0,
            release_inc: 0.0,
            state: EnvelopStage::Idle,
        }
    }
}

impl Envelop {
    pub fn note_on(&mut self) {
        self.state = EnvelopStage::Attack;
    }

    pub fn note_off(&mut self) {
        self.state = EnvelopStage::Release;
    }


    pub fn initialize(&mut self, sample_rate: u32, attack_time: f32, release_time: f32) {
        let sample_rate = sample_rate as f32;
        let attack_inc = if attack_time > 0.0 {
            1.0 / (attack_time * sample_rate)
        } else {
            1.0
        };
        let release_inc = if release_time > 0.0 {
            1.0 / (release_time * sample_rate)
        } else {
            1.0
        };

        self.level = 0.0;
        self.state = EnvelopStage::Idle;
        self.attack_inc = attack_inc;
        self.release_inc = release_inc;
    }

    pub fn current_state(&self) -> EnvelopStage {
        self.state
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopStage::Attack => {
                self.level += self.attack_inc;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvelopStage::Idle;
                }
            }
            EnvelopStage::Release => {
                self.level -= self.release_inc;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvelopStage::Idle;
                }
            }
            EnvelopStage::Idle => {}

        }
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelop_do_nothing_when_in_idle() {
        let mut env = Envelop::default();
        env.initialize(44100, 0.1, 0.2); // 0.1 seconds attack, 0.2 seconds release
        assert_eq!(env.current_state(), EnvelopStage::Idle);
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_envelop_attack() {
        let mut env = Envelop::default();
        env.initialize(44100, 0.1, 0.2); // 0.1 seconds attack, 0.2 seconds release
        assert_eq!(env.current_state(), EnvelopStage::Idle);
        env.note_on();
        assert_eq!(env.current_state(), EnvelopStage::Attack);
        let mut level = 0.0;
        for _ in 0..4411 { // 4410 samples for 0.1 seconds at 44100 Hz + 1 extra to reach 1.0
            level = env.next_sample();
        }
        assert_eq!(level, 1.0, "Level: {level}");
        assert_eq!(env.current_state(), EnvelopStage::Idle, "State: {:?}", env.current_state());
    }

    #[test]
    fn test_envelop_release() {
        let mut env = Envelop::default();
        env.initialize(44100, 0.1, 0.2); // 0.1 seconds attack, 0.2 seconds release
        env.note_on();
        for _ in 0..4411 { // Reach peak level
            env.next_sample();
        }
        env.note_off();
        assert_eq!(env.current_state(), EnvelopStage::Release);
        let mut level = 1.0;
        for _ in 0..8821 { // 8820 samples for 0.2 seconds at 44100 Hz + 1 extra to reach 0.0
            level = env.next_sample();
        }
        assert_eq!(level, 0.0, "Level: {level}");
        assert_eq!(env.current_state(), EnvelopStage::Idle, "State: {:?}", env.current_state());
    }

    #[test]
    fn test_envelop_no_attack_release() {
        let mut env = Envelop::default();
        env.initialize(44100, 0.0, 0.0); // No attack, no release
        env.note_on();
        let level = env.next_sample();
        assert_eq!(level, 1.0, "Level: {level}");
        env.note_off();
        let level = env.next_sample();
        assert_eq!(level, 0.0, "Level: {level}");
    }
}