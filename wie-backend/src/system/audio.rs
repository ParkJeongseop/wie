use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec, vec::Vec};

use smaf_player::{SmafEvent, parse_smaf};

use crate::{AudioCommand, AudioEventData, AudioHandle, AudioSequence, AudioSink, TimedAudioEvent};

#[derive(Debug)]
pub enum AudioError {
    InvalidHandle,
}

pub struct Audio {
    sink: Box<dyn AudioSink>,
    files: BTreeMap<AudioHandle, Arc<AudioSequence>>,
    // whether each started playback repeats
    playing: BTreeMap<AudioHandle, bool>,
    last_audio_handle: AudioHandle,
    volume: u8,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>) -> Self {
        Self {
            sink,
            files: BTreeMap::new(),
            playing: BTreeMap::new(),
            last_audio_handle: 0,
            volume: 100,
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> Result<AudioHandle, AudioError> {
        let audio_handle = self.last_audio_handle;
        let sequence = Arc::new(convert_smaf_events(parse_smaf(data)));

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, sequence);

        Ok(audio_handle)
    }

    pub fn play(&mut self, audio_handle: AudioHandle, repeat: bool) -> Result<(), AudioError> {
        let sequence = self.files.get(&audio_handle).cloned().ok_or(AudioError::InvalidHandle)?;

        self.stop(audio_handle);
        self.playing.insert(audio_handle, repeat);
        if self.volume != 0 {
            self.sink.send(AudioCommand::Play {
                handle: audio_handle,
                sequence,
                repeat,
            });
        }

        Ok(())
    }

    /// How long the sequence plays once, in milliseconds.
    pub fn duration(&self, audio_handle: AudioHandle) -> Result<u64, AudioError> {
        self.files.get(&audio_handle).map(|x| x.duration).ok_or(AudioError::InvalidHandle)
    }

    pub fn stop(&mut self, audio_handle: AudioHandle) {
        if self.playing.remove(&audio_handle).is_some() && self.volume != 0 {
            self.sink.send(AudioCommand::Stop { handle: audio_handle });
        }
    }

    /// The device's volume, 0 (silent) to 100. Every clip plays through it.
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Only silence reaches the output: titles turn their sound off by setting the volume to 0 and go on playing.
    pub fn set_volume(&mut self, volume: u8) {
        let was_silent = self.volume == 0;
        self.volume = volume.min(100);

        if self.volume == 0 && !was_silent {
            for &handle in self.playing.keys() {
                self.sink.send(AudioCommand::Stop { handle });
            }
        } else if self.volume != 0 && was_silent {
            // a clip that plays once has had its turn by now
            self.playing.retain(|_, repeat| *repeat);
            for &handle in self.playing.keys() {
                if let Some(sequence) = self.files.get(&handle) {
                    self.sink.send(AudioCommand::Play {
                        handle,
                        sequence: sequence.clone(),
                        repeat: true,
                    });
                }
            }
        }
    }

    pub fn close(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        self.stop(audio_handle);

        if self.files.remove(&audio_handle).is_none() {
            return Err(AudioError::InvalidHandle);
        }

        Ok(())
    }
}

