use crate::app::AppContext;
use crate::core::chart::ChartMeta;
use crate::models::{Rank, SongMeta};
use crate::states::playing::PlayingState;
use crate::states::{StateAction, Stateful};
use crate::ui;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode::{Char, Esc};
use ratatui::crossterm::event::KeyEvent;

pub struct ResultState {
    pub score: u32,
    pub max_combo: u32,
    pub perfect_count: u32,
    pub good_count: u32,
    pub miss_count: u32,
    pub rank: Rank,
    pub accuracy: f64, // 0.0..=101.0
    pub song_meta: SongMeta,
    pub chart_meta: ChartMeta,
}

impl Stateful for ResultState {
    fn handle_input(&mut self, _ctx: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q' | 'q') | Esc => StateAction::GoToCollection,
            _ => StateAction::None,
        }
    }

    fn draw(&self, _ctx: &AppContext, f: &mut Frame) {
        ui::result::draw_result(self, f)
    }
}

impl ResultState {
    pub(crate) fn from_playing(p: &PlayingState, score: u32, rank: Rank) -> Self {
        Self {
            score,
            max_combo: p.max_combo,
            perfect_count: p.perfect_count,
            good_count: p.good_count,
            miss_count: p.miss_count,
            rank,
            accuracy: p.get_accuracy_pct(),
            song_meta: p.song_meta.clone(),
            chart_meta: p.chart_meta.clone(),
        }
    }
}
