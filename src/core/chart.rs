use anyhow::bail;
use log::error;
use crate::core::timing::{Beat, TimingMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub struct Chart {
    pub meta: ChartMeta,
    pub timing_map: TimingMap,
    pub tracks: Vec<Track>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub id: u8,
    pub notes: Vec<Note>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChartMeta {
    pub charter: String,
    pub level: u8,
    pub desc: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Note {
    Tap { beat: Beat },
    Hold { start: Beat, end: Beat },
}

impl Note {
    pub fn beat(&self) -> Beat {
        match self{
            Note::Tap{beat} => beat.clone(),
            Note::Hold {start, ..}=>start.clone()
        }
    }
}
impl Chart {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
pub fn json_to_chart(json_str: &str) -> anyhow::Result<Chart> {
    let chart: Chart = serde_json::from_str(json_str)
        .inspect_err(|e| error!("Error parsing json to chart: {e}"))?;
    if chart.timing_map.bpm_changes.is_empty() {
        bail!(
            "The chart does not have a bpm change!"
        )
    };
    Ok(chart)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::chart::{Chart, ChartMeta, Note, Track};
    use crate::core::timing::{Beat, BpmChange, Time};
    #[test]
    fn test_gen_chart() {
        let notes= vec![
            Note::Tap {
                beat: Beat(1.0)
            },
            Note::Hold {
                start: Beat(2.0),
                end: Beat(3.0),
            }
        ];
        let tracks = vec![Track {id: 0, notes}];
        let map = TimingMap {
            offset: Time(0.0),
            bpm_changes: vec![BpmChange{beat: Beat(0.0), bpm: 180.0}]
        };
        let chart = Chart {
            tracks,
            meta: ChartMeta{
                charter: String::from("SakiMidare"),
                desc: String::from("test"),
                level: 18
            },
            timing_map: map,
        };

        let json_str = &chart.to_json().unwrap();
        println!("{json_str}");
        println!("{:?}", &chart);
    }
}