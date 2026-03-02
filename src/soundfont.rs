use std::convert::TryInto;
use std::fmt;

/// SF2の基本チャンク構造を示すエラー型
#[derive(Debug)]
pub enum SoundFontError {
    InvalidFormat,
    UnexpectedEof,
    ChunkNotFound(&'static str),
}

impl fmt::Display for SoundFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoundFontError::InvalidFormat => write!(f, "Invalid SoundFont format"),
            SoundFontError::UnexpectedEof => write!(f, "Unexpected end of file"),
            SoundFontError::ChunkNotFound(s) => write!(f, "Chunk not found: {}", s),
        }
    }
}

impl std::error::Error for SoundFontError {}

/// SoundFont 2 ジェネレータのオペレータID (0〜60)
/// 値は16bit整数 (WORD) として定義されています。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum GeneratorOperator {
    StartAddrsOffset = 0,
    EndAddrsOffset = 1,
    StartloopAddrsOffset = 2,
    EndloopAddrsOffset = 3,
    StartAddrsCoarseOffset = 4,
    ModLfoToPitch = 5,
    VibLfoToPitch = 6,
    ModEnvToPitch = 7,
    InitialFilterFc = 8,
    InitialFilterQ = 9,
    ModLfoToFilterFc = 10,
    ModEnvToFilterFc = 11,
    EndAddrsCoarseOffset = 12,
    ModLfoToVolume = 13,
    Unused1 = 14,
    ChorusEffectsSend = 15,
    ReverbEffectsSend = 16,
    Pan = 17,
    Unused2 = 18,
    Unused3 = 19,
    Unused4 = 20,
    DelayModLFO = 21,
    FreqModLFO = 22,
    DelayVibLFO = 23,
    FreqVibLFO = 24,
    DelayModEnv = 25,
    AttackModEnv = 26,
    HoldModEnv = 27,
    DecayModEnv = 28,
    SustainModEnv = 29,
    ReleaseModEnv = 30,
    KeynumToModEnvHold = 31,
    KeynumToModEnvDecay = 32,
    DelayVolEnv = 33,
    AttackVolEnv = 34,
    HoldVolEnv = 35,
    DecayVolEnv = 36,
    SustainVolEnv = 37,
    ReleaseVolEnv = 38,
    KeynumToVolEnvHold = 39,
    KeynumToVolEnvDecay = 40,
    Instrument = 41,
    Reserved1 = 42,
    KeyRange = 43,
    VelRange = 44,
    StartloopAddrsCoarseOffset = 45,
    Keynum = 46,
    Velocity = 47,
    InitialAttenuation = 48,
    Reserved2 = 49,
    EndloopAddrsCoarseOffset = 50,
    CoarseTune = 51,
    FineTune = 52,
    SampleID = 53,
    SampleModes = 54,
    Reserved3 = 55,
    ScaleTuning = 56,
    ExclusiveClass = 57,
    OverridingRootKey = 58,
    Unused5 = 59,
    EndOper = 60,
}

