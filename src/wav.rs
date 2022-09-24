const WAV_FORMAT_PCM: u16 = 0x01;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Header {
    pub audio_format: u16,
    pub channel_count: u16,
    pub sampling_rate: u32,
    pub bytes_per_second: u32,
    pub bytes_per_sample: u16,
    pub bits_per_sample: u16,
}

impl Header {
    #[must_use]
    pub fn new(channel_count: u16, sampling_rate: u32) -> Header {
        let bits_per_sample = 16;
        Header {
            audio_format: WAV_FORMAT_PCM,
            channel_count,
            sampling_rate,
            bits_per_sample,
            bytes_per_second: u32::from((bits_per_sample >> 3) * channel_count) * sampling_rate,
            bytes_per_sample: (bits_per_sample >> 3) * channel_count,
        }
    }
}

impl From<Header> for [u8; 16] {
    #[allow(clippy::shadow_unrelated)]
    fn from(h: Header) -> Self {
        let mut v: [u8; 16] = [0; 16];

        let b = h.audio_format.to_le_bytes();
        v[0] = b[0];
        v[1] = b[1];
        let b = h.channel_count.to_le_bytes();
        v[2] = b[0];
        v[3] = b[1];
        let b = h.sampling_rate.to_le_bytes();
        v[4] = b[0];
        v[5] = b[1];
        v[6] = b[2];
        v[7] = b[3];
        let b = h.bytes_per_second.to_le_bytes();
        v[8] = b[0];
        v[9] = b[1];
        v[10] = b[2];
        v[11] = b[3];
        let b = h.bytes_per_sample.to_le_bytes();
        v[12] = b[0];
        v[13] = b[1];
        let b = h.bits_per_sample.to_le_bytes();
        v[14] = b[0];
        v[15] = b[1];

        v
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Loop {
    #[default]
    Forward = 0,
    PingPong = 1,
    Backward = 2,
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SampleLoop {
    id: u32,
    loop_type: Loop,
    start: u32,
    end: u32,
}

impl From<&SampleLoop> for [u8; 24] {
    fn from(s: &SampleLoop) -> Self {
        let mut v = [0u8; 24];
        v[0..4].copy_from_slice(&s.id.to_le_bytes());
        v[4..8].copy_from_slice(&(s.loop_type as u32).to_le_bytes());
        println!("start: {}, end: {}", s.start, s.end);
        v[8..12].copy_from_slice(&s.start.to_le_bytes());
        v[12..16].copy_from_slice(&s.end.to_le_bytes());
        v
    }
}

impl From<SampleLoop> for [u8; 24] {
    fn from(s: SampleLoop) -> Self {
        (&s).into()
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct SampleChunk {
    sample_rate: u32,
    sample_loops: Vec<SampleLoop>,
}

impl SampleChunk {
    pub fn new(sample_rate: u32, start: u32, end: u32) -> Self {
        let sample_loop = SampleLoop {
            id: 0,
            loop_type: Loop::Forward,
            start,
            end,
        };
        Self {
            sample_rate,
            sample_loops: vec![sample_loop],
        }
    }
}

impl From<&SampleChunk> for Vec<u8> {
    fn from(s: &SampleChunk) -> Self {
        let mut v = vec![];
        v.resize(0x2c - 8, 0);

        let sample_period = 1_000_000_000 / s.sample_rate;
        v[8..12].copy_from_slice(&sample_period.to_le_bytes());
        v[28..32].copy_from_slice(&(s.sample_loops.len() as u32).to_le_bytes());
        for sl in &s.sample_loops {
            let lv: [u8; 24] = sl.into();
            v.extend(lv);
        }
        v
    }
}

impl From<SampleChunk> for Vec<u8> {
    fn from(s: SampleChunk) -> Self {
        (&s).into()
    }
}

pub fn write<W>(
    header: Header,
    track: &[i16],
    sample: Option<SampleChunk>,
    writer: &mut W,
) -> std::io::Result<()>
where
    W: std::io::Write + std::io::Seek,
{
    const WAVE_ID: riff::ChunkId = riff::ChunkId {
        value: [b'W', b'A', b'V', b'E'],
    };
    const HEADER_ID: riff::ChunkId = riff::ChunkId {
        value: [b'f', b'm', b't', b' '],
    };
    const DATA_ID: riff::ChunkId = riff::ChunkId {
        value: [b'd', b'a', b't', b'a'],
    };
    const SMPL_ID: riff::ChunkId = riff::ChunkId {
        value: [b's', b'm', b'p', b'l'],
    };

    let h_vec: [u8; 16] = header.into();
    let h_dat = riff::ChunkContents::Data(HEADER_ID, Vec::from(h_vec));

    let mut chunks = vec![h_dat];

    if let Some(sample) = sample {
        let s_vec = sample.into();
        let s_dat = riff::ChunkContents::Data(SMPL_ID, s_vec);
        chunks.push(s_dat);
    }

    let mut d_vec: Vec<u8> = Vec::new();
    d_vec.reserve(2 * track.len());
    for sample in track {
        d_vec.extend(sample.to_le_bytes());
    }
    let d_dat = riff::ChunkContents::Data(DATA_ID, d_vec);
    chunks.push(d_dat);

    let r = riff::ChunkContents::Children(riff::RIFF_ID.clone(), WAVE_ID, chunks);

    r.write(writer)?;

    Ok(())
}
