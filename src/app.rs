use crate::audio::AudioManager;
use crate::models::Song;
use crate::states::State::{Playing, Welcome};
use crate::states::collection::CollectionState;
use crate::states::playing::{PlayingPhase, PlayingState};
use crate::states::result::ResultState;
use crate::states::welcome::WelcomeState;
use crate::states::*;
use ratatui::crossterm::event::{self, Event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;
use std::time::{Duration, Instant};
use crate::config::GlobalConfig;

pub struct App {
    is_running: bool,
    state: State,
    context: AppContext,
}

pub struct AppContext {
    pub songs: Vec<Song>,
    pub audio: AudioManager,
    pub global_config: GlobalConfig
}

impl App {
    pub fn new(songs: Vec<Song>, global_config: GlobalConfig) -> Self {
        Self {
            is_running: true,
            state: Welcome(WelcomeState),
            context: AppContext {
                songs,
                audio: AudioManager::new(),
                global_config,
            },
        }
    }
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let mut last_tick = Instant::now();
        while self.is_running {
            // 维护一个时间差
            let now = Instant::now();
            let dt = now.duration_since(last_tick);
            last_tick = now;

            if let Playing(ref mut s) = self.state {
                if s.phase == PlayingPhase::Playing {
                    let current_pos = self.context.audio.get_pos();
                    s.sync_audio_time(
                        current_pos, 
                        self.context.global_config.playing.global_offset_ms
                    );
                }
            }

            terminal.draw(|f| self.state.draw(&self.context, f))?;

            // 轮询检测是否有按键事件
            if event::poll(Duration::from_millis(self.context.global_config.poll_period))? {
                if let Event::Key(key) = event::read()? {
                    let action = self.state.handle_input(&self.context, key);
                    self.resolve_action(action);
                }
            }

            let tick_action = self.state.tick(&self.context, dt);
            self.resolve_action(tick_action);
        }
        Ok(())
    }

    fn resolve_action(&mut self, action: StateAction) {
        match action {
            StateAction::None => {}
            StateAction::Quit => {
                self.context.audio.stop();
                self.is_running = false;
            }
            StateAction::GotoWelcome => {
                self.context.audio.stop();
                self.state = Welcome(WelcomeState);
            }
            StateAction::GoToCollection => {
                self.context.audio.stop();
                self.state = State::Collection(CollectionState::new(self.context.songs.len()))
            }
            StateAction::GoToPlaying { song, chart } => {
                self.state = Playing(PlayingState::new(song, &chart, &self.context));
            }
            StateAction::StartAudio { song_asset } => {
                if let Some(path) = song_asset.audio.get_local_path() {
                    self.context.audio.play_music(path).unwrap();
                }
            }
            StateAction::TogglePause => {
                if let Playing(ref mut s) = self.state {
                    s.toggle_pause();

                    if s.is_paused() {
                        self.context.audio.pause();
                    } else {
                        self.context.audio.resume();
                    }
                }
            }
            StateAction::ShowResult { score, rank } => {
                self.context.audio.stop();
                if let Playing(ref p) = self.state {
                    self.state = State::Result(ResultState::from_playing(p, score, rank));
                }
            }
        }
    }
}
