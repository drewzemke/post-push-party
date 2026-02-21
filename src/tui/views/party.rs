use std::cell::Cell;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::party::{ALL_PARTIES, Party};
use crate::state::State;
use crate::tui::widgets::{PaletteSelector, ShimmerBlock};

use super::{Action, Route, View, ViewResult};

const ITEM_HEIGHT: u16 = 5;
const SCROLL_PADDING: u16 = ITEM_HEIGHT;
const PALETTE_SELECTOR_WIDTH: u16 = 20;

struct PartyItem<'a> {
    /// the party being detailed
    party: &'static dyn Party,

    /// whether this party is enabled in user's state
    enabled: bool,

    /// whether this party is selected in the UI
    selected: bool,

    /// whether or not the user is currently selecting a palette for this item
    selecting_palette: bool,

    /// the index of the selected palette among the available palettes for this party
    palette_idx: usize,

    /// the palettes for this party
    palettes: Option<&'a Vec<String>>,

    /// used for animations
    tick: u32,
}

impl<'a> PartyItem<'a> {
    fn new(
        party: &'static dyn Party,
        enabled: bool,
        selected: bool,
        selecting_palette: bool,
        palette_idx: usize,
        palettes: Option<&'a Vec<String>>,
        tick: u32,
    ) -> Self {
        Self {
            party,
            enabled,
            selected,
            selecting_palette,
            palette_idx,
            palettes,
            tick,
        }
    }
}

impl<'a> Widget for PartyItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if self.selected {
            let block = ShimmerBlock::new(self.tick);
            let inner = block.inner(area).inner(Margin::new(1, 0));
            block.render(area, buf);
            inner
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(Style::default().gray());
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        // split horizontally: details on the left and palette selection on the right
        let split = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(PALETTE_SELECTOR_WIDTH),
        ])
        .split(inner);

        //
        // details
        //

        let details_split = Layout::vertical([
            Constraint::Length(1), // name
            Constraint::Length(1), // description
            Constraint::Length(1), // status
        ])
        .split(split[0]);

        // name
        let title = Text::from(self.party.name()).reset().bold();
        title.render(details_split[0], buf);

        // description
        let desc = Text::from(self.party.description()).dark_gray();
        desc.render(details_split[1], buf);

        // enabled status
        let (status_text, status_style) = if self.enabled {
            ("✓ Enabled", Style::default().fg(Color::Green))
        } else {
            ("✗ Disabled", Style::default().fg(Color::Red))
        };

        let status = Text::from(status_text).style(status_style);
        status.render(details_split[2], buf);

        //
        // palette selection
        //
        let Some(palettes) = self.palettes else {
            return;
        };

        let widget = PaletteSelector::new(palettes, self.palette_idx, self.selecting_palette);
        widget.render(split[1], buf);
    }
}

#[derive(Default, Clone, Copy)]
enum Mode {
    #[default]
    SelectingParty,
    SelectingPalette {
        /// index of selected palette within the (sorted!)
        /// list of palettes for the current party.
        palette_idx: usize,
    },
}

#[derive(Default)]
pub struct PartyView {
    /// index selected party
    selection: usize,

    /// determines what the user is doing
    mode: Mode,

    /// manages scrolling of entire view
    scroll_state: ScrollViewState,

    /// tracks viewport_height (determined at render time but used in `handle`)
    viewport_height: Cell<u16>,
}

impl PartyView {
    fn unlocked_parties(state: &State) -> impl Iterator<Item = &'static dyn Party> + use<'_> {
        ALL_PARTIES
            .iter()
            .copied()
            .filter(|&party| state.is_party_unlocked(party.id()))
    }

    fn item_count(state: &State) -> usize {
        Self::unlocked_parties(state).count()
    }

    fn selected_party(&self, state: &State) -> Option<&'static dyn Party> {
        Self::unlocked_parties(state).nth(self.selection)
    }

    fn update_scroll(&mut self) {
        let viewport_height = self.viewport_height.get();

        let selected_top = self.selection as u16 * ITEM_HEIGHT;
        let selected_bottom = selected_top + ITEM_HEIGHT;

        let current_offset = self.scroll_state.offset().y;
        let viewport_bottom = current_offset + viewport_height;

        if selected_bottom + SCROLL_PADDING > viewport_bottom {
            let new_offset = (selected_bottom + SCROLL_PADDING).saturating_sub(viewport_height);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        } else if selected_top < current_offset + SCROLL_PADDING {
            let new_offset = selected_top.saturating_sub(SCROLL_PADDING);
            self.scroll_state.set_offset(Position::new(0, new_offset));
        }
    }

    fn palettes_for_selected_party<'b>(&'_ self, state: &'b State) -> Option<&'b Vec<String>> {
        self.selected_party(state)
            .map(|party| party.id())
            .and_then(|id| state.unlocked_palettes(id))
    }
}

