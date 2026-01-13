use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::state::{self, State};

#[derive(Clone, Copy, PartialEq)]
enum Selection {
    CommitValue,
    PartyLevel,
}

pub struct App {
    state: State,
    message: Option<String>,
    selection: Selection,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: state::load(),
            message: None,
            selection: Selection::CommitValue,
        }
    }

    fn try_upgrade(&mut self) {
        match self.selection {
            Selection::CommitValue => self.upgrade_commit_value(),
            Selection::PartyLevel => self.upgrade_party_level(),
        }
    }

    fn upgrade_commit_value(&mut self) {
        let cost = self.state.upgrade_cost();
        if self.state.party_points >= cost {
            self.state.party_points -= cost;
            self.state.commit_value_level += 1;
            let _ = state::save(&self.state);
            self.message = Some(format!("Commit value now {}!", self.state.points_per_commit()));
        } else {
            self.message = Some(format!("Need {} more points", cost - self.state.party_points));
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
                self.message = Some(format!("Need {} more points", cost - self.state.party_points));
            }
        } else {
            self.message = Some("Party maxed out!".to_string());
        }
    }

    fn can_afford_selected(&self) -> bool {
        match self.selection {
            Selection::CommitValue => self.state.party_points >= self.state.upgrade_cost(),
            Selection::PartyLevel => self
                .state
                .party_upgrade_cost()
                .map(|c| self.state.party_points >= c)
                .unwrap_or(false),
        }
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
                    KeyCode::Char('u') | KeyCode::Enter => app.try_upgrade(),
                    KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') | KeyCode::Char('k') => {
                        app.selection = match app.selection {
                            Selection::CommitValue => Selection::PartyLevel,
                            Selection::PartyLevel => Selection::CommitValue,
                        };
                        app.message = None;
                    }
                    KeyCode::Char('1') => {
                        app.selection = Selection::CommitValue;
                        app.message = None;
                    }
                    KeyCode::Char('2') => {
                        app.selection = Selection::PartyLevel;
                        app.message = None;
                    }
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
            Constraint::Length(3), // title
            Constraint::Length(3), // points
            Constraint::Length(4), // commit value
            Constraint::Length(4), // party level
            Constraint::Length(2), // message
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // help
        ])
        .split(area);

    // title
    let title = Paragraph::new("🎉 POST-PUSH PARTY 🎉")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // points display
    let points_text = format!(" {} party points", app.state.party_points);
    let points = Paragraph::new(points_text)
        .block(Block::default().title(" Points ").borders(Borders::ALL));
    frame.render_widget(points, chunks[1]);

    // commit value upgrade
    let cv_selected = app.selection == Selection::CommitValue;
    let cv_affordable = app.state.party_points >= app.state.upgrade_cost();
    let cv_style = match (cv_selected, cv_affordable) {
        (true, true) => Style::default().fg(Color::Green).bold(),
        (true, false) => Style::default().fg(Color::Yellow).bold(),
        (false, _) => Style::default().fg(Color::DarkGray),
    };
    let cv_text = format!(
        " [1] Commit Value: {} pts/commit (lvl {})\n     Next: {} points",
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
    frame.render_widget(commit_value, chunks[2]);

    // party level upgrade
    let pl_selected = app.selection == Selection::PartyLevel;
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
            " [2] Party Style: {}\n     Next: {} ({} points)",
            app.state.party_level_name(),
            next.name,
            next.cost
        )
    } else {
        format!(
            " [2] Party Style: {} (MAX)",
            app.state.party_level_name()
        )
    };
    let pl_block = if pl_selected {
        Block::default().title(" ▶ Upgrade ").borders(Borders::ALL)
    } else {
        Block::default().title("   Upgrade ").borders(Borders::ALL)
    };
    let party_level = Paragraph::new(pl_text).style(pl_style).block(pl_block);
    frame.render_widget(party_level, chunks[3]);

    // message
    if let Some(msg) = &app.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg_widget, chunks[4]);
    }

    // help
    let help_text = if app.can_afford_selected() {
        "↑↓ select  [u] upgrade  [q] quit"
    } else {
        "↑↓ select  [q] quit"
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[6]);
}
