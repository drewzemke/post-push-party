use ratatui::prelude::*;
use ratatui::symbols::border::THICK as BORDER;
use ratatui::widgets::Widget;

const SHIMMER_WIDTH: u32 = 18;
const CYCLE_LENGTH: u32 = 500;

pub struct ShimmerBlock {
    tick: u32,
}

impl ShimmerBlock {
    pub fn new(tick: u32) -> Self {
        Self { tick }
    }

    pub fn inner(&self, area: Rect) -> Rect {
        Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    }
}

impl Widget for ShimmerBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }

        for (x, y, s) in border_chars(area) {
            let color = shimmer_color(x, y, area, self.tick);
            buf[(x, y)].set_symbol(s).set_fg(color);
        }
    }
}

/// yields (x, y, char) for each position around the border (thick style)
fn border_chars(area: Rect) -> impl Iterator<Item = (u16, u16, &'static str)> {
    let top_left = (area.x, area.y);
    let top_right = (area.x + area.width - 1, area.y);
    let bottom_right = (area.x + area.width - 1, area.y + area.height - 1);
    let bottom_left = (area.x, area.y + area.height - 1);

    let top = (area.x..=top_right.0).map(move |x| {
        let ch = if x == top_left.0 {
            BORDER.top_left
        } else if x == top_right.0 {
            BORDER.top_right
        } else {
            BORDER.horizontal_top
        };
        (x, area.y, ch)
    });

    let right =
        ((area.y + 1)..bottom_right.1).map(move |y| (top_right.0, y, BORDER.vertical_right));

    let bottom = (bottom_left.0..=bottom_right.0).map(move |x| {
        let ch = if x == bottom_right.0 {
            BORDER.bottom_right
        } else if x == bottom_left.0 {
            BORDER.bottom_left
        } else {
            BORDER.horizontal_bottom
        };
        (x, bottom_left.1, ch)
    });

    let left = ((area.y + 1)..bottom_left.1).map(move |y| (area.x, y, BORDER.vertical_left));

    top.chain(right).chain(bottom).chain(left)
}

fn shimmer_color(x: u16, y: u16, area: Rect, tick: u32) -> Color {
    let rel_x = (x - area.x) as u32;
    let rel_y = (area.height - 1 - (y - area.y)) as u32;
    let diag = rel_x + rel_y * 2;

    // shimmer happens at end of cycle, then waits
    let max_diag = (area.width + area.height * 2) as u32;
    let shimmer_pos = (tick * 4) % CYCLE_LENGTH;
    let shimmer_start = CYCLE_LENGTH.saturating_sub(max_diag + SHIMMER_WIDTH);

    // wait during the first part of the cycle
    if shimmer_pos < shimmer_start {
        return Color::Yellow;
    }

    let dist = (shimmer_pos - shimmer_start).abs_diff(diag);

    // gold shimmer effect
    match dist {
        0..=4 => Color::White,
        _ => Color::Yellow,
    }
}
