use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::views::MessageType;

// message
pub fn render_footer(
    frame: &mut Frame,
    area: Rect,
    hints: &[(&str, &str)],
    points: u64,
    message: &Option<(MessageType, String)>,
) {
    // compute layout -- points on the right, the rest of the space for the message/hints
    let points_text = format!("{} P", points);
    let points_len = points_text.chars().count();

    let chunks = Layout::horizontal([
        Constraint::Fill(1),                   // message/hints
        Constraint::Length(1),                 // spacer
        Constraint::Length(points_len as u16), // points
    ])
    .split(area.inner(Margin::new(1, 0)));

    let content_chunk = chunks[0];
    let points_chunk = chunks[2];

    if let Some((ty, msg)) = message {
        let style = match ty {
            MessageType::Success => Style::default().green(),
            MessageType::Normal => Style::default().cyan(),
            MessageType::Error => Style::default().red(),
        };
        let msg_widget = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(msg_widget, content_chunk);
    } else {
        let hints_text: Line = hints
            .iter()
            .flat_map(|(key, desc)| {
                [
                    key.white().bold(),
                    ": ".fg(Color::Gray),
                    desc.dark_gray(),
                    "  ".into(),
                ]
            })
            .collect::<Vec<_>>()
            .into();
        frame.render_widget(hints_text, content_chunk);
    }

    let points_text = Text::from(points_text.yellow());
    frame.render_widget(points_text, points_chunk);
}
