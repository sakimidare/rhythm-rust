use crate::core::chart::{json_to_chart, Chart};
use crate::models::{IlluAsset, Song, SongAsset, SongMeta};
use crate::asset::AssetLocation;
use anyhow::bail;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::config::{json_to_config, GlobalConfig};

#[derive(Debug, Deserialize, Serialize)]
pub struct SongConfig {
    pub meta: SongMeta,
    pub audio_file: String,
    pub chart_files: Vec<String>,
    pub illu_file: Option<String>
}

pub fn load_config<T>(path: T) -> anyhow::Result<GlobalConfig>
where
    T: AsRef<Path>
{
    info!("Reading config: {:?}", path.as_ref());
    let config_json = fs::read_to_string(&path)
        .inspect_err(|e| error!("Error reading config: {e}"))?;

    Ok(
        json_to_config(&config_json)
            .inspect_err(|e| error!("Error converting the json to chart: {e}"))?
    )
}

pub fn load_chart<T>(path: T) -> anyhow::Result<Chart>
where
    T: AsRef<Path>,
{
    info!("Reading chart: {:?}", path.as_ref());
    let chart_json = fs::read_to_string(&path)
        .inspect_err(|e| error!("Error reading chart: {e}"))?;
    Ok(json_to_chart(&chart_json)
        .inspect_err(|e| error!("Error converting the json to chart: {e}"))?)
}

pub fn load_all_songs<T>(root_dir: T) -> anyhow::Result<Vec<Song>>
where
    T: AsRef<Path>,
{
    let mut songs = Vec::new();

    info!("Reading root dir: {:?}", root_dir.as_ref());
    let entries = fs::read_dir(&root_dir)
        .inspect_err(|e| error!("Error reading root dir: {e}"))?;

    for entry in entries {
        let entry = entry
            .inspect_err(|e| error!("Error reading entry: {e}"))?;

        let path = entry.path();
        if path.is_dir() {
            match load_single_song(&path) {
                Ok(song) => songs.push(song),
                Err(e) => {
                    eprintln!("跳过无效歌曲目录 {:?}: {}", path, e);
                    warn!("Skipping invalid song dirs({path:?}): {e}");
                },
            }
        }
    }
    Ok(songs)
}

fn load_single_song(dir: &Path) -> anyhow::Result<Song> {
    let config_path = dir.join("song.json");

    info!("Reading song config file: {config_path:?}");
    let config_str = fs::read_to_string(&config_path)
        .inspect_err(|e| error!("Error reading file: {e}"))?;

    info!("Parsing config: {config_path:?}");
    let config: SongConfig = serde_json::from_str(&config_str)
        .inspect_err(|e|error!("Error parsing config: {e}"))?;

    // 2. 构建 Asset 路径 (Local 模式)
    let song_asset = SongAsset {
        audio: AssetLocation::Local(dir.join(&config.audio_file)),
    };

    let illu_asset: Option<IlluAsset> = config.illu_file.as_ref().map(|filename| {
        IlluAsset {
            illu: AssetLocation::Local(dir.join(filename)),
        }
    });

    // 3. 读取并解析所有 Chart 文件
    info!("Parsing chart files");
    if config.chart_files.is_empty() {
        let err_msg = "No charts found in the config file!";
        error!("{}", err_msg);
        bail!("{}", err_msg);
    }
    let mut charts = Vec::new();
    for c_cfg in config.chart_files {
        let chart = load_chart(dir.join(c_cfg)).inspect_err(
            |e| error!("Error parsing chart: {e}")
        )?;
        charts.push(chart);
    }

    Ok(Song {
        asset: song_asset,
        meta: config.meta,
        charts,
        illu: illu_asset
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_gen_song_config() {
        let cfg = SongConfig {
            meta: SongMeta {
                title: "Wow".into(),
                length: Duration::from_secs(200),
                artist: "Me".into(),
                bpm: 200.0,
            },
            audio_file: "song.mp3".into(),
            chart_files: vec!["charts/in.json".into()],
            illu_file: Some("test.png".into())
        };

        println!("{}", serde_json::to_string(&cfg).unwrap())
    }
}