impl std::convert::TryFrom<u16> for GeneratorOperator {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GeneratorOperator::StartAddrsOffset),
            1 => Ok(GeneratorOperator::EndAddrsOffset),
            2 => Ok(GeneratorOperator::StartloopAddrsOffset),
            3 => Ok(GeneratorOperator::EndloopAddrsOffset),
            4 => Ok(GeneratorOperator::StartAddrsCoarseOffset),
            5 => Ok(GeneratorOperator::ModLfoToPitch),
            6 => Ok(GeneratorOperator::VibLfoToPitch),
            7 => Ok(GeneratorOperator::ModEnvToPitch),
            8 => Ok(GeneratorOperator::InitialFilterFc),
            9 => Ok(GeneratorOperator::InitialFilterQ),
            10 => Ok(GeneratorOperator::ModLfoToFilterFc),
            11 => Ok(GeneratorOperator::ModEnvToFilterFc),
            12 => Ok(GeneratorOperator::EndAddrsCoarseOffset),
            13 => Ok(GeneratorOperator::ModLfoToVolume),
            14 => Ok(GeneratorOperator::Unused1),
            15 => Ok(GeneratorOperator::ChorusEffectsSend),
            16 => Ok(GeneratorOperator::ReverbEffectsSend),
            17 => Ok(GeneratorOperator::Pan),
            18 => Ok(GeneratorOperator::Unused2),
            19 => Ok(GeneratorOperator::Unused3),
            20 => Ok(GeneratorOperator::Unused4),
            21 => Ok(GeneratorOperator::DelayModLFO),
            22 => Ok(GeneratorOperator::FreqModLFO),
            23 => Ok(GeneratorOperator::DelayVibLFO),
            24 => Ok(GeneratorOperator::FreqVibLFO),
            25 => Ok(GeneratorOperator::DelayModEnv),
            26 => Ok(GeneratorOperator::AttackModEnv),
            27 => Ok(GeneratorOperator::HoldModEnv),
            28 => Ok(GeneratorOperator::DecayModEnv),
            29 => Ok(GeneratorOperator::SustainModEnv),
            30 => Ok(GeneratorOperator::ReleaseModEnv),
            31 => Ok(GeneratorOperator::KeynumToModEnvHold),
            32 => Ok(GeneratorOperator::KeynumToModEnvDecay),
            33 => Ok(GeneratorOperator::DelayVolEnv),
            34 => Ok(GeneratorOperator::AttackVolEnv),
            35 => Ok(GeneratorOperator::HoldVolEnv),
            36 => Ok(GeneratorOperator::DecayVolEnv),
            37 => Ok(GeneratorOperator::SustainVolEnv),
            38 => Ok(GeneratorOperator::ReleaseVolEnv),
            39 => Ok(GeneratorOperator::KeynumToVolEnvHold),
            40 => Ok(GeneratorOperator::KeynumToVolEnvDecay),
            41 => Ok(GeneratorOperator::Instrument),
            42 => Ok(GeneratorOperator::Reserved1),
            43 => Ok(GeneratorOperator::KeyRange),
            44 => Ok(GeneratorOperator::VelRange),
            45 => Ok(GeneratorOperator::StartloopAddrsCoarseOffset),
            46 => Ok(GeneratorOperator::Keynum),
            47 => Ok(GeneratorOperator::Velocity),
            48 => Ok(GeneratorOperator::InitialAttenuation),
            49 => Ok(GeneratorOperator::Reserved2),
            50 => Ok(GeneratorOperator::EndloopAddrsCoarseOffset),
            51 => Ok(GeneratorOperator::CoarseTune),
            52 => Ok(GeneratorOperator::FineTune),
            53 => Ok(GeneratorOperator::SampleID),
            54 => Ok(GeneratorOperator::SampleModes),
            55 => Ok(GeneratorOperator::Reserved3),
            56 => Ok(GeneratorOperator::ScaleTuning),
            57 => Ok(GeneratorOperator::ExclusiveClass),
            58 => Ok(GeneratorOperator::OverridingRootKey),
            59 => Ok(GeneratorOperator::Unused5),
            60 => Ok(GeneratorOperator::EndOper),
            _ => Err("Invalid Generator Operator ID"),
        }
    }
}

