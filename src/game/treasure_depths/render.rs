use std::fmt::Write as _;

use tixel::utils::write_fg_color;
use tixel::{
    Color, HalfCellCanvas,
    utils::{write_bg_color, write_move_to},
};

use super::game::{CollectibleKind, Game, SKY_HEIGHT, Tile, TileKind};

fn collectible_color(collectible: CollectibleKind) -> Color {
    match collectible {
        CollectibleKind::Silver => (170, 170, 170).into(),
        CollectibleKind::Gold => (240, 240, 10).into(),
        CollectibleKind::Diamond => (150, 250, 250).into(),
        CollectibleKind::Ruby => (240, 10, 10).into(),
        CollectibleKind::Fuel => (10, 240, 10).into(),
        CollectibleKind::Time => (50, 70, 250).into(),
    }
}

fn tile_color(tile: Tile) -> (u8, u8, u8) {
    match tile {
        Tile::Visible(tile_kind) => match tile_kind {
            TileKind::Ground { depth, noise } => {
                let depth_mod = depth as f64 / 4.;
                let noise_mod = 8. * noise;
                let r = (70. + depth_mod + noise_mod).max(30.);
                let g = (55. + depth_mod + noise_mod).max(25.);
                let b = (40. + depth_mod + noise_mod).max(20.);
                (r as u8, g as u8, b as u8)
            }
            TileKind::Sky { noise } => {
                let color_mod = (20. * noise).max(0.);
                let r = 130. + color_mod;
                let g = 200. + color_mod;
                let b = 230. + color_mod;
                (r as u8, g as u8, b as u8)
            }
            TileKind::Tunnel => (10, 10, 10),
            TileKind::Collectible { kind } => {
                let Color::Rgb(r, g, b) = collectible_color(kind) else {
                    unreachable!()
                };
                (r, g, b)
            }
            TileKind::Rock { noise } => {
                let noise_mod = 8. * noise;
                let r = (40. + noise_mod).max(30.);
                let g = (40. + noise_mod).max(25.);
                let b = (45. + noise_mod).max(20.);
                (r as u8, g as u8, b as u8)
            }
        },
        Tile::Fog { noise, .. } => {
            let noise_mod = 8. * noise;
            let r = 22. + noise_mod;
            let g = 17. + noise_mod;
            let b = 12. + noise_mod;
            (r as u8, g as u8, b as u8)
        }
    }
}

fn lerp_u8(from: u8, to: u8, t: f64) -> u8 {
    (from as f64 * (1. - t) + to as f64 * t) as u8
}

/// blends `color` toward `clear` as opacity drops from 1 (full color) to 0 (clear)
fn faded(color: (u8, u8, u8), clear: (u8, u8, u8), opacity: f64) -> Color {
    if opacity >= 1. {
        return Color::Rgb(color.0, color.1, color.2);
    }
    Color::Rgb(
        lerp_u8(clear.0, color.0, opacity),
        lerp_u8(clear.1, color.1, opacity),
        lerp_u8(clear.2, color.2, opacity),
    )
}

pub struct Renderer {
    scroll: i64,
}

impl Renderer {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    /// updates scroll position based on player position and canvas size
    fn update_scroll(&mut self, game: &Game, canvas: &HalfCellCanvas) {
        // scroll up if the player is in the top of the screen
        self.scroll = self.scroll.min(game.pos().1 - SKY_HEIGHT);

        // scroll down if the player is in the bottom of the screen
        self.scroll = self
            .scroll
            .max(game.pos().1 + SKY_HEIGHT - canvas.height() as i64);
    }

    /// renders the game world into `output`. `opacity` (0..=1) blends the whole
    /// world toward `clear` for fade transitions; `show_hud` draws the top and
    /// bottom bars (only once the player is in a run).
    #[expect(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        canvas: &mut HalfCellCanvas,
        game: &mut Game,
        output: &mut String,
        offset: (usize, usize),
        show_hud: bool,
        opacity: f64,
        clear: (u8, u8, u8),
    ) {
        self.update_scroll(game, canvas);

        let window_top_y = self.scroll;
        let window_bot_y = window_top_y + canvas.height() as i64;

        // tiles
        for y in window_top_y..window_bot_y {
            for x in 0..(canvas.width() as i64) {
                let color = tile_color(game.get_tile((x, y)));
                canvas.set_color(
                    x as usize,
                    (y - window_top_y) as usize,
                    faded(color, clear, opacity),
                );
            }
        }

        // player
        canvas.set_color(
            game.pos().0 as usize,
            (game.pos().1 - window_top_y) as usize,
            faded((250, 250, 250), clear, opacity),
        );

        output.push_str(&canvas.render());

        let top_offset = (offset.0, offset.1.saturating_sub(1));
        let bottom_offset = (offset.0, offset.1 + canvas.height() / 2);

        // hide the hud until the player is actually in a run (past the store)
        if show_hud {
            render_top_ui(output, game, top_offset, canvas.width());
            render_bottom_ui(output, game, bottom_offset, canvas.width());
        } else {
            clear_row(output, top_offset, canvas.width());
            clear_row(output, bottom_offset, canvas.width());
        }
    }
}

