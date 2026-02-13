use serde::Deserialize;
use crate::core::chart::{Chart, ChartMeta, Track, Note};
use crate::core::timing::{Beat, TimingMap, BpmChange, Time};

// --- Malody 原始格式定义 ---
#[derive(Deserialize)]
struct McMeta {
    #[serde(rename = "$ver")]
    ver: u32,
    creator: String,
    background: String,
    version: String,  // 谱面难度名，例如 "4K", "Hard"
    id: u32,
    mode: u32,        // 0 通常代表 Key 模式
    song: McSongInfo,
    mode_ext: McModeExtension,
}

#[derive(Deserialize)]
struct McSongInfo {
    title: String,
    artist: String,
    id: u32,
}

#[derive(Deserialize)]
struct McModeExtension {
    column: u32,
}
#[derive(Deserialize)]
struct McChart {
    meta: McMeta,
    time: Vec<McTime>,
    note: Vec<McNote>,
}

#[derive(Deserialize)]
struct McNote {
    beat: [u32; 3],
    column: Option<u8>,
    endbeat: Option<[u32; 3]>,
    sound: Option<String>,
    offset: Option<i32>,
}

#[derive(Deserialize)]
struct McTime {
    beat: [u32; 3],
    bpm: f64,
}

// --- 转换逻辑 ---
fn mc_beat_to_f64(b: [u32; 3]) -> f64 {
    b[0] as f64 + (b[1] as f64 / b[2] as f64)
}

pub fn convert_mc_to_custom(mc_json: &str) -> (Chart, crate::models::SongMeta) {
    let mc: McChart = serde_json::from_str(mc_json).expect("Failed to parse Malody chart");

    // --- 1. 提取全局偏移 (Global Offset) ---
    // Malody 的 offset 单位通常是 ms，我们需要转换为我们的 Time(秒)
    let mut global_offset_ms = 0;
    for n in &mc.note {
        if n.sound.is_some() {
            if let Some(off) = n.offset {
                global_offset_ms = off;
                break; // 通常取第一个遇到的 sound note 的 offset
            }
        }
    }

    // 2. 转换 TimingMap
    let bpm_changes = mc.time.iter().map(|t| BpmChange {
        beat: Beat(mc_beat_to_f64(t.beat)),
        bpm: t.bpm,
    }).collect();

    // 3. 初始化轨道
    let column_count = mc.meta.mode_ext.column as usize;
    let mut tracks: Vec<Track> = (0..column_count)
        .map(|i| Track { id: i as u8, notes: vec![] })
        .collect();

    // 4. 转换打击音符 (过滤掉 BGM 项)
    let mut last_beat: f64 = 0.0;
    for n in mc.note {
        // 如果没有 column，说明是 BGM 项或特殊控制项，跳过
        let col = match n.column {
            Some(c) => c as usize,
            None => continue,
        };

        if col >= column_count { continue; }

        let start = mc_beat_to_f64(n.beat);
        last_beat = last_beat.max(start);

        if let Some(eb) = n.endbeat {
            let end = mc_beat_to_f64(eb);
            last_beat = last_beat.max(end);
            tracks[col].notes.push(Note::Hold {
                start: Beat(start),
                end: Beat(end)
            });
        } else {
            tracks[col].notes.push(Note::Tap {
                beat: Beat(start)
            });
        }
    }

    // 对每个轨道排序
    for track in &mut tracks {
        track.notes.sort_by(|a, b| a.beat().0.partial_cmp(&b.beat().0).unwrap());
    }

    // 5. 构建 Chart (应用全局 Offset)
    let chart = Chart {
        meta: ChartMeta {
            charter: mc.meta.creator.clone(),
            level: 0,
            desc: format!("{}K - Converted from Malody", column_count),
        },
        timing_map: TimingMap {
            // Malody 的 offset 为负时表示音频比 0 拍更早开始
            // 我们的系统通常 offset = 秒 (ms / 1000.0)
            offset: Time(global_offset_ms as f64 / 1000.0),
            bpm_changes,
        },
        tracks,
    };

    // 6. 歌曲元数据
    let first_bpm = mc.time.get(0).map(|t| t.bpm).unwrap_or(120.0);
    // 粗略计算时长
    let estimated_secs = (last_beat * 60.0 / first_bpm) + 2.0;

    let song_meta = crate::models::SongMeta {
        title: mc.meta.song.title.clone(),
        artist: mc.meta.song.artist.clone(),
        length: std::time::Duration::from_secs(estimated_secs as u64),
        bpm: first_bpm,
    };

    (chart, song_meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_mc_to_custom() {
        let raw_mc = r#"{
            "meta": {
                "$ver": 0, "creator": "test", "background": "b.jpg", "version": "4K",
                "id": 0, "mode": 0, "time": 0,
                "song": { "title": "T", "artist": "A", "id": 0, "titleorg": "T", "artistorg": "A" },
                "mode_ext": { "column": 4, "bar_begin": 0 }
            },
            "time": [{"beat": [0, 0, 1], "bpm": 120.0}],
            "note": [{"beat": [1, 0, 1], "column": 0}]
        }"#;

        let (chart, meta) = convert_mc_to_custom(raw_mc);

        assert_eq!(chart.tracks.len(), 4);
        assert_eq!(meta.title, "T");
        // 验证第一个音符是否在第 0 轨的 1.0 拍
        if let Note::Tap { beat } = &chart.tracks[0].notes[0] {
            assert_eq!(beat.0, 1.0);
        } else {
            panic!("Note should be a Tap");
        }
    }
}