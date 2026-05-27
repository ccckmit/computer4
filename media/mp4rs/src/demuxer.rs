use std::io::{Read, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

use crate::error::{Mp4Error, Result};
use crate::types::*;

pub(crate) struct SampleEntry {
    pub(super) offset: u64,
    pub(super) size: u32,
}

pub(crate) struct TrackEntry {
    pub(super) info: TrackInfo,
    pub(super) entries: Vec<SampleEntry>,
    pub(super) avc: Option<AvcNalInfo>,
}

pub struct Demuxer {
    pub(super) info: Mp4Info,
    pub(super) tracks: Vec<TrackEntry>,
}

// ---- box reader ----

struct BoxReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> BoxReader<R> {
    #[allow(dead_code)]
    fn new(reader: R) -> Self {
        BoxReader { reader }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(buf)?;
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut b = [0u8; 2];
        self.read_exact(&mut b)?;
        Ok(u16::from_be_bytes(b))
    }

    #[allow(dead_code)]
    fn read_u24(&mut self) -> Result<u32> {
        let mut b = [0u8; 3];
        self.read_exact(&mut b)?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(u32::from_be_bytes(b))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut b = [0u8; 8];
        self.read_exact(&mut b)?;
        Ok(u64::from_be_bytes(b))
    }

    fn read_skip(&mut self, n: u64) -> Result<()> {
        if n > 0 {
            self.reader.seek(SeekFrom::Current(n as i64))?;
        }
        Ok(())
    }

    fn seek(&mut self, pos: u64) -> Result<()> {
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    fn tell(&mut self) -> Result<u64> {
        Ok(self.reader.seek(SeekFrom::Current(0))?)
    }

    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut v = vec![0u8; n];
        self.read_exact(&mut v)?;
        Ok(v)
    }

    fn read_fourcc(&mut self) -> Result<[u8; 4]> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(b)
    }

    #[allow(dead_code)]
    fn read_cstring(&mut self, n: usize) -> Result<String> {
        let bytes = self.read_bytes(n)?;
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(n);
        Ok(String::from_utf8_lossy(&bytes[..end]).to_string())
    }
}

// ---- box types as u32 ----

const BOX_FTYP: u32 = u32::from_be_bytes(*b"ftyp");
const BOX_MOOV: u32 = u32::from_be_bytes(*b"moov");
const BOX_MVHD: u32 = u32::from_be_bytes(*b"mvhd");
const BOX_TRAK: u32 = u32::from_be_bytes(*b"trak");
const BOX_TKHD: u32 = u32::from_be_bytes(*b"tkhd");
const BOX_MDIA: u32 = u32::from_be_bytes(*b"mdia");
const BOX_MDHD: u32 = u32::from_be_bytes(*b"mdhd");
const BOX_HDLR: u32 = u32::from_be_bytes(*b"hdlr");
const BOX_MINF: u32 = u32::from_be_bytes(*b"minf");
const BOX_STBL: u32 = u32::from_be_bytes(*b"stbl");
const BOX_STSD: u32 = u32::from_be_bytes(*b"stsd");
const BOX_STTS: u32 = u32::from_be_bytes(*b"stts");
const BOX_STSC: u32 = u32::from_be_bytes(*b"stsc");
const BOX_STSZ: u32 = u32::from_be_bytes(*b"stsz");
const BOX_STCO: u32 = u32::from_be_bytes(*b"stco");
const BOX_CO64: u32 = u32::from_be_bytes(*b"co64");
const BOX_STSS: u32 = u32::from_be_bytes(*b"stss");
const BOX_MDAT: u32 = u32::from_be_bytes(*b"mdat");
const BOX_AVC1: u32 = u32::from_be_bytes(*b"avc1");
const BOX_AVCC: u32 = u32::from_be_bytes(*b"avcC");
#[allow(dead_code)]
const BOX_VMHD: u32 = u32::from_be_bytes(*b"vmhd");
#[allow(dead_code)]
const BOX_SMHD: u32 = u32::from_be_bytes(*b"smhd");

