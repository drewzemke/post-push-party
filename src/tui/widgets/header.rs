use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::{state::State, tui::action::Route};

#[derive(PartialEq)]
enum Tab {
    Store,
    Party,
    Packs,
    Games,
}

impl Tab {
    fn name(&self) -> &'static str {
        match self {
            Tab::Store => "Store",
            Tab::Party => "Party",
            Tab::Packs => "Packs",
            Tab::Games => "Game",
        }
    }
}

const TABS: [Tab; 4] = [Tab::Store, Tab::Party, Tab::Packs, Tab::Games];

pub fn render_header(frame: &mut Frame, area: Rect, route: &Route, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // title bar -- shows app title and version
    let title_chunks =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(chunks[0]);

    let title = Paragraph::new("POST-PUSH PARTY 🎉").blue().bold();
    let version = Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
        .alignment(Alignment::Right)
        .dark_gray();
    frame.render_widget(title, title_chunks[0]);
    frame.render_widget(version, title_chunks[1]);

    // tabs
    let pack_total = state.pack_total();
    let selected = route.tab_index();
    let tabs: Vec<Span> = TABS
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let style = if i == selected {
                Style::default().fg(Color::Reset)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let name = tab.name();
            let count = if *tab == Tab::Packs && pack_total > 0 {
                format!(" ({pack_total})")
            } else {
                "".to_string()
            };
            let sep = if i < TABS.len() - 1 { "   " } else { "" };
            vec![
                Span::styled(format!("{name}{count}"), style),
                Span::raw(sep),
            ]
        })
        .collect();

    let tabs_line = Line::from(tabs);
    frame.render_widget(Paragraph::new(tabs_line), chunks[1]);
}
