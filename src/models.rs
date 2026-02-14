pub(crate) use crate::asset::{IlluAsset, SongAsset};
use crate::core::chart::Chart;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct Song {
    pub asset: SongAsset,
    pub meta: SongMeta,
    pub charts: Vec<Chart>,
    pub illu: Option<IlluAsset>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SongMeta {
    pub title: String,
    pub artist: String,
    pub length: Duration,
    pub bpm: f64,
}