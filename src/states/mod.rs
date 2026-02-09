use crate::app::AppContext;
use crate::models::{ChartMeta, Rank, SongMeta};
use ratatui::crossterm::event::{KeyEvent, KeyCode::*};
use std::time::Duration;
enum StateAction {
    None,
    Quit,
    GoToCollection,
    StartPlaying {
        song: SongMeta,
        chart: ChartMeta,
    },
    TogglePause,
    ShowResult {
        score: u32,
        rank: Rank,
    },
}

trait Stateful {
    fn handle_input(
        &mut self,
        ctx: &AppContext,
        event: KeyEvent
    ) -> StateAction;
    fn tick(&mut self, ctx: &AppContext) -> StateAction{
        StateAction::None
    }
}

pub(crate) enum State {
    Welcome(WelcomeState),
    Collection(CollectionState),
    Playing(PlayingState),
    Result(ResultState),
}

impl State {
    fn handle_input(
        &mut self,
        ctx: &AppContext,
        event: KeyEvent,
    ) -> StateAction {
        match self {
            State::Welcome(s) => s.handle_input(ctx, event),
            State::Collection(s) => s.handle_input(ctx, event),
            State::Playing(s) => s.handle_input(ctx, event),
            State::Result(s) => s.handle_input(ctx, event),
        }
    }
}
pub(crate) struct WelcomeState;
impl Stateful for WelcomeState {
    fn handle_input(
        &mut self,
        context: &AppContext,
        event: KeyEvent,
    ) -> StateAction {
        match event.code {
            Char('Q') | Esc => StateAction::Quit,
            Enter | Char(' ')=> StateAction::GoToCollection,
            _ => StateAction::None
        }
    }
}

/// Invariant:
/// - song_count > 0
/// - 0 <= cursor < song_count
pub(crate) struct CollectionState {
    cursor: usize,
}

impl CollectionState {
    pub fn new(song_count: usize) -> Option<Self> {
        if song_count == 0 {
            None
        } else {
            Some(Self { cursor: 0 })
        }
    }

    fn move_up(&mut self, song_count: usize) {
        self.cursor = self
            .cursor
            .checked_sub(1)
            .unwrap_or(song_count - 1);
    }

    fn move_down(&mut self, song_count: usize) {
        self.cursor = self
            .cursor
            .checked_add(1)
            .unwrap_or(0)
            % song_count
    }
}

impl Stateful for CollectionState {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q') | Esc => StateAction::Quit,
            Up | Left | Char('W') | Char('A') => {self.move_up(ctx.songs.len()); StateAction::None},
            Down | Right |Char('S') | Char('D') => {self.move_down(ctx.songs.len()); StateAction::None},
            _ => StateAction::None
        }
    }
}

enum PlayingPhase{
    Ready,
    Playing(Duration),
    Paused(Duration),
    Finished,
}

pub(crate) struct PlayingState{
    phase: PlayingPhase,
    song: SongMeta,
    chart: ChartMeta,
}

impl Stateful for PlayingState {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q') | Esc => StateAction::TogglePause,
            _ => todo!("Playing")
        }
    }
}

pub(crate) struct ResultState {
    score: u32,
    rank: Rank,
    song: SongMeta,
    chart: ChartMeta,
}

impl Stateful for ResultState {
    fn handle_input(&mut self, ctx: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q') | Esc => StateAction::GoToCollection,
            _ => StateAction::None,
        }
    }
}