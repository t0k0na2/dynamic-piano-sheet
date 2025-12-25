mod utils;
mod rectangle;
mod note;
mod bar;
mod synth;
use synth::SoundSource;
use bar::Bar;
use note::Note;
use rectangle::Rectangle;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array,};

use web_sys::{CanvasRenderingContext2d, File, AudioContext, DynamicsCompressorNode, GainNode};
use midly::{Format, Smf, Timing, TrackEventKind, MidiMessage, MetaMessage};

fn bpm_to_tempo(bpm: f64) -> f64{
    60000000.0 / bpm
}

fn calc_sec_per_tick(ticks_per_beat: u16, tempo: f64) -> f64{
    tempo as f64 * 0.000001 / ticks_per_beat as f64
}

pub fn parse_midi(data: &[u8]) -> Result<(Vec<Bar>, Vec<Note>, u8), String>{
    let smf = match Smf::parse(data){
        Ok(smf) => smf,
        Err(e) => {
            log!("Error parsing MIDI file: {:?}", e);
            return Err(format!("Failed to parse MIDI file"));
        }
    };

    if smf.header.format != Format::Parallel{
        return Err(format!("Parallelだけサポート").into());
    }

    let ticks_per_beat = match smf.header.timing {
        Timing::Timecode(_, _) => return Err(format!("タイムコードは未サポート")),
        Timing::Metrical(res) => res.as_int(),
    };

    #[derive(Default, Clone, Copy)]
    struct TrackState{
        interval_ticks: u32,
        currrent_index: usize,
        ended: bool,
    }
    let mut track_states: Vec<TrackState> = vec![TrackState::default(); smf.tracks.len()];

    for (i, track) in smf.tracks.iter().enumerate() {
        if track.is_empty(){
            continue;
        }
        track_states[i].currrent_index = 0;
        track_states[i].interval_ticks = track[track_states[i].currrent_index].delta.as_int();
    }

    let mut ticks_per_bar: usize = (ticks_per_beat * 4) as usize;  // デフォルトは4/4拍子にしとく
    let mut sec_per_tick = calc_sec_per_tick(ticks_per_beat, bpm_to_tempo(120.0));// とりあえず初期テンポ120BPM
    let mut current_time: f64 = 0.0;
    let mut remain_bar_ticks = 0;
    let mut bars: Vec<Bar> = Vec::new();
    let mut notes: Vec<Note> = Vec::new();
    let mut playing_notes: HashMap<(u8, u8), usize> = HashMap::new();

    while track_states.iter().any(|&state| state.ended == false) {
        
        // 小節情報
        if remain_bar_ticks <= 0{
            if let Some(bar) = bars.last_mut(){
                bar.set_end_time(current_time);
            }
            bars.push(Bar::new(current_time, -1.0, bars.len() as u32));

            remain_bar_ticks = ticks_per_bar;
        }

        for (i, track) in smf.tracks.iter().enumerate() {
            let track_state = &mut track_states[i];
            
            if track_state.ended{
                continue;
            }

            // 同じタイミングで複数のイベントが発生することがあるのでループで処理する
            while track_state.currrent_index < track.len(){

                if track_state.interval_ticks > 0{
                    track_state.interval_ticks -= 1;
                    break;
                }

                match track[track_state.currrent_index].kind {
                    TrackEventKind::Midi{channel, message} =>{
                        match message {
                            MidiMessage::NoteOn { key, vel } =>{
                                let hash_key = (channel.as_int(), key.as_int());
                                if vel > 0 {
                                    let note_id = notes.len();
                                    notes.push(Note::new(current_time, -1.0, key.as_int(), vel.as_int(), i as u8));
                                    if let Some(_) = playing_notes.insert(hash_key, note_id){
                                        return Err(format!("Error NoteOnが重複しました。"));
                                    }
                                }else{
                                    // vel0はNoteOff扱い?
                                    if let Some(id) = playing_notes.remove(&hash_key){
                                        notes[id].set_off_time(current_time);
                                    }
                                }
                            },
                            MidiMessage::NoteOff { key, .. } =>{
                                let hash_key = (channel.as_int(), key.as_int());
                                if let Some(id) = playing_notes.remove(&hash_key){
                                    notes[id].set_off_time(current_time);
                                }
                            },
                            _ => (),
                        }
                    }
                    TrackEventKind::Meta(message) =>{
                        match message{
                            MetaMessage::Tempo(tempo) => {
                                sec_per_tick = calc_sec_per_tick(ticks_per_beat, tempo.as_int() as f64);
                            },
                            MetaMessage::EndOfTrack =>{
                                track_state.ended = true;
                            },
                            MetaMessage::TimeSignature(num, denom, _ , _) =>{
                                ticks_per_bar = ticks_per_beat as usize / (2u32.pow(denom as u32) / 4) as usize * num as usize;   
                                remain_bar_ticks = ticks_per_bar;
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
                track_state.currrent_index += 1;
                if track_state.currrent_index < track.len(){
                    track_state.interval_ticks = track[track_state.currrent_index].delta.as_int();
                }
            }  
        }

        remain_bar_ticks -= 1;
        current_time += sec_per_tick;
    }

    if let Some(bar) = bars.last_mut(){
        bar.set_end_time(current_time + remain_bar_ticks as f64 * sec_per_tick);
    }

    Ok((bars, notes, smf.tracks.len() as u8))
}

fn calc_key_area(rect: &Rectangle, min_key: u8, max_key: u8) -> Vec<Rectangle>{
    // キーボードの１オクターブ分の鍵盤の比率位置テーブルを作成、黒鍵は白鍵にかぶさる上に幅や位置が等幅ではないので定義して使うことにした
    const OCTAVE_SIZE_RATIO_TABLE:[(f64, f64); 12] = [(0.0, 0.1428), (0.0951, 0.1666),(0.1428, 0.2857),(0.2618, 0.3333),(0.2857, 0.4285),(0.4285, 0.5714),(0.5237, 0.5952),(0.5714, 0.7142),(0.6784, 0.7499),(0.7142, 0.8571),(0.8332, 0.9047),(0.8571, 1.0)];

    let min_octave = min_key / 12;
    let min_note = min_key % 12;
    let _max_octave = max_key / 12;
    let max_note = max_key % 12;
    
    let base_offset_ratio = min_octave as f64 + OCTAVE_SIZE_RATIO_TABLE[min_note as usize].0;
    let octave_width = {
        let min_aligned = min_key + 12 - (min_key % 12);
        let max_aligned = max_key - (max_key % 12);
        let total_octave_ratio = (max_aligned - min_aligned) as f64 / 12.0 + (1.0 - OCTAVE_SIZE_RATIO_TABLE[min_note as usize].0) + OCTAVE_SIZE_RATIO_TABLE[max_note as usize].1;
        rect.width() / total_octave_ratio
    };

    let mut ret: Vec<Rectangle> = Vec::new();
    for key in min_key..=max_key{
        let note_index = (key % 12) as usize;
        let octave = (key / 12) as f64;
        let left = rect.left() + (octave + OCTAVE_SIZE_RATIO_TABLE[note_index].0 - base_offset_ratio) * octave_width;
        let right = rect.left() + (octave + OCTAVE_SIZE_RATIO_TABLE[note_index].1 - base_offset_ratio) * octave_width;
        ret.push(Rectangle::new(left, rect.top(), right - left, rect.height()));
    }
    ret
}

#[wasm_bindgen]
pub struct MidiPlayer{
    audio_context: AudioContext,
    comp: DynamicsCompressorNode,
    master_volume: GainNode,
    sound_sources: Vec<SoundSource>,
    bars: Vec<Bar>,
    notes: Vec<Note>,
    current_time: f64,
    playing: bool,
    display_range_sec: f64,
    num_tracks: u8,
}

#[wasm_bindgen]
impl MidiPlayer{
    pub fn new() -> Result<MidiPlayer, JsValue>{
        utils::set_panic_hook();
        let audio_context = AudioContext::new()?;

        let master_volume = audio_context.create_gain()?;
        master_volume.connect_with_audio_node(&audio_context.destination())?;

        // 音が重なるとノイズが気になるので出力の手前にコンプ刺す
        let comp = audio_context.create_dynamics_compressor()?;
        comp.threshold().set_value(-20.0);
        comp.knee().set_value(15.0);
        comp.ratio().set_value(20.0); 
        comp.connect_with_audio_node(&master_volume)?;

        Ok(MidiPlayer{
            audio_context: audio_context,
            comp: comp,
            master_volume: master_volume,
            bars: Vec::new(),
            notes: Vec::new(),
            current_time: 0.0,
            sound_sources: Vec::new(),
            playing: false,
            display_range_sec: 3.0,
            num_tracks: 0,
        })
    }

    pub async fn load_midi(&mut self, file: &File) -> Result<(), JsValue>{
        let buffer = JsFuture::from(file.array_buffer()).await?;
        let bin = Uint8Array::new(&buffer).to_vec();
        let parse_result = match parse_midi(&bin){
            Ok(parse_result) => {
                parse_result
            },
            Err(e) => {
                return Err(JsValue::from_str(&format!("Error parsing MIDI file: {}", e)));
            }
        };

        self.playing = false;
        self.current_time = 0.0;
        self.sound_sources.clear();
        self.bars = parse_result.0;
        self.notes = parse_result.1;
        self.num_tracks = parse_result.2;

        Ok(())
    }

    pub fn play(&mut self){
        if self.ready() == false{
            return;
        }
        self.playing = true;
    }

    pub fn stop(&mut self){
        self.playing = false;
        self.sound_sources.clear();
    }

    pub fn ready(&self) -> bool{
        self.notes.len() > 0 && self.bars.len() > 0
    }

    pub fn num_bars(&self) -> usize{
        self.bars.len()
    }

    pub fn volume(&self) -> f32{
        self.master_volume.gain().value()
    }

    pub fn set_volume(&mut self, volume: f32){
        self.master_volume.gain().set_value(volume);
    }

    pub fn current_bar(&self) -> usize{
        if self.bars.len() == 0{
            return 0;
        }

        if self.current_time < self.bars[0].begin_time(){
            return 0;
        }

        for bar in self.bars.iter(){
            if bar.begin_time() <= self.current_time && self.current_time < bar.end_time(){
                return bar.number() as usize;
            }
        }
        self.bars.len()
    }

    pub fn set_display_range(&mut self, range_sec: f64){
        self.display_range_sec = range_sec;
    }

    pub fn seek_bar(&mut self, bar: usize, clear_sounds:bool){
        let bar = &self.bars[bar.clamp(0, self.bars.len() - 1)];
        self.seek_time(bar.begin_time(), clear_sounds);
    }

    pub fn seek_time(&mut self, time: f64, clear_sounds:bool){
        if clear_sounds {
            self.sound_sources.clear();
        }
        self.current_time = time;
    }

    pub fn skip(&mut self, delta: f64, clear_sounds:bool){
        self.seek_time(self.current_time + delta, clear_sounds);
    }

    pub fn tick(&mut self, delta_time: f64) -> Result<(),JsValue>{
        if self.playing == false{
            return Ok(());
        }
        
        let delta_sec = delta_time / 1000.0;// ms -> s

        for sound_source in self.sound_sources.iter_mut(){
            sound_source.tick(delta_sec);
        }
        self.sound_sources.retain(|source| !source.finished());

        for note in self.notes.iter(){
            if self.current_time <= note.on_time() && note.on_time() < self.current_time + delta_sec{
                let start_time = self.audio_context.current_time() + (self.current_time + delta_sec - note.on_time());
                let end_time = start_time + (note.off_time() - note.on_time());
                self.sound_sources.push(SoundSource::new(&self.audio_context, &self.comp, note.key(), note.velocity(), start_time, end_time)?);
            }
        }

        self.current_time += delta_sec; 

        if self.current_bar() >= self.bars.len() {
            self.playing = false;
        }

        Ok(())
    }

    pub fn render(&self, context: &CanvasRenderingContext2d, left: f64, top: f64, width: f64, height: f64) -> Result<(), JsValue>{
        let keybord_height = height * 0.1;
        let min_key: u8 = 21;
        let max_key: u8 = 108;
        let rect = Rectangle::new(left, top, width, height);
        let key_areas = calc_key_area(&rect, min_key, max_key);
        
        // 背景
        context.set_fill_style_str("black");
        context.fill_rect(rect.left(), rect.top(), rect.width(), rect.height());

        // オクターブ分割線
        context.set_stroke_style_str("gray");
        for key in min_key..=max_key{
            if key % 12 == 0{
                let area = &key_areas[(key - min_key) as usize];
                context.begin_path();
                context.move_to(area.left(), area.top());
                context.line_to(area.left(), area.bottom());
                context.stroke();
            }
        }
        
        let display_start_sec = self.current_time;
        let display_end_sec = display_start_sec + self.display_range_sec;
        let pixel_per_sec = rect.height() / self.display_range_sec;
        let current_time_pos = rect.height() - keybord_height;

        // 小節線描画
        context.set_stroke_style_str("gray");
        context.set_fill_style_str("gray");
        context.set_text_align("right");
        context.set_text_baseline("bottom");
        context.set_font("32px sans-serif");
        for bar in self.bars.iter(){
            if bar.begin_time() > display_end_sec || bar.end_time() < display_start_sec {
                continue;
            }
            let bar_pos = current_time_pos - (bar.begin_time() - self.current_time) * pixel_per_sec;
            context.begin_path();
            context.move_to(rect.left(), bar_pos);
            context.line_to(rect.right(), bar_pos);
            context.stroke();
            context.fill_text(&(bar.number() + 1).to_string(), rect.right() - 2.0, bar_pos - 2.0)?;
            if bar.number() == self.bars.len() as u32 - 1{
                // 最後の小節線も描画
                let end_bar_pos = current_time_pos - (bar.end_time() - self.current_time) * pixel_per_sec;
                context.begin_path();
                context.move_to(rect.left(), end_bar_pos);
                context.line_to(rect.right(), end_bar_pos);
                context.stroke();
                context.fill_text("おわり", rect.right() - 2.0, end_bar_pos - 2.0)?;
            }
        }

        // ノート描画
        const TRACK_COLORS: [&str; 4] = ["#4682B4", "#E66101", "#009E73", "#7B4173"];
        for track_no in 0..self.num_tracks{
            let color_index = (track_no as usize % TRACK_COLORS.len()) as usize;
            context.set_stroke_style_str(TRACK_COLORS[color_index]);
            context.set_fill_style_str(TRACK_COLORS[color_index]);
            for note in self.notes.iter(){
                if note.track() != track_no{
                    continue;
                }

                if note.key() < min_key || note.key() > max_key{
                    continue;
                }

                if note.on_time() > display_end_sec || note.off_time() < display_start_sec{
                    continue;
                }

                let area = &key_areas[(note.key() - min_key) as usize];
                let note_top = current_time_pos - (note.off_time() - self.current_time) * pixel_per_sec;
                let note_height = current_time_pos - (note.on_time() - self.current_time) * pixel_per_sec - note_top;
                let note_left = area.left();
                let note_width = area.width();

                context.begin_path();
                context.round_rect_with_f64(note_left, note_top, note_width, note_height, 4.0)?;
                context.fill();
                context.stroke();
            }
        }
        

        // 白鍵
        let white_note_height = keybord_height;
        context.set_stroke_style_str("gray");
        context.set_fill_style_str("white");
        for key in min_key..=max_key{
            match key % 12{
                0 | 2 | 4 | 5 | 7 | 9 | 11 => {
                    let area = &key_areas[(key - min_key) as usize];
                    let top = area.bottom() - white_note_height;
                    context.fill_rect(area.left(), top, area.width(), white_note_height);
                    context.begin_path();
                    context.move_to(area.left(), top);
                    context.line_to(area.left(), area.bottom());
                    context.stroke();
                },
                _ => (),
            }
        }

        // 黒鍵
        let black_note_height = keybord_height * 0.6;
        context.set_fill_style_str("black");
        for key in min_key..=max_key{
            match key % 12{
                1 | 3 | 6 | 8 | 10 => {
                    let area = &key_areas[(key - min_key) as usize];
                    context.fill_rect(area.left(), area.bottom() - white_note_height, area.width(), black_note_height);
                },
                _ => (),
            }
        }

        Ok(())
    }
}

mod test{
    #[test]
    fn test_parse_midi(){
        let data = include_bytes!("../tests/assets/Test.mid");
        let result = super::parse_midi(data);
        assert!(result.is_ok());
        let midi = result.unwrap();
        assert!(midi.1.len() > 0);
        for note in &midi.1{
            println!("{:?}", note);
        }

        assert!(midi.0.len() > 0);
        for bar in &midi.0{
            println!("{:?}", bar);
        }
    }
}

/*
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell{
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        //let _timer = utils::Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                //log!(
                //    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                //    row,
                //    col,
                //    cell,
                //    live_neighbors
                //);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };
                // log!("    it becomes {:?}", next_cell);

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();
        let cells = (0..width * height)
            .map(|_| {
                if Math::random() >= 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }

}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
*/