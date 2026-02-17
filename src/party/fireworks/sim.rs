use std::f64::consts::PI;

use rand::RngExt;

const GRAVITY: f64 = -30.;
const NUM_ROCKETS: usize = 5;

const VEL_X_VARIANCE: f64 = 10.;
const VEL_Y_VARIANCE_RATIO: f64 = 0.2;

const NUM_EXPLOSION_PARTICLES: usize = 200;
const EXPLOSION_VEL: f64 = 25.;
const EXPLOSION_VEL_RANGE: std::ops::Range<f64> = 0.2 * EXPLOSION_VEL..EXPLOSION_VEL;

pub struct Particle {
    /// used for color identification
    pub color_idx: usize,

    pub x: f64,
    pub y: f64,

    pub vel_x: f64,
    pub vel_y: f64,
}

impl Particle {
    pub fn new(id: usize, x: f64, y: f64, vel_x: f64, vel_y: f64) -> Self {
        Self {
            color_idx: id,
            x,
            y,
            vel_x,
            vel_y,
        }
    }

    pub fn update(&mut self, dt_secs: f64) {
        self.vel_y += GRAVITY * dt_secs;
        self.x += self.vel_x * dt_secs;
        self.y += self.vel_y * dt_secs;
    }
}

pub struct Sim {
    /// (width, height)
    #[expect(dead_code)]
    dims: (f64, f64),

    /// rockets
    /// boolean indicates if it is still unexploded
    rockets: Vec<(Particle, bool)>,

    /// results of explosions
    particles: Vec<Particle>,
}

impl Sim {
    pub fn new(width: f64, height: f64) -> Self {
        // how high we want the particles to go before stopping
        let max_height = (2. / 3.) * height;

        // if the initial vertical velocity of a partical that starts at y=0
        // is V, then it will travel a height of
        //   H = - 0.5 * V^2 / G
        // before reaching its peak (source: calculus)

        // so we can solve for V to get:
        let vel_y_base = (-2. * max_height * GRAVITY).sqrt();

        // create rocket particles with slight variance on x- and y- velocities
        let mut rng = rand::rng();
        let rockets = (0..NUM_ROCKETS)
            .map(|n| {
                let x = (2 * n + 1) as f64 * width / (2. * NUM_ROCKETS as f64);
                let y = 0.;

                let vel_x = rng.random_range(-VEL_X_VARIANCE..VEL_X_VARIANCE);
                let vel_y = vel_y_base
                    * (1. + rng.random_range(-VEL_Y_VARIANCE_RATIO..VEL_Y_VARIANCE_RATIO));

                (Particle::new(n, x, y, vel_x, vel_y), true)
            })
            .collect();

        Self {
            dims: (width, height),
            rockets,
            particles: vec![],
        }
    }

    pub fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.rockets
            .iter()
            .filter_map(|(p, live)| live.then_some(p))
            .chain(self.particles.iter())
    }

    pub fn update(&mut self, dt_secs: f64) {
        let mut rng = rand::rng();

        for (p, live) in &mut self.rockets {
            if !*live {
                continue;
            }

            p.update(dt_secs);

            // if y-velocity is negative, the particle has reached its peak i
            // so, it's time to explode! boom!
            if p.vel_y < 0.0 {
                *live = false;

                for n in 0..NUM_EXPLOSION_PARTICLES {
                    let speed = rng.random_range(EXPLOSION_VEL_RANGE);

                    let angle = (n as f64 / NUM_EXPLOSION_PARTICLES as f64) * 2. * PI;
                    let vel_x = speed * f64::cos(angle);
                    let vel_y = speed * f64::sin(angle);

                    let new_p = Particle::new(p.color_idx, p.x, p.y, vel_x, vel_y);
                    self.particles.push(new_p);
                }
            }
        }

        for p in &mut self.particles {
            p.update(dt_secs);
        }
    }
}
