use crate::app::AppContext;
use crate::core::chart::{Chart, ChartMeta};
use crate::core::judge::{JudgeManager, JudgeResult};
use crate::core::timing::Time;
use crate::models::{Song, SongAsset, SongMeta};
use crate::rank::Rank;
use crate::states::{StateAction, Stateful};
use crate::ui;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode::{Char, Enter, Esc};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum PlayingPhase {
    Ready,
    Playing,
    Paused,
    Finished,
}

pub struct PlayingState {
    pub elapsed_time: Time,
    pub phase: PlayingPhase,
    pub song_meta: SongMeta,
    pub song_asset: SongAsset, // 存储 asset 引用以便触发 StartAudio
    pub chart_meta: ChartMeta,
    pub combo: u32,
    pub max_combo: u32,
    pub score: u32,
    pub max_theoretical_score: u32,
    pub perfect_count: u32,
    pub good_count: u32,
    pub miss_count: u32,
    pub manager: JudgeManager,
    pub last_judge: Option<(JudgeResult, Instant)>,
    pub key_pressed: HashMap<u8, bool>,
    pub debug_logs: Vec<String>,
    pub is_autoplay: bool
}

impl PlayingState {
    pub fn new(s: Song, c: &Chart, ctx: &AppContext) -> Self {
        let start_offset = ctx.global_config.playing.ready_seconds; // 2秒倒计时,为正
        let total_notes: usize = c.tracks.iter().map(|t| t.notes.len()).sum();
        let max_score = (total_notes * 1000) as u32;
        let mut man = JudgeManager::new(
            c.tracks.clone(),
            c.timing_map.clone(),
            ctx.global_config.playing.judge_core,
        );
        man.judges.sort_by_key(|j| j.id);
        
        let mut key_pressed = HashMap::new();
        for &track_idx in ctx.global_config.playing.keybind.values() {
            key_pressed.insert(track_idx, false);
        }

        Self {
            elapsed_time: Time(-start_offset),
            phase: PlayingPhase::Ready,
            song_meta: s.meta,
            song_asset: s.asset,
            chart_meta: c.meta.clone(),
            combo: 0,
            max_combo: 0,
            score: 0,
            max_theoretical_score: max_score,
            perfect_count: 0,
            good_count: 0,
            miss_count: 0,
            manager: man,
            last_judge: None,
            key_pressed,
            debug_logs: vec![],
            is_autoplay: ctx.global_config.playing.autoplay
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.phase {
            PlayingPhase::Playing => self.phase = PlayingPhase::Paused,
            PlayingPhase::Paused => self.phase = PlayingPhase::Playing,
            _ => {}
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.phase, PlayingPhase::Paused)
    }

    pub fn sync_audio_time(&mut self, audio_time: Duration, offset_ms: i32) {
        if self.phase == PlayingPhase::Playing {
            // 核心公式：游戏逻辑时间 = 音频硬件时间 + 偏置
            let offset_secs = offset_ms as f64 / 1000.0;
            self.elapsed_time = Time(audio_time.as_secs_f64() + offset_secs);
        }
    }

    // 辅助函数，让 UI 层获取纯秒数
    pub fn current_time(&self) -> f64 {
        self.elapsed_time.0
    }

    fn process_judge_result(&mut self, ctx: &AppContext, result: JudgeResult) {
        self.last_judge = Some((result, Instant::now()));
        match result {
            JudgeResult::Perfect(_) => {
                // ctx.audio.play_hit_effect();
                self.perfect_count += 1;
                self.score += 1000;
                self.combo += 1;
                self.max_combo = self.max_combo.max(self.combo);
            }
            JudgeResult::Good(_) => {
                // ctx.audio.play_hit_effect();
                self.good_count += 1;
                self.score += 500;
                self.combo += 1;
                self.max_combo = self.max_combo.max(self.combo);
            }
            JudgeResult::Miss => {
                self.miss_count += 1;
                self.combo = 0;
            }
        }
    }

    pub fn get_accuracy_pct(&self) -> f64 {
        if self.max_theoretical_score == 0 {
            return 0.0;
        }
        (self.score as f64 / self.max_theoretical_score as f64) * 101.0
    }

    /// 计算当前理论最高准度 (Potential Accuracy)
    /// 逻辑：(当前分数 + 剩余音符全部 Perfect 的分数) / 总分
    pub fn get_potential_accuracy_pct(&self) -> f64 {
        if self.max_theoretical_score == 0 {
            return 101.0;
        }

        // 1. 获取所有轨道中还未被判定的音符总数
        let remaining_notes: usize = self
            .manager
            .judges
            .iter()
            .map(|j| j.notes.len().saturating_sub(j.cursor))
            .sum();

        // 2. 假设剩下的全是 Perfect (每个 1000 分)
        let potential_score = self.score + (remaining_notes as u32 * 1000);

        // 3. 映射到 101.0 基准
        (potential_score as f64 / self.max_theoretical_score as f64) * 101.0
    }

    /// 获取当前分数的评价等级
    pub fn get_rank(&self) -> Rank {
        Rank::from_percentage(self.get_accuracy_pct())
    }

    /// 获取当前可能达到的最高评价等级
    pub fn get_potential_rank(&self) -> Rank {
        Rank::from_percentage(self.get_potential_accuracy_pct())
    }

    pub fn log_event(&mut self, key_code: KeyCode, kind: KeyEventKind, time: f64) {
        let log = format!("[{:.3}] {:?} {:?}", time, kind, key_code);
        self.debug_logs.push(log);
        if self.debug_logs.len() > 10 {
            self.debug_logs.remove(0);
        }
    }
}

impl Stateful for PlayingState {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        self.log_event(event.code, event.kind, self.elapsed_time.0);
        let now = self.elapsed_time;
        let is_down = match event.kind {
            KeyEventKind::Press => true,
            KeyEventKind::Release => false,
            _ => return StateAction::None,
        };

