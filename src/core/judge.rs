//! ```
//! ┌────────────┐
//! │ Input Time │   ← keyboard / replay / bot
//! └─────┬──────┘
//!       │
//! ┌─────▼──────┐
//! │ NoteJudge  │   ← Which note？
//! └─────┬──────┘
//!       │
//! ┌─────▼──────┐
//! │ JudgeCore  │   ← Perfect / Good / Miss
//! └────────────┘
//! ```

use crate::core::chart::{Note, Track};
use crate::core::timing::{Time, TimingMap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JudgeResult {
    Perfect(Time),
    Good(Time),
    Miss,
}

/// |delta| <= perfect  -> Perfect
/// |delta| <= good     -> Good
/// else                 Miss
#[derive(Debug, Clone, Copy)]
pub struct JudgeWindow {
    pub perfect: Time,
    pub good: Time,
}

#[derive(Copy, Clone, Debug)]
pub struct JudgeCore {
    pub window: JudgeWindow,
    pub hold_tolerance: Time,
}

impl JudgeCore {
    pub fn new(window: JudgeWindow, hold_tolerance: Time) -> Self {
        Self { window, hold_tolerance }
    }

    fn result_from_delta(&self, delta: Time) -> JudgeResult {
        let abs = delta.abs();

        match abs {
            d if d <= self.window.perfect => JudgeResult::Perfect(delta),
            d if d <= self.window.good => JudgeResult::Good(delta),
            _ => JudgeResult::Miss,
        }
    }

    fn judge(&self, input: JudgeInput) -> JudgeResult {
        let delta = input.input_time - input.note_time;
        self.result_from_delta(delta)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct JudgeInput {
    pub note_time: Time,
    pub input_time: Time,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteState {
    Pending,
    Holding(JudgeResult), // Hold 按下, 记录按下时的判定
    Releasing(JudgeResult, Time), // 防抖
    Hit,
    Missed,
}

/// Invariants:
/// - notes sorted by judge time (Tap.time / Hold.end)
/// - states.len() == notes.len()
/// - cursor points to first Pending note
pub struct NoteJudge {
    id: u8,
    pub notes: Vec<Note>,
    pub states: Vec<NoteState>,
    pub(crate) cursor: usize,
}

impl NoteJudge {
    fn new(track: Track) -> Self {
        let states = vec![NoteState::Pending; track.notes.len()];
        Self {
            id: track.id,
            notes: track.notes,
            states,
            cursor: 0,
        }
    }

    /// 自动 Miss，且自动判定无尾判 Hold
    fn update(
        &mut self,
        now: Time,
        judge: &JudgeCore,
        timing_map: &TimingMap,
    ) -> Vec<(usize, JudgeResult)> {
        let mut results = Vec::new();

        while self.cursor < self.notes.len() {
            let note = &self.notes[self.cursor];
            let state = self.states[self.cursor];

            match note {
                Note::Tap { beat, .. } => {
                    let time = timing_map.beat_to_time(beat);
                    // Tap 超时未打 -> Miss
                    if now - time > judge.window.good {
                        self.states[self.cursor] = NoteState::Missed;
                        results.push((self.cursor, JudgeResult::Miss));
                        self.cursor += 1;
                        continue;
                    }
                }
                Note::Hold { start, end, .. } => {
                    let start_time = timing_map.beat_to_time(start);
                    let end_time = timing_map.beat_to_time(end);

                    match state {
                        NoteState::Pending => {
                            if now - start_time > judge.window.good {
                                self.states[self.cursor] = NoteState::Missed;
                                results.push((self.cursor, JudgeResult::Miss));
                                self.cursor += 1;
                                continue;
                            }
                        }
                        NoteState::Holding(j) => {
                            if end_time - now < judge.window.good {
                                self.states[self.cursor] = NoteState::Hit;
                                results.push((self.cursor, j));
                                self.cursor += 1;
                                continue;
                            }
                            break; // 还在 Holding 期间
                        }
                        NoteState::Releasing(j, release_time) => {
                            // 如果距离结束很近，直接判 Hit
                            if end_time - now < judge.window.good {
                                self.states[self.cursor] = NoteState::Hit;
                                results.push((self.cursor, j));
                                self.cursor += 1;
                                continue;
                            }
                            // 如果松手时间超过了容错值，判定为 Missed
                            if now - release_time > judge.hold_tolerance {
                                self.states[self.cursor] = NoteState::Missed;
                                results.push((self.cursor, JudgeResult::Miss));
                                self.cursor += 1;
                                continue;
                            }
                            break; // 还在容错观察期内
                        }
                        _ => { break; }
                    }
                }
            }
            break;
        }
        results
    }

    /// 输入事件（同一 track）
    fn on_input(
        &mut self,
        input_time: Time,
        is_down: bool,
        judge: &JudgeCore,
        timing_map: &TimingMap,
    ) -> Option<JudgeResult> {
        // 清理过期 note
        self.update(input_time, judge, timing_map);

        if self.cursor >= self.notes.len() {
            return None;
        }

        let note = &self.notes[self.cursor];

        match note {
            // ================= TAP =================
            Note::Tap { beat, .. } => {
                if !is_down {
                    return None;
                }

                let time = timing_map.beat_to_time(beat);
                let delta = input_time - time;
                if delta.abs() <= judge.window.good {
                    let result = judge.judge(JudgeInput {
                        note_time: time,
                        input_time,
                    });
                    self.states[self.cursor] = if let JudgeResult::Miss = result {
                        NoteState::Missed
                    } else {
                        NoteState::Hit
                    };
                    self.cursor += 1;
                    return Some(result);
                }
                None
            }

            // ================= HOLD =================
            /*Note::Hold { start, .. } => {
                match self.states[self.cursor] {
                    NoteState::Pending => {
                        if !is_down {
                            return None;
                        }

                        let start_time = timing_map.beat_to_time(start);
                        let delta = input_time - start_time;
                        if delta.abs() <= judge.window.good {
                            let result = judge.judge(JudgeInput {
                                note_time: start_time,
                                input_time,
                            });
                            self.states[self.cursor] = NoteState::Holding(result);

                            return None;
                        }
                        None
                    }

                    NoteState::Holding(_) => {
                        if is_down {
                            return None;
                        } // 持续按住，无事件发生
                        let start_time = timing_map.beat_to_time(start);
                        // 玩家松手了，进行结尾判定
                        let result = judge.judge(JudgeInput {
                            note_time: start_time,
                            input_time,
                        });

                        // 无论判定如何，松手就意味着这个 Hold 结束了
                        self.states[self.cursor] = match result {
                            JudgeResult::Miss => NoteState::Missed,
                            _ => NoteState::Hit,
                        };
                        self.cursor += 1;
                        Some(result)
                    }
                    _ => None,
                }
            }*/

            Note::Hold { start, .. } => {
                match self.states[self.cursor] {
                    NoteState::Pending => {
                        if !is_down { return None; }
                        let start_time = timing_map.beat_to_time(start);
                        if (input_time - start_time).abs() <= judge.window.good {
                            let result = judge.judge(JudgeInput { note_time: start_time, input_time });
                            self.states[self.cursor] = NoteState::Holding(result);
                        }
                        None
                    }
                    NoteState::Holding(res) => {
                        if is_down { return None; }
                        // 进入观察期
                        self.states[self.cursor] = NoteState::Releasing(res, input_time);
                        None
                    }
                    NoteState::Releasing(res, _) => {
                        if !is_down { return None; }
                        // 容错期内重新按下，恢复 Holding
                        self.states[self.cursor] = NoteState::Holding(res);
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}

pub struct JudgeManager {
    pub judges: Vec<NoteJudge>,
    core: JudgeCore,
    pub map: TimingMap,
}

pub struct UpdateResult {
    pub track_idx: usize,
    pub note_idx: usize,
    pub result: JudgeResult,
}
impl JudgeManager {
    pub fn new(tracks: Vec<Track>, timing_map: TimingMap, core: JudgeCore) -> Self {
        let mut result = vec![];
        for track in tracks {
            result.push(NoteJudge::new(track))
        }
        Self {
            judges: result,
            core,
            map: timing_map,
        }
    }
    pub fn on_input(&mut self, track: u8, time: Time, is_down: bool) -> Option<JudgeResult> {
        self.judges
            .iter_mut()
            .find(|nj| nj.id == track)?
            .on_input(time, is_down, &self.core, &self.map)
    }

    pub fn update(&mut self, now: Time) -> Vec<UpdateResult> {
        let mut all_results = Vec::new();
        for (track_idx, judge) in self.judges.iter_mut().enumerate() {
            let results = judge.update(now, &self.core, &self.map);
            for (note_idx, res) in results {
                // 返回 (轨道号, Note编号, 结果)
                all_results.push(UpdateResult {
                    track_idx,
                    note_idx,
                    result: res,
                });
            }
        }
        all_results
    }

    pub fn clear_and_count_unjudged(&mut self) -> u32 {
        let mut total_unjudged = 0;

        for nj in &mut self.judges {
            // 1. 计算当前轨道还没判定的音符数量
            let remaining = nj.notes.len().saturating_sub(nj.cursor);
            total_unjudged += remaining as u32;

            // 2. 将这些音符的状态全部强转为 Missed (防止 UI 渲染出错)
            for i in nj.cursor..nj.notes.len() {
                nj.states[i] = NoteState::Missed;
            }

            // 3. 将游标推到最后，标记该轨道已清空
            nj.cursor = nj.notes.len();
        }

        total_unjudged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::chart::Note;
    use crate::core::judge::JudgeResult::Perfect;
    use crate::core::timing::{Beat, BpmChange};

    // 辅助函数：快速创建测试环境
    fn setup_test(notes: Vec<Note>) -> (NoteJudge, JudgeCore, TimingMap) {
        let core = JudgeCore::new(JudgeWindow {
            perfect: Time(0.03),
            good: Time(0.08),
        }, Time(0.008));
        let map = TimingMap {
            offset: Time(0.0),
            bpm_changes: vec![BpmChange{beat: Beat(0.0), bpm: 60.0}],
        };
        (NoteJudge::new(Track { id: 0, notes }), core, map)
    }

    #[test]
    fn test_hold_miss_at_start() {
        let notes = vec![Note::Hold {
            start: Beat(1.0),
            end: Beat(2.0),
        }];
        let (mut nj, core, map) = setup_test(notes);

        // 时间还没到，依然是 Pending
        nj.update(Time(0.5), &core, &map);
        assert_eq!(nj.states[0], NoteState::Pending);

        // 时间超过 start + miss_window (1.0 + 0.15 = 1.15)
        let missed = nj.update(Time(1.16), &core, &map);
        assert_eq!(missed.len(), 1);
        assert_eq!(nj.states[0], NoteState::Missed);
        assert_eq!(nj.cursor, 1);
    }

    #[test]
    fn test_hold_success_flow() {
        let notes = vec![Note::Hold {
            start: Beat(1.0),
            end: Beat(2.0),
        }];
        let (mut nj, core, map) = setup_test(notes);

        // 1. 在开头按下 (Down)
        nj.on_input(Time(1.01), true, &core, &map);
        assert!(matches!(nj.states[0], NoteState::Holding(_)));

        // 2. 时间经过 end，没松手也应该判 Perfect
        let result = nj.update(Time(2.2), &core, &map);
        assert!(!result.is_empty());

        if let Perfect(d) = result[0].1 {
            assert!((d.0 - 0.01).abs() < 1e-6);
        } else {
            panic!()
        }
        assert_eq!(nj.states[0], NoteState::Hit);
        assert_eq!(nj.cursor, 1);
    }

    #[test]
    fn test_hold_release_with_delta_early() {
        let notes = vec![Note::Hold {
            start: Beat(1.0),
            end: Beat(2.0),
        }];

        let (mut nj, core, map) = setup_test(notes);
        nj.on_input(Time(1.01), true, &core, &map);
        assert!(matches!(nj.states[0], NoteState::Holding(_)));

        // 2. 时间到达 end，稍微松晚了一点点，落在good窗口里
        let result = nj.update(Time(1.95), &core, &map);
        // update
        assert!(!result.is_empty());

        if let Perfect(d) = result[0].1 {
            assert!((d.0 - 0.01).abs() < 1e-6);
        } else {
            panic!()
        }
        assert_eq!(nj.states[0], NoteState::Hit);
        assert_eq!(nj.cursor, 1);
    }

    #[test]
    fn test_hold_early_release_miss() {
        let notes = vec![Note::Hold {
            start: Beat(1.0),
            end: Beat(2.0),
        }];
        let (mut nj, core, map) = setup_test(notes);

        // 按下开头
        nj.on_input(Time(1.0), true, &core, &map);

        // 太早松开 (在 2.0 的 good 窗口 0.08s 之外)
        let _res = nj.on_input(Time(1.5), false, &core, &map);

        // 松手时间在 good 窗口外无效 -> 判定为 Miss
        assert_eq!(nj.states[0], NoteState::Missed);
        assert_eq!(nj.cursor, 1);
    }

    #[test]
    fn test_hold_infinite_press_no_miss() {
        let notes = vec![Note::Hold {
            start: Beat(1.0),
            end: Beat(2.0),
        }];
        let (mut nj, core, map) = setup_test(notes);

        nj.on_input(Time(1.0), true, &core, &map);

        // 哪怕到了 10 秒后，不松手，状态也要更新成 Hit
        nj.update(Time(10.0), &core, &map);
        assert_eq!(nj.states[0], NoteState::Hit);
        assert_eq!(nj.cursor, 1); // Note 被击打，不卡游标
    }
}
