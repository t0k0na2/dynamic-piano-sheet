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

    /// 16-bit PCM波形データ ("smpl" チャンク内のデータ)
    pub sample_data: Vec<i16>,
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
                if list_size % 2 != 0 { offset += 1; }
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
                                sf.sample_data.push(read_i16(data, &mut offset)?);
                            }
                        } else {
                            offset += chunk_size as usize;
                        }
                        if chunk_size % 2 != 0 { offset += 1; }
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
                        if chunk_size % 2 != 0 { offset += 1; }
                    }
                }
                _ => {
                    // それ以外のLISTはスキップ
                }
            }
            
            // padding
            offset = list_end;
            if list_size % 2 != 0 { offset += 1; }
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
                println!("Preset p:{} b:{} name:{}", preset.preset, preset.bank, preset.name);
            });

            sf.sample_headers.iter().for_each(|sample| {
                println!("Sample name:{} start:{} end:{} smpl_rate:{}", sample.name, sample.start, sample.end, sample.sample_rate);
            });
            
            println!("Parsed SF2: {} presets, {} samples, {} waveform words", 
                sf.preset_headers.len(), sf.sample_headers.len(), sf.sample_data.len());
        } else {
            // ファイルが存在しない環境でもCIが通るようにスキップ
            println!("test.sf2 not found. Skipping real file test.");
        }
    }
}