// ---- ISO language code (macintosh 3-char packed) to string ----

fn mac_lang_to_str(code: u16) -> [u8; 4] {
    if (code & 0x8000) != 0 {
        let packed = code & 0x7FFF;
        let a = ((packed >> 10) & 0x1F) + 0x60;
        let b = ((packed >> 5) & 0x1F) + 0x60;
        let c = (packed & 0x1F) + 0x60;
        [a as u8, b as u8, c as u8, 0]
    } else {
        [b'u', b'n', b'd', 0]
    }
}

// ---- parser ----

impl Demuxer {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let file_size = file.metadata()?.len();
        let mut br = BoxReader::new(file);

        let mut ftyp_box: Option<([u8; 4], u32, Vec<[u8; 4]>)> = None;
        let mut mvhd_box: Option<(u32, u64)> = None; // (timescale, duration)
        let mut tracks: Vec<TrackEntry> = Vec::new();
        let mut mdat_boxes: Vec<(u64, u64)> = Vec::new(); // (offset, size)

        // First pass: locate all top-level boxes
        let mut pos: u64 = 0;
        while pos < file_size {
            br.seek(pos)?;
            let (box_type, box_size, header_size) = read_box_header(&mut br, file_size)?;

            if box_type == BOX_FTYP {
                let major = br.read_fourcc()?;
                let minor = br.read_u32()?;
                let mut brands = Vec::new();
                let remaining = box_size - header_size - 8;
                let mut read = 0u64;
                while read < remaining {
                    brands.push(br.read_fourcc()?);
                    read += 4;
                }
                ftyp_box = Some((major, minor, brands));
            } else if box_type == BOX_MOOV {
                parse_moov(&mut br, box_size, header_size, &mut mvhd_box, &mut tracks)?;
            } else if box_type == BOX_MDAT {
                mdat_boxes.push((pos + header_size, box_size - header_size));
            }

            pos += box_size;
        }

        let (major, minor, brands) = ftyp_box.ok_or(Mp4Error::InvalidData("no ftyp box"))?;
        let (tscale, dur) = mvhd_box.ok_or(Mp4Error::NoMoovBox)?;

        let info = Mp4Info {
            major_brand: major,
            minor_version: minor,
            compatible_brands: brands,
            timescale: tscale,
            duration: dur,
            tracks: tracks.iter().map(|t| t.info.clone()).collect(),
        };

        // Build sample index for each track from sample tables
        // This requires seeking back to parse stbl contents stored during moov parsing
        // Actually we parse it all during moov parsing now.

        Ok(Demuxer { info, tracks })
    }

    #[allow(dead_code)]
    pub fn track_samples(&self, track_idx: usize) -> Result<&[SampleEntry]> {
        Ok(&self.tracks[track_idx].entries)
    }

    #[allow(dead_code)]
    pub fn avc_config(&self, track_idx: usize) -> Option<&AvcNalInfo> {
        self.tracks[track_idx].avc.as_ref()
    }
}

fn read_box_header<R: Read + Seek>(br: &mut BoxReader<R>, file_size: u64) -> Result<(u32, u64, u64)> {
    let size32 = br.read_u32()?;
    let box_type = br.read_u32()?;
    let mut box_size = size32 as u64;
    let mut header_size = 8u64;

    if size32 == 1 {
        box_size = br.read_u64()?;
        header_size = 16;
    } else if size32 == 0 {
        box_size = file_size - br.tell()? + 8;
    }

    Ok((box_type, box_size, header_size))
}