/// ジェネレータのオペレータIDをインデックスとして、仕様書における元の名称を取得するための配列
pub const GENERATOR_NAMES: [&str; 61] = [
    "startAddrsOffset",           // 0
    "endAddrsOffset",             // 1
    "startloopAddrsOffset",       // 2
    "endloopAddrsOffset",         // 3
    "startAddrsCoarseOffset",     // 4
    "modLfoToPitch",              // 5
    "vibLfoToPitch",              // 6
    "modEnvToPitch",              // 7
    "initialFilterFc",            // 8
    "initialFilterQ",             // 9
    "modLfoToFilterFc",           // 10
    "modEnvToFilterFc",           // 11
    "endAddrsCoarseOffset",       // 12
    "modLfoToVolume",             // 13
    "unused1",                    // 14
    "chorusEffectsSend",          // 15
    "reverbEffectsSend",          // 16
    "pan",                        // 17
    "unused2",                    // 18
    "unused3",                    // 19
    "unused4",                    // 20
    "delayModLFO",                // 21
    "freqModLFO",                 // 22
    "delayVibLFO",                // 23
    "freqVibLFO",                 // 24
    "delayModEnv",                // 25
    "attackModEnv",               // 26
    "holdModEnv",                 // 27
    "decayModEnv",                // 28
    "sustainModEnv",              // 29
    "releaseModEnv",              // 30
    "keynumToModEnvHold",         // 31
    "keynumToModEnvDecay",        // 32
    "delayVolEnv",                // 33
    "attackVolEnv",               // 34
    "holdVolEnv",                 // 35
    "decayVolEnv",                // 36
    "sustainVolEnv",              // 37
    "releaseVolEnv",              // 38
    "keynumToVolEnvHold",         // 39
    "keynumToVolEnvDecay",        // 40
    "instrument",                 // 41
    "reserved1",                  // 42
    "keyRange",                   // 43
    "velRange",                   // 44
    "startloopAddrsCoarseOffset", // 45
    "keynum",                     // 46
    "velocity",                   // 47
    "initialAttenuation",         // 48
    "reserved2",                  // 49
    "endloopAddrsCoarseOffset",   // 50
    "coarseTune",                 // 51
    "fineTune",                   // 52
    "sampleID",                   // 53
    "sampleModes",                // 54
    "reserved3",                  // 55
    "scaleTuning",                // 56
    "exclusiveClass",             // 57
    "overridingRootKey",          // 58
    "unused5",                    // 59
    "endOper",                    // 60
];

// ---------------------------------------------------------
// 各レコードの構造体
// sfspec24.pdfに基づいたサイズとフィールド定義
// ---------------------------------------------------------

/// プリセットのヘッダー情報 (38 bytes)
#[derive(Debug, Clone)]
pub struct PresetHeader {
    pub name: String,
    pub preset: u16,
    pub bank: u16,
    pub preset_bag_ndx: u16,
    pub library: u32,
    pub genre: u32,
    pub morphology: u32,
}

/// プリセットのゾーンインデックス (4 bytes)
#[derive(Debug, Clone)]
pub struct PresetBag {
    pub gen_ndx: u16,
    pub mod_ndx: u16,
}

/// モジュレータ定義 (10 bytes)
#[derive(Debug, Clone)]
pub struct Modulator {
    pub mod_src_oper: u16,
    pub mod_dest_oper: u16,
    pub mod_amount: i16,
    pub mod_amt_src_oper: u16,
    pub mod_trans_oper: u16,
}

/// ジェネレータ定義 (4 bytes)
#[derive(Debug, Clone)]
pub struct Generator {
    pub gen_oper: u16,
    pub gen_amount: i16, // 値はu16やi16のユニオンだが基本的に16bit値
}

/// インストゥルメントのヘッダー情報 (22 bytes)
#[derive(Debug, Clone)]
pub struct Instrument {
    pub name: String,
    pub inst_bag_ndx: u16,
}

/// インストゥルメントのゾーンインデックス (4 bytes)
#[derive(Debug, Clone)]
pub struct InstrumentBag {
    pub inst_gen_ndx: u16,
    pub inst_mod_ndx: u16,
}

/// サンプルパッチのヘッダー情報 (46 bytes)
#[derive(Debug, Clone)]
pub struct SampleHeader {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub start_loop: u32,
    pub end_loop: u32,
    pub sample_rate: u32,
    pub original_pitch: u8,
    pub pitch_correction: i8,
    pub sample_link: u16,
    pub sample_type: u16,
}

