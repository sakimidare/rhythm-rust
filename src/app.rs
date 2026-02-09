use crate::models::Song;
use crate::states::State;

struct App {
    is_running: bool,
    state: State,
    context: AppContext
}

pub struct AppContext {
    pub songs: Vec<Song>,
}
