use midly::MidiMessage;

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub tick: u64,
    pub message: MidiMessage,
}

pub struct Transport {
    pub bpm: f32,
    pub ticks_per_beat: u16,
    pub sample_rate: u32,

    samples_per_tick: f32,
    sample_accumulator: f32,
    pub current_tick: u64,
}

impl Transport {
    pub fn new(bpm: f32, ticks_per_beat: u16, sample_rate: u32) -> Self {
        let seconds_per_beat = 60.0 / bpm;
        let seconds_per_tick = seconds_per_beat / ticks_per_beat as f32;
        Transport {
            bpm,
            ticks_per_beat,
            sample_rate,
            samples_per_tick: seconds_per_tick * sample_rate as f32,
            sample_accumulator: 0.0,
            current_tick: 0,
        }
    }

    pub fn advance_samples(&mut self, num_samples: u32) -> u64 {
        self.sample_accumulator += num_samples as f32;

        let mut ticks = self.current_tick;
        while self.sample_accumulator >= self.samples_per_tick {
            self.sample_accumulator -= self.samples_per_tick;
            self.current_tick += 1;
            ticks += 1;
        }
        ticks
    }

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }
}

pub fn collect_midi_events(smf: &midly::Smf) -> Vec<MidiEvent> {
    let mut current_tick: u64 = 0;

    let mut events = smf
        .tracks
        .iter()
        .flat_map(|track| {
            track.iter().filter_map(move |event| {
                current_tick += event.delta.as_int() as u64;
                match &event.kind {
                    midly::TrackEventKind::Midi { message, .. } => Some(MidiEvent {
                        tick: current_tick,
                        message: *message,
                    }),
                    _ => None,
                }
            })
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|a| a.tick);
    events
}

pub struct MidiEventScheduler {
    events: Vec<MidiEvent>,
    event_index: usize,
}

impl MidiEventScheduler {
    pub fn new(events: Vec<MidiEvent>) -> Self {
        MidiEventScheduler {
            events,
            event_index: 0,
        }
    }

    pub fn process(&mut self, transport: &Transport, mut handler: impl FnMut(&MidiMessage)) {
        let current_tick = transport.current_tick();
        while self.event_index < self.events.len() {
            let event = unsafe { &self.events.get_unchecked(self.event_index) };
            if event.tick > current_tick {
                break;
            }
            handler(&event.message);
            self.event_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midly::{Header, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

    #[test]
    fn collect_midi_events_flatten_and_sort_events() {
        let smf = Smf {
            header: Header {
                format: midly::Format::SingleTrack,
                timing: Timing::Metrical(96.into()),
            },
            tracks: vec![
                vec![
                    TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOn {
                                key: 60.into(),
                                vel: 100.into(),
                            },
                        },
                    },
                    TrackEvent {
                        delta: 96.into(),
                        kind: TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOff {
                                key: 60.into(),
                                vel: 0.into(),
                            },
                        },
                    },
                ],
                vec![
                    TrackEvent {
                        delta: 10.into(),
                        kind: TrackEventKind::Midi {
                            channel: 1.into(),
                            message: MidiMessage::NoteOn {
                                key: 62.into(),
                                vel: 100.into(),
                            },
                        },
                    },
                    TrackEvent {
                        delta: 90.into(),
                        kind: TrackEventKind::Midi {
                            channel: 1.into(),
                            message: MidiMessage::NoteOff {
                                key: 62.into(),
                                vel: 0.into(),
                            },
                        },
                    },
                ],
            ],
        };

        let events = collect_midi_events(&smf);
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[1].tick, 10);
        assert_eq!(events[2].tick, 96);
        assert_eq!(events[3].tick, 100);
    }

    #[test]
    fn midi_event_scheduler_processes_events_at_correct_ticks() {
        let events = vec![
            MidiEvent { tick: 0, message: MidiMessage::NoteOn { key: 60.into(), vel: 100.into() } },
            MidiEvent { tick: 10, message: MidiMessage::NoteOff { key: 60.into(), vel: 0.into() } },
            MidiEvent { tick: 20, message: MidiMessage::NoteOn { key: 62.into(), vel: 100.into() } },
        ];
        let mut scheduler = MidiEventScheduler::new(events);
        let mut processed_messages = Vec::new();
        let mut transport = Transport::new(120.0, 96, 44100);
        scheduler.process(&transport, |message| {
            processed_messages.push(message.clone());
        });
        assert_eq!(processed_messages.len(), 1);
        assert_eq!(processed_messages[0], MidiMessage::NoteOn { key: 60.into(), vel: 100.into() });

        transport.advance_samples(44100 / 12); // Advance to tick 10
        scheduler.process(&transport, |message| {
            processed_messages.push(message.clone());
        });
        assert_eq!(processed_messages.len(), 2);
        assert_eq!(processed_messages[1], MidiMessage::NoteOff { key: 60.into(), vel: 0.into() });

        transport.advance_samples(44100 / 12); // Advance to tick 20
        scheduler.process(&transport, |message| {
            processed_messages.push(message.clone());
        });
        assert_eq!(processed_messages.len(), 3);
        assert_eq!(processed_messages[2], MidiMessage::NoteOn { key: 62.into(), vel: 100.into() });
    }
}