fn parse_moov<R: Read + Seek>(
    br: &mut BoxReader<R>,
    moov_size: u64,
    moov_header: u64,
    mvhd_out: &mut Option<(u32, u64)>,
    tracks: &mut Vec<TrackEntry>,
) -> Result<()> {
    let end_pos = br.tell()? + moov_size - moov_header;
    while br.tell()? < end_pos {
        let box_start = br.tell()?;
        let (box_type, box_size, header_size) = read_box_header(br, end_pos)?;
        match box_type {
            BOX_MVHD => { parse_mvhd(br, mvhd_out)?; }
            BOX_TRAK => { parse_trak(br, box_size, header_size, tracks)?; }
            _ => {}
        }
        // skip remaining bytes of this box (if any)
        let consumed = br.tell()? - box_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }
    Ok(())
}

fn parse_mvhd<R: Read + Seek>(br: &mut BoxReader<R>, out: &mut Option<(u32, u64)>) -> Result<()> {
    let version = br.read_u8()?;
    br.read_skip(3)?; // flags
    if version == 0 {
        br.read_skip(4 + 4)?; // creation_time, modification_time (32-bit)
        let timescale = br.read_u32()?;
        let duration = br.read_u32()?;
        *out = Some((timescale, duration as u64));
    } else {
        br.read_skip(8 + 8)?; // creation_time, modification_time (64-bit)
        let timescale = br.read_u32()?;
        let duration = br.read_u64()?;
        *out = Some((timescale, duration));
    }
    Ok(())
}

fn parse_trak<R: Read + Seek>(
    br: &mut BoxReader<R>,
    trak_size: u64,
    trak_header: u64,
    tracks: &mut Vec<TrackEntry>,
) -> Result<()> {
    let end_pos = br.tell()? + trak_size - trak_header;
    let mut tkhd: Option<(u32, u64)> = None;
    let mut mdhd: Option<(u32, u64, u16)> = None;
    let mut hdlr: Option<u32> = None;
    let mut stts_entries: Vec<(u32, u32)> = Vec::new();
    let mut stsc_entries: Vec<(u32, u32, u32)> = Vec::new();
    let mut stsz_sample_size: u32 = 0;
    let mut sz_samples: Vec<u32> = Vec::new();
    let mut stco_entries: Vec<u64> = Vec::new();
    let mut stss_entries: Vec<u32> = Vec::new();
    let mut video_width: Option<u32> = None;
    let mut video_height: Option<u32> = None;
    let mut audio_ch: Option<u32> = None;
    let mut audio_sr: Option<u32> = None;
    let mut avcc_data: Option<Vec<u8>> = None;

    while br.tell()? < end_pos {
        let sub_start = br.tell()?;
        let (box_type, box_size, header_size) = read_box_header(br, end_pos)?;

        match box_type {
            BOX_TKHD => {
                let version = br.read_u8()?;
                br.read_skip(3)?;
                if version == 0 {
                    br.read_skip(4 + 4)?;
                    let track_id = br.read_u32()?;
                    br.read_skip(4)?;
                    let duration = br.read_u32()?;
                    tkhd = Some((track_id, duration as u64));
                } else {
                    br.read_skip(8 + 8)?;
                    let track_id = br.read_u32()?;
                    br.read_skip(4)?;
                    let duration = br.read_u64()?;
                    tkhd = Some((track_id, duration));
                }
            }
            BOX_MDIA => {
                parse_mdia(br, box_size, header_size, &mut mdhd, &mut hdlr,
                    &mut video_width, &mut video_height, &mut audio_ch, &mut audio_sr,
                    &mut avcc_data, &mut stts_entries, &mut stsc_entries,
                    &mut stsz_sample_size, &mut sz_samples,
                    &mut stco_entries, &mut stss_entries)?;
            }
            _ => {}
        }
        let consumed = br.tell()? - sub_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }

    if let (Some((tid, _)), Some(hdlr_code)) = (tkhd, hdlr) {
        let kind = TrackKind::from_handler(hdlr_code);
        let (tscale, dur, lang_code) = mdhd.unwrap_or((1, 0, 0));
        let language = mac_lang_to_str(lang_code);

        let entries = if !stco_entries.is_empty() && !stsc_entries.is_empty() && !sz_samples.is_empty() {
            build_sample_entries(&stsc_entries, &stco_entries, &stsz_sample_size, &sz_samples)
        } else {
            Vec::new()
        };

        let avc = avcc_data.map(|data| parse_avcc(&data));

        let info = TrackInfo {
            track_id: tid,
            kind,
            sample_count: sz_samples.len() as u32,
            duration: dur,
            timescale: tscale,
            width: video_width,
            height: video_height,
            channel_count: audio_ch,
            sample_rate: audio_sr,
            language,
        };

        tracks.push(TrackEntry { info, entries, avc });
    }

    Ok(())
}

