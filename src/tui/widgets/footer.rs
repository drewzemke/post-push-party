use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render_footer(frame: &mut Frame, area: Rect, hints: &[(&str, &str)], points: u64) {
    let hints_text: String = hints
        .iter()
        .map(|(key, desc)| format!("{} {}", key, desc))
        .collect::<Vec<_>>()
        .join("  ");

    let points_text = format!("{} P", points);

    // calculate spacing
    let available = area.width as usize;
    let hints_len = hints_text.chars().count();
    let points_len = points_text.chars().count();
    let spacing = available.saturating_sub(hints_len + points_len);

    let line = Line::from(vec![
        Span::styled(hints_text, Style::default().fg(Color::DarkGray)),
        Span::raw(" ".repeat(spacing)),
        Span::styled(points_text, Style::default().fg(Color::Green)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
