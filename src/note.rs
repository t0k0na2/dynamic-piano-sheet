use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumeEvent {
    pub time: f64,
    pub volume: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchBendEvent {
    pub time: f64,
    pub bend: i16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanEvent {
    pub time: f64,
    pub pan: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReverbEvent {
    pub time: f64,
    pub reverb: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChorusEvent {
    pub time: f64,
    pub chorus: u8,
}

#[derive(Clone)]
pub struct Note {
    on_time: f64,
    off_time: f64,
    key: u8,
    velocity: u8,
    channel_volumes: Vec<VolumeEvent>,
    pitch_bends: Vec<PitchBendEvent>,
    pans: Vec<PanEvent>,
    reverbs: Vec<ReverbEvent>,
    choruses: Vec<ChorusEvent>,
    pitch_bend_sensitivity: f32,
    track: u8,
    program: u8,
    bank: u16,
}

impl Note {
    pub fn new(
        on_time: f64,
        off_time: f64,
        key: u8,
        velocity: u8,
        channel_volume: u8,
        pitch_bend: i16,
        pan: u8,
        reverb: u8,
        chorus: u8,
        pitch_bend_sensitivity: f32,
        track: u8,
        program: u8,
        bank: u16,
    ) -> Self {
        Note {
            on_time,
            off_time,
            key,
            velocity,
            channel_volumes: vec![VolumeEvent {
                time: on_time,
                volume: channel_volume,
            }],
            pitch_bends: vec![PitchBendEvent {
                time: on_time,
                bend: pitch_bend,
            }],
            pans: vec![PanEvent { time: on_time, pan }],
            reverbs: vec![ReverbEvent {
                time: on_time,
                reverb,
            }],
            choruses: vec![ChorusEvent {
                time: on_time,
                chorus,
            }],
            pitch_bend_sensitivity,
            track,
            program,
            bank,
        }
    }
    pub fn on_time(&self) -> f64 {
        self.on_time
    }
    pub fn off_time(&self) -> f64 {
        self.off_time
    }
    pub fn set_off_time(&mut self, off_time: f64) {
        self.off_time = off_time;
    }
    pub fn key(&self) -> u8 {
        self.key
    }
    pub fn velocity(&self) -> u8 {
        self.velocity
    }

    pub fn channel_volumes(&self) -> &Vec<VolumeEvent> {
        &self.channel_volumes
    }

    pub fn add_channel_volume(&mut self, time: f64, volume: u8) {
        self.channel_volumes.push(VolumeEvent { time, volume });
    }

    pub fn pitch_bends(&self) -> &Vec<PitchBendEvent> {
        &self.pitch_bends
    }

    pub fn add_pitch_bend(&mut self, time: f64, bend: i16) {
        self.pitch_bends.push(PitchBendEvent { time, bend });
    }

    pub fn pans(&self) -> &Vec<PanEvent> {
        &self.pans
    }

    pub fn add_pan(&mut self, time: f64, pan: u8) {
        self.pans.push(PanEvent { time, pan });
    }

    pub fn reverbs(&self) -> &Vec<ReverbEvent> {
        &self.reverbs
    }

    pub fn add_reverb(&mut self, time: f64, reverb: u8) {
        self.reverbs.push(ReverbEvent { time, reverb });
    }

    pub fn choruses(&self) -> &Vec<ChorusEvent> {
        &self.choruses
    }

    pub fn add_chorus(&mut self, time: f64, chorus: u8) {
        self.choruses.push(ChorusEvent { time, chorus });
    }

    pub fn pitch_bend_sensitivity(&self) -> f32 {
        self.pitch_bend_sensitivity
    }

    pub fn track(&self) -> u8 {
        self.track
    }

    pub fn program(&self) -> u8 {
        self.program
    }
    pub fn bank(&self) -> u16 {
        self.bank
    }

    fn midi_key_to_note_name(key: u8) -> String {
        const SCALE: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "G#", "A", "A#", "B", "B#", "C",
        ];
        format!("{}{}", key / 12, SCALE[(key % 12) as usize])
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Note {{ key: {}, on_time: {}, off_time: {}, track: {}, velocity: {}, program: {}, bank: {}}}",
            Self::midi_key_to_note_name(self.key),
            self.on_time,
            self.off_time,
            self.track,
            self.velocity,
            self.program,
            self.bank
        )
    }
}