fn parse_mdia<R: Read + Seek>(
    br: &mut BoxReader<R>,
    mdia_size: u64,
    mdia_header: u64,
    mdhd_out: &mut Option<(u32, u64, u16)>,
    hdlr_out: &mut Option<u32>,
    video_width: &mut Option<u32>,
    video_height: &mut Option<u32>,
    audio_ch: &mut Option<u32>,
    audio_sr: &mut Option<u32>,
    avcc_data: &mut Option<Vec<u8>>,
    stts_entries: &mut Vec<(u32, u32)>,
    stsc_entries: &mut Vec<(u32, u32, u32)>,
    stsz_sample_size: &mut u32,
    stsz_samples: &mut Vec<u32>,
    stco_entries: &mut Vec<u64>,
    stss_entries: &mut Vec<u32>,
) -> Result<()> {
    let end_pos = br.tell()? + mdia_size - mdia_header;
    while br.tell()? < end_pos {
        let sub_start = br.tell()?;
        let (box_type, box_size, header_size) = read_box_header(br, end_pos)?;

        match box_type {
            BOX_MDHD => {
                let version = br.read_u8()?;
                br.read_skip(3)?;
                if version == 0 {
                    br.read_skip(4 + 4)?;
                    let timescale = br.read_u32()?;
                    let duration = br.read_u32()?;
                    let lang = br.read_u16()?;
                    br.read_skip(2)?;
                    *mdhd_out = Some((timescale, duration as u64, lang));
                } else {
                    br.read_skip(8 + 8)?;
                    let timescale = br.read_u32()?;
                    let duration = br.read_u64()?;
                    let lang = br.read_u16()?;
                    br.read_skip(2)?;
                    *mdhd_out = Some((timescale, duration, lang));
                }
            }
            BOX_HDLR => {
                br.read_skip(4)?;
                br.read_skip(4)?;
                let handler = br.read_u32()?;
                *hdlr_out = Some(handler);
            }
            BOX_MINF => {
                parse_minf(br, box_size - header_size, video_width, video_height, audio_ch, audio_sr,
                    avcc_data, stts_entries, stsc_entries,
                    stsz_sample_size, stsz_samples, stco_entries, stss_entries)?;
            }
            _ => {}
        }
        let consumed = br.tell()? - sub_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }
    Ok(())
}

fn parse_minf<R: Read + Seek>(
    br: &mut BoxReader<R>,
    minf_payload: u64,
    video_width: &mut Option<u32>,
    video_height: &mut Option<u32>,
    audio_ch: &mut Option<u32>,
    audio_sr: &mut Option<u32>,
    avcc_data: &mut Option<Vec<u8>>,
    stts_entries: &mut Vec<(u32, u32)>,
    stsc_entries: &mut Vec<(u32, u32, u32)>,
    stsz_sample_size: &mut u32,
    stsz_samples: &mut Vec<u32>,
    stco_entries: &mut Vec<u64>,
    stss_entries: &mut Vec<u32>,
) -> Result<()> {
    let end_pos = br.tell()? + minf_payload;
    while br.tell()? < end_pos {
        let sub_start = br.tell()?;
        let (box_type, box_size, _header_size) = read_box_header(br, end_pos)?;
        match box_type {
            BOX_STBL => {
                parse_stbl(br, box_size, video_width, video_height, audio_ch, audio_sr,
                    avcc_data, stts_entries, stsc_entries,
                    stsz_sample_size, stsz_samples, stco_entries, stss_entries)?;
            }
            _ => {}
        }
        let consumed = br.tell()? - sub_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }
    Ok(())
}

