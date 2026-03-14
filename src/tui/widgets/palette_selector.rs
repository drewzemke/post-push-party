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
            .and_then(|id| ALL_PALETTES.iter().find(|&&p| p.id() == id))
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
        let [top_line, mid_line, btm_line] = area.layout(&Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]));

        let center_palette = self.get_palette(self.idx);

        let (palette_name, palette_swatch) = if let Some(palette) = center_palette {
            (palette.name(), palette_preview(palette, false))
        } else {
            ("Random", "??????".into())
        };

        let layout = Layout::horizontal([
            Constraint::Fill(1),   // text
            Constraint::Length(1), // spacer
            Constraint::Length(6), // swatch
        ]);

        let [mid_left, _, mid_right] = mid_line.layout(&layout);

        Text::from(palette_swatch).render(mid_right, buf);

        if self.active {
            // show name of the palette
            Text::from(palette_name)
                .alignment(Alignment::Right)
                .render(mid_left, buf);

            // top/previous palette
            let top_palette =
                self.get_palette((self.idx + self.palettes.len()) % (self.palettes.len() + 1));
            let top_swatch = if let Some(palette) = top_palette {
                palette_preview(palette, true)
            } else {
                "••••••".dim().into()
            };

            let [top_left, _, top_right] = top_line.layout(&layout);

            Text::from(top_swatch).render(top_right, buf);

            Text::from("▲")
                .alignment(Alignment::Right)
                .render(top_left, buf);

            // bottom/next palette
            let btm_palette = self.get_palette((self.idx + 1) % (self.palettes.len() + 1));
            let btm_swatch = if let Some(palette) = btm_palette {
                palette_preview(palette, true)
            } else {
                "••••••".dim().into()
            };

            let [btm_left, _, btm_right] = btm_line.layout(&layout);

            Text::from(btm_swatch).render(btm_right, buf);

            Text::from("▼")
                .alignment(Alignment::Right)
                .render(btm_left, buf);
        }
    }
}
