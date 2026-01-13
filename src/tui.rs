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

pub struct App {
    state: State,
    message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: state::load(),
            message: None,
        }
    }

    fn try_upgrade(&mut self) {
        let cost = self.state.upgrade_cost();
        if self.state.party_points >= cost {
            self.state.party_points -= cost;
            self.state.commit_value_level += 1;
            let _ = state::save(&self.state);
            self.message = Some(format!("Upgraded to level {}!", self.state.commit_value_level));
        } else {
            self.message = Some(format!("Need {} more points", cost - self.state.party_points));
        }
    }
}

pub fn run() -> io::Result<()> {
    // setup
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    // main loop
    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('u') | KeyCode::Enter => app.try_upgrade(),
                    _ => {}
                }
            }
        }
    }

    // cleanup
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
            Constraint::Length(5), // points
            Constraint::Length(5), // upgrade
            Constraint::Length(3), // message
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
    let points_text = format!(
        "\n  {} party points",
        app.state.party_points
    );
    let points = Paragraph::new(points_text)
        .block(Block::default().title(" Points ").borders(Borders::ALL));
    frame.render_widget(points, chunks[1]);

    // upgrade info
    let can_afford = app.state.party_points >= app.state.upgrade_cost();
    let upgrade_style = if can_afford {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let upgrade_text = format!(
        "\n  Commit Value: {} (level {})\n  Next upgrade: {} points",
        app.state.points_per_commit(),
        app.state.commit_value_level,
        app.state.upgrade_cost()
    );
    let upgrade = Paragraph::new(upgrade_text)
        .style(upgrade_style)
        .block(Block::default().title(" Upgrade ").borders(Borders::ALL));
    frame.render_widget(upgrade, chunks[2]);

    // message
    if let Some(msg) = &app.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg_widget, chunks[3]);
    }

    // help
    let help_text = if can_afford {
        "[u] upgrade  [q] quit"
    } else {
        "[q] quit"
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[5]);
}
