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

use crate::state::{self, State};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Store,
    Config,
}

#[derive(Clone, Copy, PartialEq)]
enum StoreSelection {
    CommitValue,
    PartyLevel,
}

#[derive(Clone, Copy, PartialEq)]
enum ConfigSelection {
    Summary,
    Colorful,
    Quotes,
    BigText,
}

pub struct App {
    state: State,
    message: Option<String>,
    tab: Tab,
    store_selection: StoreSelection,
    config_selection: ConfigSelection,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: state::load(),
            message: None,
            tab: Tab::Store,
            store_selection: StoreSelection::CommitValue,
            config_selection: ConfigSelection::Summary,
        }
    }

    fn handle_store_action(&mut self) {
        match self.store_selection {
            StoreSelection::CommitValue => self.upgrade_commit_value(),
            StoreSelection::PartyLevel => self.upgrade_party_level(),
        }
    }

    fn handle_config_action(&mut self) {
        match self.config_selection {
            ConfigSelection::Summary => self.state.show_summary = !self.state.show_summary,
            ConfigSelection::Colorful => {
                if self.state.party_level >= 1 {
                    self.state.show_colorful = !self.state.show_colorful;
                }
            }
            ConfigSelection::Quotes => {
                if self.state.party_level >= 2 {
                    self.state.show_quotes = !self.state.show_quotes;
                }
            }
            ConfigSelection::BigText => {
                if self.state.party_level >= 3 {
                    self.state.show_big_text = !self.state.show_big_text;
                }
            }
        }
        let _ = state::save(&self.state);
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

    fn upgrade_party_level(&mut self) {
        if let Some(cost) = self.state.party_upgrade_cost() {
            if self.state.party_points >= cost {
                self.state.party_points -= cost;
                self.state.party_level += 1;
                let _ = state::save(&self.state);
                self.message = Some(format!("Unlocked {} party!", self.state.party_level_name()));
            } else {
                self.message = Some(format!(
                    "Need {} more points",
                    cost - self.state.party_points
                ));
            }
        } else {
            self.message = Some("Party maxed out!".to_string());
        }
    }

    fn can_afford_selected(&self) -> bool {
        match self.store_selection {
            StoreSelection::CommitValue => self.state.party_points >= self.state.upgrade_cost(),
            StoreSelection::PartyLevel => self
                .state
                .party_upgrade_cost()
                .map(|c| self.state.party_points >= c)
                .unwrap_or(false),
        }
    }

    fn move_selection_up(&mut self) {
        match self.tab {
            Tab::Store => {
                self.store_selection = match self.store_selection {
                    StoreSelection::CommitValue => StoreSelection::PartyLevel,
                    StoreSelection::PartyLevel => StoreSelection::CommitValue,
                };
            }
            Tab::Config => {
                self.config_selection = match self.config_selection {
                    ConfigSelection::Summary => ConfigSelection::BigText,
                    ConfigSelection::Colorful => ConfigSelection::Summary,
                    ConfigSelection::Quotes => ConfigSelection::Colorful,
                    ConfigSelection::BigText => ConfigSelection::Quotes,
                };
            }
        }
        self.message = None;
    }

    fn move_selection_down(&mut self) {
        match self.tab {
            Tab::Store => {
                self.store_selection = match self.store_selection {
                    StoreSelection::CommitValue => StoreSelection::PartyLevel,
                    StoreSelection::PartyLevel => StoreSelection::CommitValue,
                };
            }
            Tab::Config => {
                self.config_selection = match self.config_selection {
                    ConfigSelection::Summary => ConfigSelection::Colorful,
                    ConfigSelection::Colorful => ConfigSelection::Quotes,
                    ConfigSelection::Quotes => ConfigSelection::BigText,
                    ConfigSelection::BigText => ConfigSelection::Summary,
                };
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // commit value
            Constraint::Length(4), // party level
            Constraint::Min(0),    // spacer
        ])
        .split(area);

    // commit value upgrade
    let cv_selected = app.store_selection == StoreSelection::CommitValue;
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

    // party level upgrade
    let pl_selected = app.store_selection == StoreSelection::PartyLevel;
    let pl_affordable = app
        .state
        .party_upgrade_cost()
        .map(|c| app.state.party_points >= c)
        .unwrap_or(false);
    let pl_maxed = app.state.party_upgrade_cost().is_none();
    let pl_style = match (pl_selected, pl_affordable, pl_maxed) {
        (_, _, true) => Style::default().fg(Color::DarkGray),
        (true, true, _) => Style::default().fg(Color::Green).bold(),
        (true, false, _) => Style::default().fg(Color::Yellow).bold(),
        (false, _, _) => Style::default().fg(Color::DarkGray),
    };
    let pl_text = if let Some(next) = app.state.next_party_level() {
        format!(
            " Party Style: {}\n Next: {} ({} points)",
            app.state.party_level_name(),
            next.name,
            next.cost
        )
    } else {
        format!(" Party Style: {} (MAX)", app.state.party_level_name())
    };
    let pl_block = if pl_selected {
        Block::default().title(" ▶ Upgrade ").borders(Borders::ALL)
    } else {
        Block::default().title("   Upgrade ").borders(Borders::ALL)
    };
    let party_level = Paragraph::new(pl_text).style(pl_style).block(pl_block);
    frame.render_widget(party_level, chunks[1]);
}

fn render_config(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // summary
            Constraint::Length(3), // colorful
            Constraint::Length(3), // quotes
            Constraint::Length(3), // big text
            Constraint::Min(0),    // spacer
        ])
        .split(area);

    render_toggle(
        frame,
        app,
        chunks[0],
        ConfigSelection::Summary,
        "Point Summary",
        app.state.show_summary,
        true,
    );
    render_toggle(
        frame,
        app,
        chunks[1],
        ConfigSelection::Colorful,
        "Colorful Text",
        app.state.show_colorful,
        app.state.party_level >= 1,
    );
    render_toggle(
        frame,
        app,
        chunks[2],
        ConfigSelection::Quotes,
        "Quotes",
        app.state.show_quotes,
        app.state.party_level >= 2,
    );
    render_toggle(
        frame,
        app,
        chunks[3],
        ConfigSelection::BigText,
        "Big Text",
        app.state.show_big_text,
        app.state.party_level >= 3,
    );
}

fn render_toggle(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    item: ConfigSelection,
    name: &str,
    enabled: bool,
    unlocked: bool,
) {
    let selected = app.config_selection == item;
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
    let text = format!(" {} [{}]", name, status);
    let block = if selected {
        Block::default().title(" ▶ ").borders(Borders::ALL)
    } else {
        Block::default().title("   ").borders(Borders::ALL)
    };
    let widget = Paragraph::new(text).style(style).block(block);
    frame.render_widget(widget, area);
}
