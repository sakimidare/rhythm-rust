pub mod welcome;
pub mod collection;
pub mod playing;
pub mod result;

use crate::app::AppContext;
use crate::core::chart::Chart;
use crate::models::{Song, SongAsset};
use ratatui::crossterm::event::KeyEvent;
use ratatui::Frame;
use std::time::Duration;
use crate::rank::Rank;

pub enum StateAction {
    None,
    Quit,
    GotoWelcome,
    GoToCollection,
    GoToPlaying{
        song: Song,
        chart: Chart
    },
    StartAudio {
        song_asset: SongAsset,
    },
    TogglePause,
    ShowResult {
        score: u32,
        rank: Rank,
    },
}

trait Stateful {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction;

    fn draw(&self, ctx: &AppContext, f: &mut Frame);

    fn tick(&mut self, _ctx: &AppContext, _dt: Duration) -> StateAction {
        StateAction::None
    }
}

pub enum State {
    Welcome(welcome::WelcomeState),
    Collection(collection::CollectionState),
    Playing(playing::PlayingState),
    Result(result::ResultState),
}

impl State {
    pub fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        match self {
            State::Welcome(s) => s.handle_input(ctx, event),
            State::Collection(s) => s.handle_input(ctx, event),
            State::Playing(s) => s.handle_input(ctx, event),
            State::Result(s) => s.handle_input(ctx, event),
        }
    }

    pub fn draw(&self, ctx: &AppContext, f: &mut Frame) {
        match self {
            State::Welcome(s) => s.draw(ctx, f),
            State::Collection(s) => s.draw(ctx, f),
            State::Playing(s) => s.draw(ctx, f),
            State::Result(s) => s.draw(ctx, f),
        }
    }

    pub fn tick(&mut self, ctx: &AppContext, dt: Duration) -> StateAction {
        match self {
            State::Welcome(s) => s.tick(ctx, dt),
            State::Collection(s) => s.tick(ctx, dt),
            State::Playing(s) => s.tick(ctx, dt),
            State::Result(s) => s.tick(ctx, dt),
        }
    }
}