impl View for PartyView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State, tick: u32) {
        self.viewport_height.set(area.height);

        let content_area = area.inner(Margin::new(1, 0));
        let content_width = content_area.width.saturating_sub(1); // leave room for scrollbar
        let content_height = Self::item_count(state) as u16 * ITEM_HEIGHT;

        let mut scroll_view = ScrollView::new(Size::new(content_width, content_height))
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never);

        for (i, party) in Self::unlocked_parties(state).enumerate() {
            let enabled = state.is_party_enabled(party.id());
            let selected = self.selection == i;

            let (selecting_palette, palette_idx) = match self.mode {
                Mode::SelectingParty => (false, state.selected_palette_idx(party.id())),
                Mode::SelectingPalette { palette_idx } if selected => (true, palette_idx),
                Mode::SelectingPalette { .. } => (false, state.selected_palette_idx(party.id())),
            };

            let item = PartyItem::new(
                party,
                enabled,
                selected,
                selecting_palette,
                palette_idx,
                state.unlocked_palettes(party.id()),
                tick,
            );
            let item_rect = Rect::new(0, i as u16 * ITEM_HEIGHT, content_width, ITEM_HEIGHT);
            scroll_view.render_widget(item, item_rect);
        }

        frame.render_stateful_widget(scroll_view, content_area, &mut self.scroll_state.clone());
    }

    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult {
        let mode = self.mode;
        match (action, mode) {
            (Action::Up, Mode::SelectingParty) => {
                let count = Self::item_count(state);
                self.selection = (self.selection + count - 1) % count;
                self.update_scroll();
                ViewResult::Redraw
            }
            (Action::Up, Mode::SelectingPalette { palette_idx }) => {
                let count = self.palettes_for_selected_party(state).map(|v| v.len());

                if let Some(count) = count {
                    let palette_idx = (palette_idx + count) % (count + 1); // add one for random option
                    self.mode = Mode::SelectingPalette { palette_idx };
                }

                ViewResult::Redraw
            }
            (Action::Down, Mode::SelectingParty) => {
                self.selection = (self.selection + 1) % Self::item_count(state);
                self.update_scroll();
                ViewResult::Redraw
            }
            (Action::Down, Mode::SelectingPalette { palette_idx }) => {
                let count = self.palettes_for_selected_party(state).map(|v| v.len());

                if let Some(count) = count {
                    let palette_idx = (palette_idx + 1) % (count + 1); // add one for random option
                    self.mode = Mode::SelectingPalette { palette_idx };
                }
                ViewResult::Redraw
            }
            (Action::Select, Mode::SelectingParty) => {
                if let Some(party) = self.selected_party(state) {
                    state.toggle_party(party.id());
                }
                ViewResult::Redraw
            }
            (Action::Select, Mode::SelectingPalette { palette_idx }) => {
                if let Some(party) = self.selected_party(state) {
                    state.set_selected_palette(party.id(), palette_idx);
                }
                self.mode = Mode::SelectingParty;
                ViewResult::Redraw
            }
            (Action::Back, Mode::SelectingPalette { .. }) => {
                self.mode = Mode::SelectingParty;
                ViewResult::Redraw
            }
            (Action::Palette, Mode::SelectingParty) => {
                if let Some(party) = self.selected_party(state) {
                    let palette_idx = state.selected_palette_idx(party.id());
                    self.mode = Mode::SelectingPalette { palette_idx };
                }
                ViewResult::Redraw
            }
            (Action::Palette, Mode::SelectingPalette { .. }) => {
                self.mode = Mode::SelectingParty;
                ViewResult::Redraw
            }
            (Action::Tab(i), _) => {
                self.mode = Mode::SelectingParty;
                ViewResult::Navigate(match i {
                    0 => Route::Store(Default::default()),
                    1 => Route::Party,
                    2 => Route::Packs,
                    _ => Route::Games,
                })
            }
            (Action::Quit, _) => ViewResult::Exit,
            _ => ViewResult::None,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        match self.mode {
            Mode::SelectingParty => vec![
                ("↑↓", "select"),
                ("enter", "toggle"),
                ("p", "change palette"),
                ("q", "quit"),
            ],
            Mode::SelectingPalette { .. } => vec![
                ("↑↓", "select"),
                ("enter", "set palette"),
                ("esc", "cancel"),
                ("q", "quit"),
            ],
        }
    }
}