/// 全体のSoundFont構造体
#[derive(Debug, Clone, Default)]
pub struct SoundFont {
    pub preset_headers: Vec<PresetHeader>,
    pub preset_bags: Vec<PresetBag>,
    pub preset_modulators: Vec<Modulator>,
    pub preset_generators: Vec<Generator>,
    pub instruments: Vec<Instrument>,
    pub instrument_bags: Vec<InstrumentBag>,
    pub instrument_modulators: Vec<Modulator>,
    pub instrument_generators: Vec<Generator>,
    pub sample_headers: Vec<SampleHeader>,

    /// f32に正規化された波形データ ("smpl" チャンク内のデータ)
    pub sample_data: Vec<f32>,
}

// ---------------------------------------------------------
// バイト列をパースするユーティリティ
// ---------------------------------------------------------

fn read_u8(data: &[u8], offset: &mut usize) -> Result<u8, SoundFontError> {
    if *offset + 1 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let val = data[*offset];
    *offset += 1;
    Ok(val)
}

fn read_i8(data: &[u8], offset: &mut usize) -> Result<i8, SoundFontError> {
    if *offset + 1 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let val = data[*offset] as i8;
    *offset += 1;
    Ok(val)
}

fn read_u16(data: &[u8], offset: &mut usize) -> Result<u16, SoundFontError> {
    if *offset + 2 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let val = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    Ok(val)
}

fn read_i16(data: &[u8], offset: &mut usize) -> Result<i16, SoundFontError> {
    if *offset + 2 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let val = i16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap());
    *offset += 2;
    Ok(val)
}

fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, SoundFontError> {
    if *offset + 4 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let val = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    Ok(val)
}

fn read_string(data: &[u8], offset: &mut usize, len: usize) -> Result<String, SoundFontError> {
    if *offset + len > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let slice = &data[*offset..*offset + len];
    let end = slice.iter().position(|&c| c == 0).unwrap_or(len);
    let s = String::from_utf8_lossy(&slice[..end]).into_owned();
    *offset += len;
    Ok(s)
}

fn read_chunk_header(data: &[u8], offset: &mut usize) -> Result<([u8; 4], u32), SoundFontError> {
    if *offset + 8 > data.len() {
        return Err(SoundFontError::UnexpectedEof);
    }
    let mut id = [0; 4];
    id.copy_from_slice(&data[*offset..*offset + 4]);
    *offset += 4;
    let size = read_u32(data, offset)?;
    Ok((id, size))
}

// ---------------------------------------------------------
// メインのパースロジック
// ---------------------------------------------------------

