use std::time::Duration;

use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
};

use crate::tui::Terminal;

pub struct Transition {
    buffer: Buffer,
}

impl Transition {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    pub fn transition_to(&mut self, color: (u8, u8, u8), terminal: &mut Terminal) -> Result<()> {
        let color = Color::Rgb(color.0, color.1, color.2);
        let style = Style::default().bg(color);

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        for pos in (0..width).step_by(2).chain([width]) {
            terminal.draw(|f| {
                let buffer = f.buffer_mut();

                // draw the target color to the left of `pos`
                for x in 0..pos {
                    for y in 0..height {
                        if let Some(cell) = buffer.cell_mut((x, y)) {
                            cell.reset();
                            cell.set_style(style);
                        }
                    }
                }

                // copy from the original buffer to the right of `pos`
                for x in pos..width {
                    for y in 0..height {
                        if let Some(cell) = buffer.cell_mut((x, y))
                            && let Some(original) = self.buffer.cell((x, y))
                        {
                            *cell = original.clone();
                        }
                    }
                }
            })?;

            std::thread::sleep(Duration::from_millis(10))
        }

        Ok(())
    }

    pub fn transition_from(&self, color: (u8, u8, u8), terminal: &mut Terminal) -> Result<()> {
        let color = Color::Rgb(color.0, color.1, color.2);
        let style = Style::default().bg(color);

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        for pos in (0..width).step_by(2).chain([width]) {
            terminal.draw(|f| {
                let buffer = f.buffer_mut();

                // copy from the original buffer to the left of `pos`
                for x in 0..pos {
                    for y in 0..height {
                        if let Some(cell) = buffer.cell_mut((x, y))
                            && let Some(original) = self.buffer.cell((x, y))
                        {
                            *cell = original.clone();
                        }
                    }
                }

                // draw the target color to the right of `pos`
                for x in pos..width {
                    for y in 0..height {
                        if let Some(cell) = buffer.cell_mut((x, y)) {
                            cell.reset();
                            cell.set_style(style);
                        }
                    }
                }
            })?;

            std::thread::sleep(Duration::from_millis(10))
        }

        Ok(())
    }
}
