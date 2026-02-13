use crate::app::AppContext;
use crate::states::collection::CollectionState;
use ratatui::{prelude::*, widgets::*};

pub fn draw_collection(state: &CollectionState, ctx: &AppContext, f: &mut Frame) {
    let area = f.area();

    // 增加底部操作提示条
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题
            Constraint::Min(0),    // 内容
            Constraint::Length(1), // 快捷键提示
        ])
        .split(area);

    // 1. 标题栏
    let title = Paragraph::new(" SONG COLLECTION ")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, main_chunks[0]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    // 2. 左侧：歌曲列表
    if ctx.songs.is_empty() {
        render_empty_list(f, content_chunks[0]);
        render_empty_details(f, content_chunks[1]);
    } else {
        let items: Vec<ListItem> = ctx.songs.iter().enumerate().map(|(i, song)| {
            let has_charts = !song.charts.is_empty();
            let mut content = format!(" {:02} | {}", i + 1, song.meta.title);
            if !has_charts { content.push_str(" (No Charts)"); }

            let mut style = Style::default();
            if !has_charts { style = style.fg(Color::DarkGray); }

            if state.song_cursor == Some(i) {
                // 如果正在选谱面，歌曲高亮色变淡
                let bg_color = if state.is_selecting_chart { Color::Rgb(30, 30, 60) } else { Color::Blue };
                style = style.bg(bg_color).fg(Color::White);
            }
            ListItem::new(content).style(style)
        }).collect();

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(" Playlist ")
            .border_style(if !state.is_selecting_chart { Style::default().fg(Color::Yellow) } else { Style::default() });

        f.render_widget(List::new(items).block(list_block), content_chunks[0]);

        // 3. 右侧：详情与谱面选择
        if let Some(idx) = state.song_cursor {
            render_song_details(&ctx.songs[idx], f, content_chunks[1], state);
        }
    }

    // 4. 底部提示条
    render_hint_bar(state, f, main_chunks[2]);
}
fn render_song_details(song: &crate::models::Song, f: &mut Frame, area: Rect, state: &CollectionState) {
    let border_color = if state.is_selecting_chart { Color::Yellow } else { Color::White };
    let details_block = Block::default()
        .borders(Borders::ALL)
        .title(" Song Information ")
        .border_style(Style::default().fg(border_color))
        .padding(Padding::uniform(1));

    let inner_area = details_block.inner(area);
    f.render_widget(details_block, area);

    // 拆分详情区：上部文字，下部谱面列表
    let detail_chunks = Layout::default()
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(inner_area);

    let info_text = vec![
        Line::from(vec![
            Span::styled("Title:  ", Style::default().fg(Color::Gray)),
            Span::styled(&song.meta.title, Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Artist: ", Style::default().fg(Color::Gray)),
            Span::styled(&song.meta.artist, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("BPM:    ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1}", song.meta.bpm), Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(info_text), detail_chunks[0]);

    // 谱面选择区
    if song.charts.is_empty() {
        f.render_widget(
            Paragraph::new("! No charts available for this song !")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center),
            detail_chunks[1]
        );
    } else {
        let charts: Vec<ListItem> = song.charts.iter().enumerate().map(|(i, chart)| {
            let is_selected = state.is_selecting_chart && state.chart_cursor == i;
            let style = if is_selected {
                Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let symbol = if is_selected { "> " } else { "  " };
            ListItem::new(format!("{}[LV.{:02}] charter: {} {}", symbol, chart.meta.level, chart.meta.charter, chart.meta.desc)).style(style)
        }).collect();

        let chart_list = List::new(charts)
            .block(Block::default().title(" Select Difficulty ").borders(Borders::TOP));
        f.render_widget(chart_list, detail_chunks[1]);
    }
}
fn render_empty_details(f: &mut Frame, area: Rect) {
    let msg = Paragraph::new("Select a song to see details")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Song Information "))
        .style(Style::default().fg(Color::DarkGray));

    // 使用 Layout 将文字垂直居中
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Length(1),
            Constraint::Percentage(45),
        ])
        .split(area);

    f.render_widget(msg, vertical_center[1]);
}
fn render_hint_bar(state: &CollectionState, f: &mut Frame, area: Rect) {
    let hint = if state.is_selecting_chart {
        " [UP/DOWN] Change Chart  [ENTER] Play  [ESC/Q] Cancel "
    } else if state.song_cursor.is_some() {
        " [UP/DOWN] Select Song  [ENTER] Choose Chart  [ESC/Q] Back "
    } else {
        " [ESC] Quit "
    };

    let p = Paragraph::new(hint)
        .style(Style::default().bg(Color::Cyan).fg(Color::Black))
        .alignment(Alignment::Left);
    f.render_widget(p, area);
}

fn render_empty_list(f: &mut Frame, area: Rect) {
    let empty_msg = Paragraph::new("No songs found.\nCheck your 'assets' folder.")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Playlist "));
    f.render_widget(empty_msg, area);
}