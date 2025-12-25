use std::fmt;

#[derive(Clone, Copy)]
pub struct Note{
    on_time: f64,
    off_time: f64,
    key: u8,
    velocity: u8,
    track: u8,
}

impl Note{
    pub fn new(on_time: f64, off_time: f64, key: u8, velocity: u8, track: u8) -> Self{
        Note{
            on_time,
            off_time,
            key,
            velocity,
            track
        }
    }
    pub fn on_time(&self) -> f64{
        self.on_time
    }
    pub fn off_time(&self) -> f64{
        self.off_time
    }
    pub fn set_off_time(&mut self, off_time: f64){
        self.off_time = off_time;
    }
    pub fn key(&self) -> u8{
        self.key
    }
    pub fn velocity(&self) -> u8{
        self.velocity
    }

    pub fn track(&self) -> u8{
        self.track
    }

    fn midi_key_to_note_name(key: u8) -> String{
        const SCALE: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "G#", "A", "A#", "B", "B#", "C"];
        format!("{}{}", key / 12, SCALE[(key % 12) as usize])
    }
}

impl fmt::Debug for Note{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Note {{ key: {}, on_time: {}, off_time: {}, velocity: {} }}", Self::midi_key_to_note_name(self.key), self.on_time, self.off_time, self.velocity)
    }
}