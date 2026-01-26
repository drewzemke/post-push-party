use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::state::State;
use crate::tui::action::{Action, Route, StoreRoute};
use crate::tui::views::{MessageType, View, ViewResult};

const GRID_ITEMS: [(StoreRoute, &str, &str); 4] = [
    (
        StoreRoute::Upgrades,
        "Party Upgrades",
        "Make your party more fancy, set colors.",
    ),
    (
        StoreRoute::Bonuses,
        "Bonuses",
        "Unlock ways to earn more points.",
    ),
    (
        StoreRoute::Grid, // placeholder - packs not implemented yet
        "Packs",
        "Buy packs which contain upgrades, points, games.",
    ),
    (
        StoreRoute::Grid, // placeholder - games not implemented yet
        "Games",
        "Spend points to unlock more attempts at mini-games.",
    ),
];

pub struct GridView {
    selection: usize,
}

impl Default for GridView {
    fn default() -> Self {
        Self { selection: 0 }
    }
}

impl View for GridView {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &State) {
        // 2x2 grid layout
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .margin(1)
            .split(area);

        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(rows[0]);

        let bottom_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(rows[1]);

        let cells = [top_row[0], top_row[1], bottom_row[0], bottom_row[1]];

        for (i, &(_, title, desc)) in GRID_ITEMS.iter().enumerate() {
            let card = GridCell::new()
                .title(title)
                .description(vec![Line::from(desc)])
                .selected(i == self.selection);
            frame.render_widget(card, cells[i]);
        }
    }

    fn handle(&mut self, action: Action, _state: &mut State) -> ViewResult {
        match action {
            Action::Up => {
                if self.selection >= 2 {
                    self.selection -= 2;
                }
                ViewResult::Redraw
            }
            Action::Down => {
                if self.selection < 2 {
                    self.selection += 2;
                }
                ViewResult::Redraw
            }
            Action::Left => {
                if self.selection % 2 == 1 {
                    self.selection -= 1;
                }
                ViewResult::Redraw
            }
            Action::Right => {
                if self.selection % 2 == 0 {
                    self.selection += 1;
                }
                ViewResult::Redraw
            }
            Action::Select => {
                let (route, _, _) = GRID_ITEMS[self.selection];
                if route == StoreRoute::Grid {
                    ViewResult::Message(MessageType::Normal, "Coming soon...".to_string())
                } else {
                    ViewResult::Navigate(Route::Store(route))
                }
            }
            Action::Tab(i) => ViewResult::Navigate(match i {
                0 => Route::Store(Default::default()),
                1 => Route::Party,
                2 => Route::Packs,
                _ => Route::Games,
            }),
            Action::Quit => ViewResult::Exit,
            _ => ViewResult::None,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![("↑↓←→", "select"), ("enter", "open"), ("q", "quit")]
    }
}

struct GridCell<'a> {
    title: &'a str,
    description: Vec<Line<'a>>,
    selected: bool,
    border_style: Style,
}

impl<'a> GridCell<'a> {
    pub fn new() -> Self {
        Self {
            title: "<missing title>",
            description: vec![],
            selected: false,
            border_style: Style::default().fg(Color::DarkGray),
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn description(mut self, content: Vec<Line<'a>>) -> Self {
        self.description = content;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        if selected {
            self.border_style = Style::default().fg(Color::Cyan);
        }
        self
    }
}

impl<'a> Default for GridCell<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GridCell<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        // center the content.
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1), // title
            Constraint::Length(1), // space
            Constraint::Length(2), // description
            Constraint::Fill(1),
        ])
        .split(inner);

        let title_area = layout[1];
        let description_area = layout[3].inner(Margin::new(1, 0));

        let title = Text::from(self.title.bold()).alignment(Alignment::Center);
        title.render(title_area, buf);

        let description = Text::from(self.description);
        Paragraph::new(description)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(description_area, buf);
    }
}
