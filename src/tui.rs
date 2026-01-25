use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::state::{self, feature_cost, PartyFeature, State, PARTY_FEATURES};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Store,
    Config,
}

pub struct App {
    state: State,
    message: Option<String>,
    tab: Tab,
    store_selection: usize, // 0 = commit value, 1+ = feature index
    config_selection: usize, // index into PARTY_FEATURES
}

impl App {
    pub fn new() -> Self {
        Self {
            state: state::load(),
            message: None,
            tab: Tab::Store,
            store_selection: 0,
            config_selection: 0,
        }
    }

    fn store_item_count(&self) -> usize {
        1 + PARTY_FEATURES.len() // commit value + features
    }

    fn handle_store_action(&mut self) {
        if self.store_selection == 0 {
            self.upgrade_commit_value();
        } else {
            let feature_idx = self.store_selection - 1;
            if let Some(&feature) = PARTY_FEATURES.get(feature_idx) {
                self.unlock_feature(feature);
            }
        }
    }

    fn handle_config_action(&mut self) {
        if let Some(&feature) = PARTY_FEATURES.get(self.config_selection) {
            if self.state.is_unlocked(feature) {
                self.state.toggle_feature(feature);
                let _ = state::save(&self.state);
            }
        }
    }

    fn upgrade_commit_value(&mut self) {
        let cost = self.state.upgrade_cost();
        if self.state.party_points >= cost {
            self.state.party_points -= cost;
            self.state.commit_value_level += 1;
            let _ = state::save(&self.state);
            self.message = Some(format!(
                "Commit value now {}!",
                self.state.points_per_commit()
            ));
        } else {
            self.message = Some(format!(
                "Need {} more points",
                cost - self.state.party_points
            ));
        }
    }

    fn unlock_feature(&mut self, feature: PartyFeature) {
        if self.state.is_unlocked(feature) {
            self.message = Some(format!("{} already unlocked!", feature.name()));
            return;
        }

        let cost = feature_cost(feature);
        if self.state.party_points >= cost {
            self.state.party_points -= cost;
            self.state.unlock_feature(feature);
            let _ = state::save(&self.state);
            self.message = Some(format!("Unlocked {}!", feature.name()));
        } else {
            self.message = Some(format!(
                "Need {} more points",
                cost - self.state.party_points
            ));
        }
    }

    fn can_afford_selected(&self) -> bool {
        if self.store_selection == 0 {
            self.state.party_points >= self.state.upgrade_cost()
        } else {
            let feature_idx = self.store_selection - 1;
            PARTY_FEATURES.get(feature_idx).map_or(false, |&feature| {
                !self.state.is_unlocked(feature)
                    && self.state.party_points >= feature_cost(feature)
            })
        }
    }

    fn move_selection_up(&mut self) {
        match self.tab {
            Tab::Store => {
                let count = self.store_item_count();
                self.store_selection = (self.store_selection + count - 1) % count;
            }
            Tab::Config => {
                let count = PARTY_FEATURES.len();
                self.config_selection = (self.config_selection + count - 1) % count;
            }
        }
        self.message = None;
    }

    fn move_selection_down(&mut self) {
        match self.tab {
            Tab::Store => {
                self.store_selection = (self.store_selection + 1) % self.store_item_count();
            }
            Tab::Config => {
                self.config_selection = (self.config_selection + 1) % PARTY_FEATURES.len();
            }
        }
        self.message = None;
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('u') | KeyCode::Enter | KeyCode::Char(' ') => match app.tab {
                        Tab::Store => app.handle_store_action(),
                        Tab::Config => app.handle_config_action(),
                    },
                    KeyCode::Tab
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Char('h')
                    | KeyCode::Char('l') => {
                        app.tab = match app.tab {
                            Tab::Store => Tab::Config,
                            Tab::Config => Tab::Store,
                        };
                        app.message = None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                    _ => {}
                }
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title + tabs
            Constraint::Length(3), // points
            Constraint::Min(10),   // tab content
            Constraint::Length(2), // message
            Constraint::Length(1), // help
        ])
        .split(area);

    // title + tabs
    let tab_titles = vec!["Store", "Config"];
    let selected_tab = match app.tab {
        Tab::Store => 0,
        Tab::Config => 1,
    };
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .title(" 🎉 POST-PUSH PARTY ")
                .borders(Borders::ALL),
        )
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::White).bold());
    frame.render_widget(tabs, chunks[0]);

    // points display
    let points_text = format!(" {} party points", app.state.party_points);
    let points =
        Paragraph::new(points_text).block(Block::default().title(" Points ").borders(Borders::ALL));
    frame.render_widget(points, chunks[1]);

    // tab content
    match app.tab {
        Tab::Store => render_store(frame, app, chunks[2]),
        Tab::Config => render_config(frame, app, chunks[2]),
    }

    // message
    if let Some(msg) = &app.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg_widget, chunks[3]);
    }

    // help
    let help_text = match app.tab {
        Tab::Store if app.can_afford_selected() => "←→ tab  ↑↓ select  [u] upgrade  [q] quit",
        Tab::Store => "←→ tab  ↑↓ select  [q] quit",
        Tab::Config => "←→ tab  ↑↓ select  [space] toggle  [q] quit",
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[4]);
}

