use crate::states::result::ResultState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub fn draw_result(state: &ResultState, f: &mut Frame) {
    let area = f.area();

    // 1. 整体布局：上下结构（标题+内容）
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 顶部歌曲名
            Constraint::Min(0),    // 内容区
        ])
        .split(area);

    // 渲染顶部横幅
    let header = Paragraph::new(format!(" {} - [Lv. {}] ", state.song_meta.title, state.chart_meta.level))
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(header, main_layout[0]);

    // 2. 内容布局：左右结构
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // 左侧 Rank 视觉
            Constraint::Percentage(50), // 右侧 详细数据
        ])
        .margin(1)
        .split(main_layout[1]);

    // --- 左侧：巨大 Rank 渲染 ---
    let rank_color = state.rank.to_color();
    // 1. 获取字符画原始文本
    let raw_ascii =  state.rank.to_large_ascii();

    // 2. 找到最长的一行有多少个字符
    let max_width = raw_ascii.lines().map(|l| l.len()).max().unwrap_or(0);

    // 3. 计算为了居中需要加多少空格 (区域宽度 - 字符画宽度) / 2
    let left_padding = (content_layout[0].width as usize).saturating_sub(max_width) / 2;
    let indent_str = " ".repeat(left_padding);

    // 4. 给每一行前面加上相同的缩进
    let centered_ascii = raw_ascii
        .lines()
        .map(|line| format!("{}{}", indent_str, line))
        .collect::<Vec<_>>()
        .join("\n");

    // 5. 渲染时去掉 Alignment::Center
    let rank_para = Paragraph::new(centered_ascii)
        .style(Style::default().fg(rank_color).add_modifier(Modifier::BOLD))
        .block(Block::default().padding(Padding::vertical(4)));
    f.render_widget(rank_para, content_layout[0]);

    // --- 右侧：详细统计数据 ---
    // 我们将 Perfect, Good, Miss 渲染得更像统计表
    let stats_text = vec![
        Line::from(vec![
            Span::raw(" SCORE    "),
            Span::styled(format!("{:07}", state.score), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(" ACCURACY "),
            Span::styled(format!("{:.2}%", state.accuracy), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw(" COMBO    "),
            Span::styled(format!("{}", state.max_combo), Style::default().fg(Color::Green)),
        ]),
        Line::from("-".repeat(30)).style(Style::default().fg(Color::DarkGray)),

        // 判定统计
        Line::from(vec![
            Span::styled(" PERFECT  ", Style::default().fg(Color::Cyan)),
            Span::raw(format!(" {:3}", state.perfect_count)),
        ]),
        Line::from(vec![
            Span::styled(" GOOD     ", Style::default().fg(Color::Green)),
            Span::raw(format!(" {:3}", state.good_count)),
        ]),
        Line::from(vec![
            Span::styled(" MISS     ", Style::default().fg(Color::Red)),
            Span::raw(format!(" {:3}", state.miss_count)),
        ]),

        Line::from(""),
        Line::from(" [Q/Esc] Back to Collection ").style(Style::default().add_modifier(Modifier::REVERSED)),
    ];

    f.render_widget(
        Paragraph::new(stats_text)
            .block(Block::default().padding(Padding::uniform(1))),
        content_layout[1]
    );
}