impl SoundFont {
    /// SF2のバイト列全体からSoundFont構造体を構築します
    pub fn parse(data: &[u8]) -> Result<Self, SoundFontError> {
        let mut sf = SoundFont::default();
        let mut offset = 0;

        // RIFF ヘッダーの確認
        let (id, size) = read_chunk_header(data, &mut offset)?;
        if &id != b"RIFF" {
            return Err(SoundFontError::InvalidFormat);
        }

        // フォームタイプ ("sfbk") の確認
        let mut form_type = [0; 4];
        form_type.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;
        if &form_type != b"sfbk" {
            return Err(SoundFontError::InvalidFormat);
        }

        let end_offset = (offset - 4) + size as usize;

        while offset < end_offset && offset + 8 <= data.len() {
            let (list_id, list_size) = read_chunk_header(data, &mut offset)?;

            if &list_id != b"LIST" {
                offset += list_size as usize;
                // padding
                if list_size % 2 != 0 {
                    offset += 1;
                }
                continue;
            }

            let mut list_type = [0; 4];
            list_type.copy_from_slice(&data[offset..offset + 4]);
            offset += 4;

            let list_end = offset - 4 + list_size as usize;

            match &list_type {
                b"INFO" => {
                    // INFOチャンクのパース（必要ならメタ情報を抽出するが、今回はスキップ）
                }
                b"sdta" => {
                    // sdta（サンプルデータ）のパース
                    while offset < list_end {
                        let (chunk_id, chunk_size) = read_chunk_header(data, &mut offset)?;
                        if &chunk_id == b"smpl" {
                            let sample_count = chunk_size as usize / 2;
                            sf.sample_data = Vec::with_capacity(sample_count);
                            for _ in 0..sample_count {
                                let sample = read_i16(data, &mut offset)?;
                                sf.sample_data.push(sample as f32 / 32768.0);
                            }
                        } else {
                            offset += chunk_size as usize;
                        }
                        if chunk_size % 2 != 0 {
                            offset += 1;
                        }
                    }
                }
                b"pdta" => {
                    // pdta (プリセット・インストゥルメント情報のパース)
                    while offset < list_end {
                        let (chunk_id, chunk_size) = read_chunk_header(data, &mut offset)?;
                        let chunk_end = offset + chunk_size as usize;

                        match &chunk_id {
                            b"phdr" => {
                                let record_size = 38;
                                let count = chunk_size as usize / record_size;
                                for _ in 0..count {
                                    sf.preset_headers.push(PresetHeader {
                                        name: read_string(data, &mut offset, 20)?,
                                        preset: read_u16(data, &mut offset)?,
                                        bank: read_u16(data, &mut offset)?,
                                        preset_bag_ndx: read_u16(data, &mut offset)?,
                                        library: read_u32(data, &mut offset)?,
                                        genre: read_u32(data, &mut offset)?,
                                        morphology: read_u32(data, &mut offset)?,
                                    });
                                }
                            }
                            b"pbag" => {
                                let record_size = 4;
                                let count = chunk_size as usize / record_size;
                                for _ in 0..count {
                                    sf.preset_bags.push(PresetBag {
                                        gen_ndx: read_u16(data, &mut offset)?,
                                        mod_ndx: read_u16(data, &mut offset)?,
                                    });
                                }
                            }
                            b"pmod" | b"imod" => {
                                let record_size = 10;
                                let count = chunk_size as usize / record_size;
                                let mut mods = Vec::with_capacity(count);
                                for _ in 0..count {
                                    mods.push(Modulator {
                                        mod_src_oper: read_u16(data, &mut offset)?,
                                        mod_dest_oper: read_u16(data, &mut offset)?,
                                        mod_amount: read_i16(data, &mut offset)?,
                                        mod_amt_src_oper: read_u16(data, &mut offset)?,
                                        mod_trans_oper: read_u16(data, &mut offset)?,
                                    });
                                }
                                if &chunk_id == b"pmod" {
                                    sf.preset_modulators = mods;
                                } else {
                                    sf.instrument_modulators = mods;
                                }
                            }
                            b"pgen" | b"igen" => {
                                let record_size = 4;
                                let count = chunk_size as usize / record_size;
                                let mut gens = Vec::with_capacity(count);
                                for _ in 0..count {
                                    gens.push(Generator {
                                        gen_oper: read_u16(data, &mut offset)?,
                                        gen_amount: read_i16(data, &mut offset)?,
                                    });
                                }
                                if &chunk_id == b"pgen" {
                                    sf.preset_generators = gens;
                                } else {
                                    sf.instrument_generators = gens;
                                }
                            }
                            b"inst" => {
                                let record_size = 22;
                                let count = chunk_size as usize / record_size;
                                for _ in 0..count {
                                    sf.instruments.push(Instrument {
                                        name: read_string(data, &mut offset, 20)?,
                                        inst_bag_ndx: read_u16(data, &mut offset)?,
                                    });
                                }
                            }
                            b"ibag" => {
                                let record_size = 4;
                                let count = chunk_size as usize / record_size;
                                for _ in 0..count {
                                    sf.instrument_bags.push(InstrumentBag {
                                        inst_gen_ndx: read_u16(data, &mut offset)?,
                                        inst_mod_ndx: read_u16(data, &mut offset)?,
                                    });
                                }
                            }
                            b"shdr" => {
                                let record_size = 46;
                                let count = chunk_size as usize / record_size;
                                for _ in 0..count {
                                    sf.sample_headers.push(SampleHeader {
                                        name: read_string(data, &mut offset, 20)?,
                                        start: read_u32(data, &mut offset)?,
                                        end: read_u32(data, &mut offset)?,
                                        start_loop: read_u32(data, &mut offset)?,
                                        end_loop: read_u32(data, &mut offset)?,
                                        sample_rate: read_u32(data, &mut offset)?,
                                        original_pitch: read_u8(data, &mut offset)?,
                                        pitch_correction: read_i8(data, &mut offset)?,
                                        sample_link: read_u16(data, &mut offset)?,
                                        sample_type: read_u16(data, &mut offset)?,
                                    });
                                }
                            }
                            _ => {
                                // 未知のチャンクはスキップ
                            }
                        }

                        offset = chunk_end; // 念のためポインタ位置を保証
                        if chunk_size % 2 != 0 {
                            offset += 1;
                        }
                    }
                }
                _ => {
                    // それ以外のLISTはスキップ
                }
            }

            // padding
            offset = list_end;
            if list_size % 2 != 0 {
                offset += 1;
            }
        }

        Ok(sf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid_riff() {
        let data = b"RIFX\x00\x00\x00\x00sfbk";
        let err = SoundFont::parse(data).unwrap_err();
        assert!(matches!(err, SoundFontError::InvalidFormat));
    }

    #[test]
    fn test_parse_invalid_sfbk() {
        let data = b"RIFF\x04\x00\x00\x00wave";
        let err = SoundFont::parse(data).unwrap_err();
        assert!(matches!(err, SoundFontError::InvalidFormat));
    }

    #[test]
    fn test_parse_empty_sfbk() {
        let mut data = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(b"sfbk");

        let sf = SoundFont::parse(&data).unwrap();
        assert_eq!(sf.preset_headers.len(), 0);
        assert_eq!(sf.sample_data.len(), 0);
    }

    #[test]
    fn test_parse_real_sfbk() {
        // テスト用のsf2ファイルを読み込み
        let data = std::fs::read("test.sf2");
        if let Ok(data) = data {
            let sf = SoundFont::parse(&data);
            assert!(sf.is_ok(), "Failed to parse realistic soundfont file");

            let sf = sf.unwrap();
            // 基本的なパースが完了し、レコードがいくつか読み込まれていることを確認
            assert!(sf.preset_headers.len() > 0);
            assert!(sf.sample_headers.len() > 0);
            assert!(sf.sample_data.len() > 0);

            sf.preset_headers.iter().for_each(|preset| {
                println!(
                    "Preset p:{} b:{} name:{}",
                    preset.preset, preset.bank, preset.name
                );
            });

            /*
            sf.sample_headers.iter().for_each(|sample| {
                println!(
                    "Sample name:{} start:{} end:{} smpl_rate:{}",
                    sample.name, sample.start, sample.end, sample.sample_rate
                );
            });

                        let mut preset_gen_counts = std::collections::BTreeMap::new();
                        for generator in &sf.preset_generators {
                            *preset_gen_counts.entry(generator.gen_oper).or_insert(0) += 1;
                        }
                        for (gen_oper, _count) in &preset_gen_counts {
                            println!("Preset gen_oper {}", GENERATOR_NAMES[*gen_oper as usize]);
                        }

                        let mut inst_gen_counts = std::collections::BTreeMap::new();
                        for generator in &sf.instrument_generators {
                            *inst_gen_counts.entry(generator.gen_oper).or_insert(0) += 1;
                        }
                        for (gen_oper, _count) in &inst_gen_counts {
                            println!(
                                "Instrument gen_oper {}",
                                GENERATOR_NAMES[*gen_oper as usize]
                            );
                        }
            */
            println!(
                "Parsed SF2: {} presets, {} samples, {} waveform words",
                sf.preset_headers.len(),
                sf.sample_headers.len(),
                sf.sample_data.len()
            );
        } else {
            // ファイルが存在しない環境でもCIが通るようにスキップ
            println!("test.sf2 not found. Skipping real file test.");
        }
    }
}
