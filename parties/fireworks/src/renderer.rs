use crate::sim::Sim;

pub struct Renderer<'a> {
    /// (rows, cols)
    rows: usize,
    cols: usize,

    colors: &'a [&'a str],
    cell_data: Vec<(u8, usize)>,
}

impl<'a> Renderer<'a> {
    pub fn new(rows: usize, cols: usize, colors: &'a [&'a str]) -> Self {
        Self {
            rows,
            cols,
            colors,
            cell_data: vec![(0, 0); rows * cols],
        }
    }

    /// returns None if nothing is left on screen
    pub fn render(&mut self, sim: &Sim) -> Option<String> {
        self.cell_data.fill((0, 0));
        let mut empty = true;

        for particle in sim.particles() {
            let x = particle.x;
            let y = self.rows as f64 * 2. - particle.y;

            if x < 0. || y < 0. || x > self.cols as f64 || y > 2. * self.rows as f64 {
                continue;
            }

            empty = false;
            let row = y as usize / 2;
            let col = x as usize;

            // the braille unicode character 0x28XX puts dots based on the bits of the
            // 'XX' bytes, according to this layout:
            //
            // 0 3
            // 1 4
            // 2 5
            // 6 7   <- annoying bottom row
            //
            // so we use the position of the particle within the cell to
            // compute which row/column of that grid it's in, then OR that bit
            // into this cell's running value
            let x_half = (x.fract() >= 0.5) as u8;
            let y_quarter = (y / 2.).fract() * 4.0;
            let bit = (y_quarter as u8) + x_half * 3;
            let bit = if y_quarter >= 3. { 6 + x_half } else { bit };

            let idx = row * self.cols + col;

            self.cell_data[idx].0 |= 1 << bit;
            self.cell_data[idx].1 = particle.color_idx;
        }

        if empty {
            return None;
        }

        let mut output = String::new();

        for (b, color_idx) in &self.cell_data {
            if *b == 0 {
                output.push(' ');
            } else {
                output.push_str(self.colors[*color_idx % self.colors.len()]);
                output.push(char::from_u32(0x2800 | *b as u32).unwrap());
            }
        }

        Some(output)
    }
}
