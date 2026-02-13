use ratatui::crossterm::event::KeyCode::{Char, Enter, Esc};
use ratatui::crossterm::event::KeyEvent;
use ratatui::Frame;
use crate::app::AppContext;
use crate::states::{StateAction, Stateful};
use crate::ui;

pub struct WelcomeState;
impl Stateful for WelcomeState {
    fn handle_input(&mut self, _context: &AppContext, event: KeyEvent) -> StateAction {
        match event.code {
            Char('Q' | 'q') | Esc => StateAction::Quit,
            Enter | Char(' ' | '\n') => StateAction::GoToCollection,
            _ => StateAction::None,
        }
    }

    fn draw(&self, _ctx: &AppContext, f: &mut Frame) {
        ui::welcome::draw_welcome(self, f, f.area())
    }
}