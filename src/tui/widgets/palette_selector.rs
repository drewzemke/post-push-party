use ratatui::prelude::*;

use crate::party::{Palette, palette::ALL_PALETTES};

pub struct PaletteSelector<'a> {
    palettes: &'a [String],
    idx: usize,
    active: bool,
}

impl<'a> PaletteSelector<'a> {
    pub fn new(palettes: &'a [String], idx: usize, active: bool) -> Self {
        Self {
            palettes,
            idx,
            active,
        }
    }
}

fn palette_preview(palette: &Palette, faded: bool) -> Line<'_> {
    // try these: ▓▒░
    let c = if faded { '▒' } else { '█' };
    let colors = palette.all_ratatui();

    let mut spans: Vec<Span> = Vec::new();

    match colors.len() {
        1 => spans.push(format!("{c}{c}{c}{c}{c}{c}").fg(colors[0])),
        2 => {
            spans.push(format!("{c}{c}{c}").fg(colors[0]));
            spans.push(format!("{c}{c}{c}").fg(colors[1]));
        }
        3 => {
            spans.push(format!("{c}{c}").fg(colors[0]));
            spans.push(format!("{c}{c}").fg(colors[1]));
            spans.push(format!("{c}{c}").fg(colors[2]));
        }
        _ => {
            for n in 0..6 {
                let color = colors[n % colors.len()];
                spans.push(c.to_string().fg(color))
            }
        }
    }

    Line::from(spans)
}

impl<'a> Widget for PaletteSelector<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let palette_chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        let center_palette = self
            .palettes
            .get(self.idx)
            .and_then(|name| ALL_PALETTES.iter().find(|&&p| p.name() == name));

        let (palette_name, palette_swatch) = if let Some(palette) = center_palette {
            (palette.name(), palette_preview(palette, false))
        } else {
            ("Random", "??????".into())
        };

        let center_split = Layout::horizontal([
            Constraint::Fill(1),   // name
            Constraint::Length(1), // space
            Constraint::Length(6), // swatch
        ])
        .split(palette_chunks[1]);

        Text::from(palette_swatch)
            .alignment(Alignment::Left)
            .render(center_split[2], buf);

        if self.active {
            Text::from(palette_name)
                .alignment(Alignment::Right)
                .render(center_split[0], buf);

            let top_palette = self
                .palettes
                .get((self.idx + self.palettes.len()) % (self.palettes.len() + 1))
                .and_then(|name| ALL_PALETTES.iter().find(|&&p| p.name() == name));

            let top_swatch = if let Some(palette) = top_palette {
                palette_preview(palette, false)
            } else {
                "??????".dim().into()
            };

            let btm_palette = self
                .palettes
                .get((self.idx + 1) % (self.palettes.len() + 1))
                .and_then(|name| ALL_PALETTES.iter().find(|&&p| p.name() == name));

            let btm_swatch = if let Some(palette) = btm_palette {
                palette_preview(palette, false)
            } else {
                "??????".dim().into()
            };

            Text::from(top_swatch)
                .alignment(Alignment::Right)
                .render(palette_chunks[0], buf);

            Text::from("            ▲")
                .alignment(Alignment::Left)
                .render(palette_chunks[0], buf);

            Text::from(btm_swatch)
                .alignment(Alignment::Right)
                .render(palette_chunks[2], buf);

            Text::from("            ▼")
                .alignment(Alignment::Left)
                .render(palette_chunks[2], buf);
        }
    }
}
