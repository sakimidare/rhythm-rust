use ratatui::crossterm::event::KeyCode::{Char, Down, Enter, Esc, Left, Right, Up};
use ratatui::crossterm::event::KeyEvent;
use ratatui::Frame;
use crate::app::AppContext;
use crate::states::{StateAction, Stateful};
use crate::ui;

pub struct CollectionState {
    pub song_cursor: Option<usize>, // 歌曲列表光标
    pub chart_cursor: usize,        // 谱面列表光标（仅在选中歌曲后有效）
    pub is_selecting_chart: bool,   // 状态开关：是选歌还是选谱面
}

impl CollectionState {
    pub fn new(song_count: usize) -> Self {
        Self {
            song_cursor: if song_count != 0 { Some(0) } else { None },
            chart_cursor: 0,
            is_selecting_chart: false,
        }
    }

    fn move_up(&mut self, song_count: usize, chart_count: usize) {
        if self.is_selecting_chart {
            self.chart_cursor = self.chart_cursor.checked_sub(1).unwrap_or(chart_count - 1);
        } else {
            if let Some(c) = &mut self.song_cursor {
                *c = c.checked_sub(1).unwrap_or(song_count - 1);
            }
        }
    }

    fn move_down(&mut self, song_count: usize, chart_count: usize) {
        if self.is_selecting_chart {
            self.chart_cursor = self.chart_cursor.checked_add(1).unwrap_or(0) % chart_count;
        } else {
            if let Some(c) = &mut self.song_cursor {
                *c = c.checked_add(1).unwrap_or(0) % song_count
            }
        }
    }
}

impl Stateful for CollectionState {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q' | 'q') | Esc => {
                if self.is_selecting_chart {
                    self.is_selecting_chart = false; // 取消选谱，回到选歌
                    StateAction::None
                } else {
                    StateAction::GotoWelcome
                }
            }
            Up | Left | Char('W' | 'A' | 'w' | 'a') => {
                if let Some(s_cur) = self.song_cursor {
                    let s_cnt = ctx.songs.len();
                    let c_cnt = ctx.songs[s_cur].charts.len(); // 理论上来说，song_cursor 一定小于 songs.len()，报错直接unwrap得了
                    self.move_up(s_cnt, c_cnt);
                }

                StateAction::None
            }
            Down | Right | Char('S' | 'D' | 's' | 'd') => {
                if let Some(s_cur) = self.song_cursor {
                    let s_cnt = ctx.songs.len();
                    let c_cnt = ctx.songs[s_cur].charts.len(); // 理论上来说，song_cursor 一定小于 songs.len()，报错直接unwrap得了
                    self.move_down(s_cnt, c_cnt);
                }
                StateAction::None
            }
            Enter => {
                if let Some(s_idx) = self.song_cursor {
                    let song = &ctx.songs[s_idx];
                    if self.is_selecting_chart {
                        // 已经在选谱了，按 Enter 代表开始游戏
                        let chart = &song.charts[self.chart_cursor];
                        return StateAction::GoToPlaying {
                            song: song.clone(),
                            chart: chart.clone(),
                        };
                    } else if !song.charts.is_empty() {
                        // 进入选谱模式
                        self.is_selecting_chart = true;
                        self.chart_cursor = 0;
                    }
                }
                StateAction::None
            }
            _ => StateAction::None,
        }
    }

    fn draw(&self, ctx: &AppContext, f: &mut Frame) {
        ui::collection::draw_collection(self, ctx, f)
    }
}