fn convert_smaf_events(events: Vec<(usize, SmafEvent)>) -> AudioSequence {
    let duration = events.iter().map(|(time, _)| *time as u64).max().unwrap_or(0);
    let events = events
        .into_iter()
        .filter_map(|(time, event)| {
            let data = match event {
                SmafEvent::Wave {
                    channel,
                    sampling_rate,
                    data,
                } => AudioEventData::Wave {
                    channels: channel,
                    sampling_rate,
                    samples: data,
                },
                SmafEvent::MidiNoteOn { channel, note, velocity } => AudioEventData::Midi(vec![0x90 | channel, note, velocity]),
                SmafEvent::MidiNoteOff { channel, note, velocity } => AudioEventData::Midi(vec![0x80 | channel, note, velocity]),
                SmafEvent::MidiProgramChange { channel, program } => AudioEventData::Midi(vec![0xc0 | channel, program]),
                SmafEvent::MidiControlChange { channel, control, value } => AudioEventData::Midi(vec![0xb0 | channel, control, value]),
                SmafEvent::MidiPitchBend { channel, value } => {
                    AudioEventData::Midi(vec![0xe0 | channel, (value & 0x7f) as u8, ((value >> 7) & 0x7f) as u8])
                }
                SmafEvent::MidiSysEx(data) => AudioEventData::Midi(data),
                SmafEvent::End => return None,
            };

            Some(TimedAudioEvent { time: time as u64, data })
        })
        .collect();

    AudioSequence { duration, events }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use std::sync::Mutex;

    use smaf_player::SmafEvent;

    use super::{Audio, convert_smaf_events};
    use crate::{AudioCommand, AudioEventData, AudioSequence, AudioSink, TimedAudioEvent};

    struct RecordingSink(Arc<Mutex<Vec<AudioCommand>>>);

    impl AudioSink for RecordingSink {
        fn send(&self, command: AudioCommand) {
            self.0.lock().unwrap().push(command);
        }
    }

    #[test]
    fn converts_smaf_events_to_timed_transport() {
        let sequence = convert_smaf_events(vec![
            (5, SmafEvent::MidiProgramChange { channel: 2, program: 7 }),
            (10, SmafEvent::MidiPitchBend { channel: 3, value: 0x1234 }),
            (15, SmafEvent::End),
        ]);

        assert_eq!(
            sequence,
            AudioSequence {
                duration: 15,
                events: vec![
                    TimedAudioEvent {
                        time: 5,
                        data: AudioEventData::Midi(vec![0xc2, 7]),
                    },
                    TimedAudioEvent {
                        time: 10,
                        data: AudioEventData::Midi(vec![0xe3, 0x34, 0x24]),
                    },
                ],
            }
        );
    }

    #[test]
    fn replay_stops_previous_playback_before_starting_again() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(RecordingSink(commands.clone())));
        let handle = audio.load_smaf(&[]).unwrap();

        audio.play(handle, false).unwrap();
        audio.play(handle, true).unwrap();

        let commands = commands.lock().unwrap();
        let AudioCommand::Play {
            sequence: first_sequence,
            repeat: false,
            ..
        } = &commands[0]
        else {
            panic!("expected initial play command");
        };
        assert_eq!(commands[1], AudioCommand::Stop { handle });
        let AudioCommand::Play {
            sequence: second_sequence,
            repeat: true,
            ..
        } = &commands[2]
        else {
            panic!("expected replay command");
        };
        assert!(Arc::ptr_eq(first_sequence, second_sequence));
    }

    #[test]
    fn close_stops_playback_and_removes_the_handle() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(RecordingSink(commands.clone())));
        let handle = audio.load_smaf(&[]).unwrap();

        audio.play(handle, false).unwrap();
        audio.close(handle).unwrap();

        assert_eq!(commands.lock().unwrap()[1], AudioCommand::Stop { handle });
        assert!(audio.play(handle, false).is_err());
    }

    #[test]
    fn zero_volume_silences_playback_until_it_is_raised() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut audio = Audio::new(Box::new(RecordingSink(commands.clone())));
        let looped = audio.load_smaf(&[]).unwrap();
        let once = audio.load_smaf(&[]).unwrap();

        audio.play(looped, true).unwrap();
        audio.set_volume(0);
        audio.play(once, false).unwrap();
        audio.stop(once);
        audio.play(once, false).unwrap();
        assert_eq!(commands.lock().unwrap()[1..], [AudioCommand::Stop { handle: looped }]);

        audio.set_volume(60);
        assert_eq!(audio.volume(), 60);
        let commands = commands.lock().unwrap();
        assert!(matches!(commands[2], AudioCommand::Play { handle, repeat: true, .. } if handle == looped));
        assert_eq!(commands.len(), 3);
    }
}
