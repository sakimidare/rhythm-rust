use std::ops::{Add, Sub};
use serde::{Deserialize, Serialize};

/// Time in Seconds
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Time(pub f64); // seconds

impl Add for Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Time {
    type Output = Time;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Time {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Beat(pub f64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimingMap {
    // 如果谱面比音乐快，请往正方向调
    pub offset: Time,                // Time at Beat(0.0)
    pub bpm_changes: Vec<BpmChange>, // sorted by beat
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BpmChange {
    pub beat: Beat, // start beat
    pub bpm: f64,
}

impl TimingMap {
    /// Convert an absolute beat to absolute time
    /// bpm_changes must be sorted by beat ascending
    pub fn beat_to_time(&self, target: &Beat) -> Time {
        let mut time = self.offset.0;

        for window in self.bpm_changes.windows(2) {
            let curr = &window[0];
            let next = &window[1];

            if target < &next.beat {
                // target is inside this BPM segment
                let delta_beat = target.0 - curr.beat.0;
                let seconds_per_beat = 60.0 / curr.bpm;
                time += delta_beat * seconds_per_beat;
                return Time(time);
            }

            // consume whole segment
            let delta_beat = next.beat.0 - curr.beat.0;
            let seconds_per_beat = 60.0 / curr.bpm;
            time += delta_beat * seconds_per_beat;
        }

        // after last bpm change
        if let Some(last) = self.bpm_changes.last() {
            let delta_beat = target.0 - last.beat.0;
            let seconds_per_beat = 60.0 / last.bpm;
            time += delta_beat * seconds_per_beat;
        }

        Time(time)
    }

    /// Convert absolute time to absolute beat
    /// Assumes bpm_changes is sorted by beat ascending
    pub fn time_to_beat(&self, target: &Time) -> Beat {
        let mut current_time = self.offset.0;

        for window in self.bpm_changes.windows(2) {
            let curr = &window[0];
            let next = &window[1];

            let seconds_per_beat = 60.0 / curr.bpm;
            let delta_beats = next.beat.0 - curr.beat.0;
            let segment_duration = delta_beats * seconds_per_beat;

            let next_time = current_time + segment_duration;

            if target.0 < next_time {
                // target is inside this BPM segment
                let delta_time = target.0 - current_time;
                let beat = curr.beat.0 + delta_time / seconds_per_beat;
                return Beat(beat);
            }

            // consume whole segment
            current_time = next_time;
        }

        // after last bpm change
        if let Some(last) = self.bpm_changes.last() {
            let seconds_per_beat = 60.0 / last.bpm;
            let delta_time = target.0 - current_time;
            let beat = last.beat.0 + delta_time / seconds_per_beat;
            return Beat(beat);
        }

        // unreachable if bpm_changes is non-empty
        Beat(0.0)
    }
}


#[test]
fn test_inverse_mapping() {
    let map = TimingMap {
        offset: Time(-0.5),
        bpm_changes: vec![
            BpmChange { beat: Beat(0.0), bpm: 120.0 },
            BpmChange { beat: Beat(4.0), bpm: 240.0 },
        ],
    };

    let beats = [ -1.0, 0.0, 2.0, 4.0, 6.0 ];

    for &b in &beats {
        let beat = Beat(b);
        let time = map.beat_to_time(&beat);
        let back = map.time_to_beat(&time);
        assert!((back.0 - b).abs() < 1e-6);
    }
}
