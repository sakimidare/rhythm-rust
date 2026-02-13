use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use crate::states::welcome::WelcomeState;

pub fn draw_welcome(_state: &WelcomeState, f: &mut Frame, area: Rect) {
    // f.render_widget(Block::default().bg(Color::Rgb(10, 10, 20)), area);

    let central_area = centered_rect(60, 40, area);
    let title = vec![
        Line::from(vec![
            Span::styled(" RHYTHM ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("RUST", Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press [Enter] to Start", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("Press [Q] or [ESC] to Quit", Style::default().fg(Color::DarkGray))),
    ];

    let welcome_block = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" v0.1.0 ")
                .title_alignment(Alignment::Right)
        )
        .wrap(Wrap { trim: true });

    f.render_widget(welcome_block, central_area);
}
/// 创建一个居中的矩形区域
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}