fn render_store(frame: &mut Frame, app: &App, area: Rect) {
    let mut constraints = vec![Constraint::Length(4)]; // commit value
    for _ in PARTY_FEATURES {
        constraints.push(Constraint::Length(3)); // each feature
    }
    constraints.push(Constraint::Min(0)); // spacer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // commit value upgrade
    let cv_selected = app.store_selection == 0;
    let cv_affordable = app.state.party_points >= app.state.upgrade_cost();
    let cv_style = match (cv_selected, cv_affordable) {
        (true, true) => Style::default().fg(Color::Green).bold(),
        (true, false) => Style::default().fg(Color::Yellow).bold(),
        (false, _) => Style::default().fg(Color::DarkGray),
    };
    let cv_text = format!(
        " Commit Value: {} pts/commit (lvl {})\n Next: {} points",
        app.state.points_per_commit(),
        app.state.commit_value_level,
        app.state.upgrade_cost()
    );
    let cv_block = if cv_selected {
        Block::default().title(" ▶ Upgrade ").borders(Borders::ALL)
    } else {
        Block::default().title("   Upgrade ").borders(Borders::ALL)
    };
    let commit_value = Paragraph::new(cv_text).style(cv_style).block(cv_block);
    frame.render_widget(commit_value, chunks[0]);

    // feature unlocks
    for (i, &feature) in PARTY_FEATURES.iter().enumerate() {
        let selected = app.store_selection == i + 1;
        let unlocked = app.state.is_unlocked(feature);
        let cost = feature_cost(feature);
        let affordable = app.state.party_points >= cost;

        let (text, style) = if unlocked {
            (
                format!(" {} [✓ Unlocked]", feature.name()),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            let style = match (selected, affordable) {
                (true, true) => Style::default().fg(Color::Green).bold(),
                (true, false) => Style::default().fg(Color::Yellow).bold(),
                (false, _) => Style::default().fg(Color::DarkGray),
            };
            (format!(" {} [{} points]", feature.name(), cost), style)
        };

        let block = if selected {
            Block::default().title(" ▶ Unlock ").borders(Borders::ALL)
        } else {
            Block::default().title("   Unlock ").borders(Borders::ALL)
        };
        let widget = Paragraph::new(text).style(style).block(block);
        frame.render_widget(widget, chunks[i + 1]);
    }
}

fn render_config(frame: &mut Frame, app: &App, area: Rect) {
    let mut constraints = vec![];
    for _ in PARTY_FEATURES {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(0)); // spacer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, &feature) in PARTY_FEATURES.iter().enumerate() {
        let selected = app.config_selection == i;
        let unlocked = app.state.is_unlocked(feature);
        let enabled = app.state.is_enabled(feature);

        let (status, style) = if !unlocked {
            ("🔒 locked", Style::default().fg(Color::DarkGray))
        } else if enabled {
            (
                "✓ on",
                if selected {
                    Style::default().fg(Color::Green).bold()
                } else {
                    Style::default().fg(Color::Green)
                },
            )
        } else {
            (
                "✗ off",
                if selected {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::Red)
                },
            )
        };

        let text = format!(" {} [{}]", feature.name(), status);
        let block = if selected {
            Block::default().title(" ▶ ").borders(Borders::ALL)
        } else {
            Block::default().title("   ").borders(Borders::ALL)
        };
        let widget = Paragraph::new(text).style(style).block(block);
        frame.render_widget(widget, chunks[i]);
    }
}