/// blanks out a full-width row (used to hide the hud)
fn clear_row(buf: &mut String, offset: (usize, usize), width: usize) {
    write_move_to(buf, offset.0, offset.1);
    write_bg_color(buf, (0, 0, 0).into());
    let _ = write!(buf, "{}", " ".repeat(width));
}

const TIME_START_X: usize = 28;

fn render_top_ui(buf: &mut String, game: &Game, offset: (usize, usize), width: usize) {
    write_move_to(buf, offset.0, offset.1);

    // fuel gauge
    let (fuel_gauge, fuel_gauge_len) = render_fuel_gauge(game.fuel_proportion());
    let _ = write!(buf, "{fuel_gauge}");

    // spacing between fuel and time
    let _ = write!(buf, "{}", " ".repeat(TIME_START_X - fuel_gauge_len));

    // time
    let time = game.remaining_time().as_secs();
    let mins = time / 60;
    let secs = time - 60 * mins;
    let time_len = mins.to_string().len() + 3; // time is displayed as m:ss
    write_fg_color(buf, (250, 250, 250).into());
    let _ = write!(buf, "{mins}:{secs:0>2}");

    // depth
    let depth = game.current_depth();
    let depth_str = format!("↓{depth}  ");
    let depth_len = depth.to_string().len() + 3;

    // bank
    let bank = game.bank_value();
    let bank_str = format!("{bank} P ");
    let bank_len = bank_str.len();

    // spacing between time and depth
    write_bg_color(buf, (0, 0, 0).into());
    let _ = write!(
        buf,
        "{}",
        " ".repeat(width.saturating_sub(TIME_START_X + time_len + depth_len + bank_len))
    );

    write_fg_color(buf, (230, 230, 230).into());
    let _ = write!(buf, "{depth_str}{bank_str}");
}

fn render_bottom_ui(buf: &mut String, game: &Game, offset: (usize, usize), width: usize) {
    write_move_to(buf, offset.0, offset.1);

    // message
    write_fg_color(buf, (250, 250, 250).into());
    let message = game.message().unwrap_or_default();
    let message_len = message.len() + 1;
    let _ = write!(buf, " {message}");

    // inventory
    let mut inventory = String::new();
    let mut inventory_len = 0;

    for collectible in [
        CollectibleKind::Silver,
        CollectibleKind::Gold,
        CollectibleKind::Fuel,
        CollectibleKind::Diamond,
        CollectibleKind::Ruby,
    ] {
        if let Some(c) = game.inventory().get(&collectible) {
            write_fg_color(&mut inventory, collectible_color(collectible));
            let _ = write!(&mut inventory, "■");
            write_fg_color(&mut inventory, (250, 250, 250).into());
            let _ = write!(&mut inventory, " {c} ");
            inventory_len += 3 + c.to_string().len();
        }
    }

    // capacity
    let inv_size = game.inventory_size();
    let cap = game.inventory_capacity();
    let cap_str = format!("({inv_size}/{cap}) ");
    let cap_len = cap_str.len();

    // spacing to the right of inventory
    write_bg_color(buf, (0, 0, 0).into());
    let _ = write!(
        buf,
        "{}",
        " ".repeat(width.saturating_sub(inventory_len + message_len + cap_len))
    );

    let _ = write!(buf, "{inventory}{cap_str}");
}

const FUEL_GAUGE_WIDTH: usize = 8;
const EIGHTHS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// returns (output, length)
fn render_fuel_gauge(fuel_proportion: f64) -> (String, usize) {
    let mut out = String::new();

    let color = Color::Rgb(
        (250. * (1. - fuel_proportion)) as u8,
        (250. * fuel_proportion) as u8,
        10,
    );

    write_fg_color(&mut out, color);

    if fuel_proportion <= 0. {
        write_bg_color(&mut out, (40, 40, 40).into());
        let _ = write!(&mut out, "{}", " ".repeat(FUEL_GAUGE_WIDTH));
    } else if fuel_proportion >= 1. {
        let _ = write!(&mut out, "{}", "█".repeat(FUEL_GAUGE_WIDTH));
    } else {
        let cell_width = fuel_proportion * FUEL_GAUGE_WIDTH as f64;

        let full_tiles = cell_width.floor() as usize;
        let full_tiles_str = "█".repeat(full_tiles);

        let eighth_idx = (cell_width.fract() * 8.).floor() as usize;
        let eighth = EIGHTHS[eighth_idx];

        let space = " ".repeat(FUEL_GAUGE_WIDTH.saturating_sub(full_tiles + 1));
        write_bg_color(&mut out, (40, 40, 40).into());

        let _ = write!(&mut out, "{full_tiles_str}{eighth}{space}");
    }

    write_fg_color(&mut out, (250, 250, 250).into());
    write_bg_color(&mut out, (0, 0, 0).into());
    let text = format!("{:.0}%", fuel_proportion * 100.);
    let _ = write!(&mut out, " {text}");

    (out, FUEL_GAUGE_WIDTH + text.len() + 1)
}
