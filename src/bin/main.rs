use mug_tui::load;
use mug_tui::app::App;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::{io, panic};
use std::fs::File;
use log::error;
use ratatui::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};

pub fn set_panic_hook() {
    let original_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // 1. 尝试恢复终端状态
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        error!("{panic_info}");
        // 2. 调用原始的 Hook（打印错误信息到标准错误输出）
        original_hook(panic_info);
    }));
}

fn main() -> anyhow::Result<()> {
    CombinedLogger::init(
        vec![
            // 写入到项目根目录下的 game.log
            WriteLogger::new(
                LevelFilter::Debug, // 设置日志等级 (Trace < Debug < Info < Warn < Error)
                ConfigBuilder::new()
                    .set_time_format_rfc3339()
                    .set_time_offset_to_local()
                    .unwrap()
                    .build(),
                File::create("game.log")?,
            ),
        ]
    )?;
    set_panic_hook();
    let songs = load::load_all_songs("./assets")?;

    if songs.is_empty() {
         anyhow::bail!("没有找到任何歌曲！请检查 assets 目录。");
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
    stdout,
    EnterAlternateScreen,
    // 开启增强协议以获取 Release 和 Repeat 事件
    PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES
    )
)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(songs, 800);
    app.run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
