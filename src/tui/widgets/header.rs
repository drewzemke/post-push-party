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

impl From<&Route> for Tab {
    fn from(route: &Route) -> Self {
        match route {
            Route::Store(_) => Self::Store,
            Route::Party => Self::Party,
            Route::Packs | Route::PackReveal => Self::Packs,
            Route::Games => Self::Games,
        }
    }
}

impl Tab {
    fn name(&self) -> &'static str {
        match self {
            Tab::Store => "Store",
            Tab::Party => "Party",
            Tab::Packs => "Packs",
            Tab::Games => "Games",
        }
    }
}

const TABS: [Tab; 4] = [Tab::Store, Tab::Party, Tab::Packs, Tab::Games];

pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    route: &Route,
    state: &State,
    game_count_offset: u32,
) {
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
    let game_token_total = state.game_token_total() - game_count_offset;
    let tabs: Vec<Span> = TABS
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let selected = *tab == Tab::from(route);
            let style = if selected {
                Style::default().fg(Color::Reset)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let name = tab.name();
            let count = if *tab == Tab::Packs && pack_total > 0 {
                format!(" ({pack_total})")
            } else if *tab == Tab::Games && game_token_total > 0 {
                format!(" ({game_token_total})")
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
