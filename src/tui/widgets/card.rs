use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

/// a bordered card with optional title and content
pub struct Card<'a> {
    title: Option<&'a str>,
    content: Vec<Line<'a>>,
    selected: bool,
    border_style: Style,
}

impl<'a> Card<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            content: vec![],
            selected: false,
            border_style: Style::default().fg(Color::DarkGray),
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn content(mut self, content: Vec<Line<'a>>) -> Self {
        self.content = content;
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

impl<'a> Default for Card<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Card<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style);

        if let Some(title) = self.title {
            block = block.title(format!(" {} ", title));
        }

        let inner = block.inner(area);
        block.render(area, buf);

        let text = Text::from(self.content);
        Paragraph::new(text).render(inner, buf);
    }
}
