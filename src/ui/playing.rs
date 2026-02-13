use crate::app::AppContext;
use crate::core::chart::Note;
use crate::core::judge::NoteState;
use crate::states::playing::{PlayingPhase, PlayingState};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use std::time::Duration;

pub fn draw_playing(state: &PlayingState, _ctx: &AppContext, f: &mut Frame) {
    let area = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(34),
            Constraint::Fill(1),
        ])
        .split(area);

    draw_info_panel(state, f, main_chunks[0]);
    draw_play_panel(state, f, main_chunks[1]);
    draw_stats_panel(state, f, main_chunks[2]);

    draw_debug_overlay(state, f);
}

fn draw_info_panel(state: &PlayingState, f: &mut Frame, area: Rect) {
    let block = Block::default().padding(Padding::new(2, 0, 1, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // 进度显示优化：Ready 阶段显示倒计时
    let time_text = if let PlayingPhase::Ready = state.phase {
        format!("READY: {:.1}s", -state.current_time())
    } else {
        format!("TIME: {:.1}s", state.current_time())
    };

    let info = vec![
        Line::from(state.song_meta.title.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(format!("Lv.{}", state.chart_meta.level)).style(Style::default().fg(Color::Yellow)),
        Line::from(""),
        Line::from(time_text).style(Style::default().fg(Color::DarkGray)),
    ];

    f.render_widget(Paragraph::new(info), inner);
}

/// 基于 Time(f64) 的线性坐标映射
///
fn calculate_y(note_time: f64, current_time: f64, judgment_line_y: u16, speed: f64) -> i32 {
    let time_diff = note_time - current_time;
    // time_diff > 0 表示音符在未来，y 值应小于判定线（在上方）
    judgment_line_y as i32 - (time_diff * speed) as i32
}

fn draw_play_panel(state: &PlayingState, f: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" PLAYING ");
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let now = state.current_time();
    let track_width = inner_area.width / 4;
    let judgment_line_y = inner_area.bottom().saturating_sub(1); // 调整到底部
    let speed = 40.0;

    // 1. 绘制轨道背景 (当按键按下时亮起)
    for t_idx in 0..4 {
        let x = inner_area.x + (t_idx as u16 * track_width);
        if state.key_pressed[t_idx] {
            let backlight = Block::default().bg(Color::Indexed(234));
            f.render_widget(backlight, Rect::new(x, inner_area.y, track_width, inner_area.height));
        }
        // 绘制轨道分隔虚线
        /*if t_idx > 0 {
            for y in inner_area.y..inner_area.bottom() {
                f.render_widget(Paragraph::new("┊"), Rect::new(x, y, 1, 1));
            }
        }*/
    }

    // 2. 绘制判定线
    f.render_widget(
        Paragraph::new("━".repeat(inner_area.width as usize))
            .style(Style::default().fg(Color::Cyan)),
        Rect::new(inner_area.x, judgment_line_y, inner_area.width, 1)
    );

    // 3. 遍历轨道绘制音符
    for (t_idx, judge) in state.manager.judges.iter().enumerate() {
        let x = inner_area.x + (t_idx as u16 * track_width);

        for (n_idx, note) in judge.notes.iter().enumerate() {
            let note_state = judge.states[n_idx];

            // 已判定的音符不再绘制
            if !matches!(note_state, NoteState::Pending | NoteState::Holding(_)) {
                continue;
            }

            match note {
                Note::Tap { beat } => {
                    let note_time = state.manager.map.beat_to_time(beat).0;
                    let y = calculate_y(note_time, now, judgment_line_y, speed);

                    if y >= inner_area.top() as i32 && y <= judgment_line_y as i32 {
                        let style = if t_idx == 1 || t_idx == 2 { Color::Cyan } else { Color::White };
                        f.render_widget(
                            Paragraph::new("━━━━").style(Style::default().fg(style).add_modifier(Modifier::BOLD)),
                            Rect::new(x + 1, y as u16, track_width - 1, 1)
                        );
                    }
                }
                Note::Hold { start, end } => {
                    let start_time = state.manager.map.beat_to_time(start).0;
                    let end_time = state.manager.map.beat_to_time(end).0;

                    let y_start = if matches!(note_state, NoteState::Holding(_)) {
                        judgment_line_y as i32
                    } else {
                        calculate_y(start_time, now, judgment_line_y, speed)
                    };

                    let y_end = calculate_y(end_time, now, judgment_line_y, speed);

                    // 绘制长条身体
                    let draw_top = y_end.max(inner_area.top() as i32);
                    let draw_bottom = y_start.min(judgment_line_y as i32);

                    if draw_top < draw_bottom {
                        let body_style = if matches!(note_state, NoteState::Holding(_)) { Color::Yellow } else { Color::DarkGray };
                        for y_fill in draw_top..draw_bottom {
                            f.render_widget(
                                Paragraph::new("┃  ┃").style(Style::default().fg(body_style)),
                                Rect::new(x + 1, y_fill as u16, track_width - 1, 1)
                            );
                        }
                    }

                    // 绘制头部
                    if !matches!(note_state, NoteState::Holding(_)) &&
                        y_start >= inner_area.top() as i32 && y_start <= judgment_line_y as i32 {
                        f.render_widget(
                            Paragraph::new("▆▆▆▆").style(Style::default().fg(Color::Yellow)),
                            Rect::new(x + 1, y_start as u16, track_width - 1, 1)
                        );
                    }
                }
            }
        }
    }

    // 4. 判定反馈文字
    if let Some((result, time)) = state.last_judge {
        if time.elapsed() < Duration::from_millis(500) {
            let (text, color) = match result {
                crate::core::judge::JudgeResult::Perfect(_) => (" PERFECT ", Color::LightYellow),
                crate::core::judge::JudgeResult::Good(d) => {
                    if d.0 < 0.0   {("  EARLY   ", Color::LightBlue)}
                    else           {("   LATE   ", Color::LightRed)}
                },
                crate::core::judge::JudgeResult::Miss => ("  MISS   ", Color::Gray),
            };

            let judge_y = judgment_line_y.saturating_sub(4);
            f.render_widget(
                Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Rect::new(inner_area.x, judge_y, inner_area.width, 1)
            );
        }
    }
}

fn draw_stats_panel(state: &PlayingState, f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    draw_combo_panel(state, f, chunks[1]);

    f.render_widget(Paragraph::new("────────").style(Style::default().fg(Color::DarkGray)), chunks[2]);

    let rank = state.get_potential_rank();
    let stats = vec![
        Line::from(vec![
            Span::styled("ACC  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2}%", state.get_potential_accuracy_pct()), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("RANK ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", rank), Style::default().fg(rank.to_color())),
        ]),
        Line::from(vec![
            Span::styled("SCORE", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:07}", state.score), Style::default().fg(Color::Cyan)),
        ]),
    ];
    f.render_widget(Paragraph::new(stats), chunks[3]);
}

fn draw_combo_panel(state: &PlayingState, f: &mut Frame, area: Rect) {
    if state.combo == 0 { return; }

    let (base_color, modifier) = match state.combo {
        c if c >= 500 => (Color::Magenta, Modifier::BOLD | Modifier::ITALIC),
        c if c >= 100 => (Color::LightYellow, Modifier::BOLD),
        _ => (Color::Gray, Modifier::DIM),
    };

    let display_color = if let Some((result, time)) = state.last_judge {
        if !matches!(result, crate::core::judge::JudgeResult::Miss) && time.elapsed() < Duration::from_millis(100) {
            Color::White
        } else {
            base_color
        }
    } else {
        base_color
    };

    f.render_widget(
        Paragraph::new(format!("{}\nCOMBO", state.combo))
            .style(Style::default().fg(display_color).add_modifier(modifier))
            .alignment(Alignment::Left),
        area
    );
}

fn draw_debug_overlay(state: &PlayingState, f: &mut Frame) {
    // 定义一个浮动在右上角的区域
    let area = f.area();
    let debug_area = Rect {
        x: area.width.saturating_sub(40), // 靠右
        y: 1,                             // 顶部开始
        width: 38,
        height: 15,
    };

    let events: Vec<Line> = state.debug_logs.iter().rev().take(12).map(|log| {
        let color = if log.contains("Release") { Color::Red }
        else if log.contains("Repeat") { Color::DarkGray }
        else { Color::Green };
        Line::from(log.as_str()).style(Style::default().fg(color))
    }).collect();

    let block = Block::default()
        .title(" DEBUG EVENTS ")
        .borders(Borders::ALL)
        .bg(Color::Indexed(233)); // 深灰色背景，方便看清

    f.render_widget(Paragraph::new(events).block(block), debug_area);
}