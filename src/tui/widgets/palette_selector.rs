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

    fn get_palette(&self, idx: usize) -> Option<&Palette> {
        self.palettes
            .get(idx)
            .and_then(|name| ALL_PALETTES.iter().find(|&&p| p.name() == name))
            .map(|v| &**v)
    }
}

fn palette_preview(palette: &Palette, faded: bool) -> Line<'_> {
    // try these: ▓▒░ ■
    let c = if faded { '■' } else { '█' };
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

        let center_palette = self.get_palette(self.idx);

        let (palette_name, palette_swatch) = if let Some(palette) = center_palette {
            (palette.name(), palette_preview(palette, false))
        } else {
            ("Random", "??????".into())
        };

        let layout = Layout::horizontal([
            Constraint::Fill(1),   // text
            Constraint::Length(1), // space
            Constraint::Length(6), // swatch
        ]);

        let center_split = layout.split(palette_chunks[1]);

        Text::from(palette_swatch).render(center_split[2], buf);

        if self.active {
            // show name of the palette
            Text::from(palette_name)
                .alignment(Alignment::Right)
                .render(center_split[0], buf);

            // top/previous palette
            let top_palette =
                self.get_palette((self.idx + self.palettes.len()) % (self.palettes.len() + 1));
            let top_swatch = if let Some(palette) = top_palette {
                palette_preview(palette, true)
            } else {
                "••••••".dim().into()
            };

            let top_split = layout.split(palette_chunks[0]);

            Text::from(top_swatch).render(top_split[2], buf);

            Text::from("▲")
                .alignment(Alignment::Right)
                .render(top_split[0], buf);

            // bottom/next palette
            let btm_palette = self.get_palette((self.idx + 1) % (self.palettes.len() + 1));
            let btm_swatch = if let Some(palette) = btm_palette {
                palette_preview(palette, true)
            } else {
                "••••••".dim().into()
            };

            let btm_split = layout.split(palette_chunks[2]);

            Text::from(btm_swatch).render(btm_split[2], buf);

            Text::from("▼")
                .alignment(Alignment::Right)
                .render(btm_split[0], buf);
        }
    }
}