fn parse_stbl<R: Read + Seek>(
    br: &mut BoxReader<R>,
    stbl_box_size: u64,
    video_width: &mut Option<u32>,
    video_height: &mut Option<u32>,
    audio_ch: &mut Option<u32>,
    audio_sr: &mut Option<u32>,
    avcc_data: &mut Option<Vec<u8>>,
    stts_entries: &mut Vec<(u32, u32)>,
    stsc_entries: &mut Vec<(u32, u32, u32)>,
    stsz_sample_size: &mut u32,
    sz_samples: &mut Vec<u32>,
    stco_entries: &mut Vec<u64>,
    stss_entries: &mut Vec<u32>,
) -> Result<()> {
    let stbl_start = br.tell()? - 8; // back up to box header start
    let end_pos = stbl_start + stbl_box_size;
    while br.tell()? < end_pos {
        let sub_start = br.tell()?;
        let (box_type, box_size, _) = read_box_header(br, end_pos)?;
        match box_type {
            BOX_STSD => {
                parse_stsd(br, box_size, video_width, video_height, avcc_data)?;
            }
            BOX_STTS => {
                br.read_skip(4)?;
                let count = br.read_u32()?;
                for _ in 0..count {
                    let sc = br.read_u32()?;
                    let sd = br.read_u32()?;
                    stts_entries.push((sc, sd));
                }
            }
            BOX_STSC => {
                br.read_skip(4)?;
                let count = br.read_u32()?;
                for _ in 0..count {
                    let fc = br.read_u32()?;
                    let spc = br.read_u32()?;
                    let sdi = br.read_u32()?;
                    stsc_entries.push((fc, spc, sdi));
                }
            }
            BOX_STSZ => {
                br.read_skip(4)?;
                let sample_size = br.read_u32()?;
                *stsz_sample_size = sample_size;
                let count = br.read_u32()?;
                if sample_size == 0 {
                    for _ in 0..count {
                        sz_samples.push(br.read_u32()?);
                    }
                }
            }
            BOX_STCO => {
                br.read_skip(4)?;
                let count = br.read_u32()?;
                for _ in 0..count {
                    stco_entries.push(br.read_u32()? as u64);
                }
            }
            BOX_CO64 => {
                br.read_skip(4)?;
                let count = br.read_u32()?;
                for _ in 0..count {
                    stco_entries.push(br.read_u64()?);
                }
            }
            BOX_STSS => {
                br.read_skip(4)?;
                let count = br.read_u32()?;
                for _ in 0..count {
                    stss_entries.push(br.read_u32()?);
                }
            }
            _ => {}
        }
        let consumed = br.tell()? - sub_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }
    Ok(())
}

fn parse_stsd<R: Read + Seek>(
    br: &mut BoxReader<R>,
    stsd_box_size: u64,
    video_width: &mut Option<u32>,
    video_height: &mut Option<u32>,
    avcc_data: &mut Option<Vec<u8>>,
) -> Result<()> {
    let stsd_start = br.tell()? - 8;
    let end_pos = stsd_start + stsd_box_size;
    br.read_skip(4)?;
    let _entry_count = br.read_u32()?;

    while br.tell()? < end_pos {
        let sub_start = br.tell()?;
        let (box_type, box_size, _) = read_box_header(br, end_pos)?;
        if box_type == BOX_AVC1 {
            br.read_skip(6)?;
            br.read_skip(2)?;
            br.read_skip(2)?;
            br.read_skip(2)?;
            br.read_skip(4 + 4 + 4)?;
            let width = br.read_u16()? as u32;
            let height = br.read_u16()? as u32;
            *video_width = Some(width);
            *video_height = Some(height);
            br.read_skip(4 + 4 + 4 + 2 + 32 + 2 + 2)?;
            // remaining avc1 contains avcC boxes
            let remaining_avc1 = box_size - (br.tell()? - sub_start);
            let avc1_end = br.tell()? + remaining_avc1;
            while br.tell()? + 8 <= avc1_end {
                let inner_start = br.tell()?;
                let (inner_type, inner_size, _) = read_box_header(br, avc1_end)?;
                if inner_type == BOX_AVCC {
                    let avc_c_len = inner_size - 8;
                    *avcc_data = Some(br.read_bytes(avc_c_len as usize)?);
                }
                let inner_consumed = br.tell()? - inner_start;
                let inner_remaining = inner_size - inner_consumed;
                if inner_remaining > 0 {
                    br.read_skip(inner_remaining)?;
                }
            }
        }
        let consumed = br.tell()? - sub_start;
        let remaining = box_size - consumed;
        if remaining > 0 {
            br.read_skip(remaining)?;
        }
    }
    Ok(())
}