        match (self.phase, event.code) {
            (PlayingPhase::Ready, Char('q' | 'Q') | Esc) => {
                if is_down {
                    return StateAction::GoToCollection;
                }
            }

            (_, Char('q' | 'Q') | Esc) => {
                if is_down {
                    return match self.phase {
                        PlayingPhase::Playing => StateAction::TogglePause,
                        PlayingPhase::Paused => StateAction::GoToCollection,
                        _ => StateAction::None,
                    };
                }
            }

            (PlayingPhase::Ready | PlayingPhase::Paused, Enter | Char(' ')) => {
                if is_down {
                    return if self.phase == PlayingPhase::Ready {
                        // 如果玩家在倒计时按确定，可以视为“直接开始”
                        self.elapsed_time = Time(0.0);
                        self.phase = PlayingPhase::Playing;
                        StateAction::StartAudio {
                            song_asset: self.song_asset.clone(),
                        }
                    } else {
                        StateAction::TogglePause
                    };
                }
            }

            // 准备的时候也能判定
            (PlayingPhase::Ready | PlayingPhase::Playing, Char(c)) => {
                if self.is_autoplay && ctx.global_config.playing.keybind.contains_key(&c) {
                    return StateAction::None;
                }

                let track_idx = ctx.global_config.playing.keybind.get(&c).copied();

                if let Some(idx) = track_idx {
                    if is_down {
                        // 使用 entry 确保安全访问并检查状态
                        let pressed = self.key_pressed.entry(idx).or_insert(false);
                        if !*pressed {
                            *pressed = true;
                            if let Some(res) = self.manager.on_input(idx, now, true) {
                                self.process_judge_result(ctx, res);
                            }
                        }
                    } else {
                        self.key_pressed.insert(idx, false);
                        if let Some(res) = self.manager.on_input(idx, now, false) {
                            self.process_judge_result(ctx, res);
                        }
                    }
                }
            }
            _ => {}
        }
        StateAction::None
    }

    fn draw(&self, ctx: &AppContext, f: &mut Frame) {
        ui::playing::draw_playing(self, ctx, f);
    }

    fn tick(&mut self, ctx: &AppContext, dt: Duration) -> StateAction {
        match self.phase {
            PlayingPhase::Ready => {
                self.elapsed_time.0 += dt.as_secs_f64();
                // 为了平滑过渡到 Playing, 在这里要处理好 Offset
                let start_threshold = ctx.global_config.playing.global_offset_ms as f64 / 1000.0;

                if self.elapsed_time.0 >= start_threshold {
                    self.elapsed_time = Time(start_threshold);
                    self.phase = PlayingPhase::Playing;
                    return StateAction::StartAudio {
                        song_asset: self.song_asset.clone(),
                    };
                }
                StateAction::None
            }
            PlayingPhase::Playing => {
                let now = self.elapsed_time;

                // 1. 先处理 Autoplay，把产生的判定结果存入一个临时 Vec
                if self.is_autoplay {
                    let mut autoplay_results = Vec::new();

                    // 🚩 只借用 manager，不借用整个 self
                    for judge in &mut self.manager.judges {
                        if let Some(note) = judge.notes.get(judge.cursor) {
                            let note_time = self.manager.map.beat_to_time(&note.beat());

                            if now >= note_time {
                                match note {
                                    crate::core::chart::Note::Tap { .. } => {
                                        if let Some(res) = judge.on_input(now, true, &self.manager.core, &self.manager.map) {
                                            autoplay_results.push(res);
                                        }
                                    }
                                    crate::core::chart::Note::Hold { end, .. } => {
                                        let end_time = self.manager.map.beat_to_time(end);
                                        let state = judge.states[judge.cursor];

                                        if state == crate::core::judge::NoteState::Pending {
                                            judge.on_input(now, true, &self.manager.core, &self.manager.map);
                                        } else if now >= end_time {
                                            if let Some(res) = judge.on_input(now, false, &self.manager.core, &self.manager.map) {
                                                autoplay_results.push(res);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 2. Autoplay 处理完释放了 manager 的借用，现在可以安全调用 self 的方法了
                    for res in autoplay_results {
                        self.process_judge_result(ctx, res);
                    }
                }

                // 3. 处理正常的更新（如自动 Miss）
                // 这里你的代码原本就已经是先 update 拿结果，再循环处理，所以这部分通常是没问题的
                let updates = self.manager.update(self.elapsed_time);
                for update in updates {
                    self.process_judge_result(ctx, update.result);
                }

                // 检查音频结束
                if ctx.audio.is_finished() {
                    let remaining_misses = self.manager.clear_and_count_unjudged();
                    for _ in 0..remaining_misses {
                        self.process_judge_result(ctx, JudgeResult::Miss);
                    }

                    self.phase = PlayingPhase::Finished;
                    return StateAction::ShowResult {
                        score: self.score,
                        rank: Rank::from_percentage(self.get_accuracy_pct()),
                    };
                }
                StateAction::None
            }
            _ => StateAction::None,
        }
    }
}
