use std::path::PathBuf;
use std::time::Duration;

pub struct Song {
    asset: SongAsset,
    meta: SongMeta,
    charts: Vec<Chart>,
}

pub struct SongMeta {
    title: String,
    artist: String,
    length: Duration,
    bpm: f64,
    illu: IlluAsset
}

pub struct Chart {
    asset: ChartAsset,
    meta: ChartMeta
}

pub struct ChartMeta {
    charter: String,
    level: u8,
    desc: String, 
}
pub enum AssetLocation {
    Local(PathBuf),
    Remote {
        url: String,
        checksum: Option<String>,
    },
}

pub struct SongAsset {
    audio: AssetLocation,
}

pub struct ChartAsset {
    chart_file: AssetLocation,
}

pub struct IlluAsset {
    illu: AssetLocation
}

pub enum Rank {
    SSSPlus,
    SSS,
    SSPlus,
    SS,
    SPlus,
    S,
    AAA,
    AA,
    A,
    B,
    C,
    D
}