fn build_sample_entries(
    stsc: &[(u32, u32, u32)],
    stco: &[u64],
    stsz_sample_size: &u32,
    stsz_samples: &[u32],
) -> Vec<SampleEntry> {
    let mut entries = Vec::new();
    let _sample_count = if *stsz_sample_size > 0 {
        // all samples same size, compute from stco and stsc
        let total: u32 = stsc.iter().map(|(_, spc, _)| spc).sum();
        total as usize
    } else {
        stsz_samples.len()
    };

    // Build sample → chunk mapping
    // stsc entries: (first_chunk_1based, samples_per_chunk, sample_desc_index)
    let mut chunk_sample_counts = vec![0u32; stco.len()];
    for i in 0..stsc.len() {
        let (first, spc, _) = stsc[i];
        let last = if i + 1 < stsc.len() { stsc[i + 1].0 - 1 } else { stco.len() as u32 };
        for c in first..=last.min(stco.len() as u32) {
            if c > 0 && (c as usize) <= stco.len() {
                chunk_sample_counts[(c - 1) as usize] = spc;
            }
        }
    }

    let mut sample_idx = 0u32;
    for (chunk_idx, &chunk_off) in stco.iter().enumerate() {
        let spc = if chunk_idx < chunk_sample_counts.len() {
            chunk_sample_counts[chunk_idx]
        } else {
            1
        };
        let mut chunk_pos = chunk_off;
        for _ in 0..spc {
            let size = if *stsz_sample_size > 0 {
                *stsz_sample_size
            } else {
                if (sample_idx as usize) < stsz_samples.len() {
                    stsz_samples[sample_idx as usize]
                } else {
                    break;
                }
            };
            entries.push(SampleEntry { offset: chunk_pos, size });
            chunk_pos += size as u64;
            sample_idx += 1;
        }
    }

    entries
}

fn parse_avcc(data: &[u8]) -> AvcNalInfo {
    let mut sps_list = Vec::new();
    let mut pps_list = Vec::new();
    if data.len() < 7 { return AvcNalInfo { sps_list, pps_list }; }

    let _config_version = data[0];
    let _profile = data[1];
    let _compat = data[2];
    let _level = data[3];
    let _nal_len_size = (data[4] & 3) + 1;
    let sps_count = data[5] & 0x1F;

    let mut offset = 6usize;
    for _ in 0..sps_count {
        if offset + 2 > data.len() { break; }
        let len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        if offset + len > data.len() { break; }
        sps_list.push(data[offset..offset + len].to_vec());
        offset += len;
    }

    if offset >= data.len() { return AvcNalInfo { sps_list, pps_list }; }
    let pps_count = data[offset];
    offset += 1;
    for _ in 0..pps_count {
        if offset + 2 > data.len() { break; }
        let len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        if offset + len > data.len() { break; }
        pps_list.push(data[offset..offset + len].to_vec());
        offset += len;
    }

    AvcNalInfo { sps_list, pps_list }
}
