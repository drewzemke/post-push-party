use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use noise::{NoiseFn, Perlin};
use rand::RngExt;

#[derive(Clone, Copy)]
pub enum Input {
    Up,
    Down,
    Right,
    Left,
}

pub const SKY_HEIGHT: i64 = 12;
pub const BASE_REVEAL_RADIUS: i64 = 3;

pub const BASE_STARTING_FUEL: f64 = 1000.;
const FUEL_COLLECTIBLE_GAIN: f64 = 250.;
const MOVE_FUEL_COST: f64 = 2.;
const DIG_FUEL_COST: f64 = 5.;

const MESSAGE_DISPLAY_TIME: Duration = Duration::from_secs(3);
const LOW_FUEL_MESSAGE_THRESHOLD: f64 = 0.2;
const LOW_TIME_MESSAGE_THRESHOLD: Duration = Duration::from_secs(15);

pub const BASE_STARTING_TIME: Duration = Duration::from_secs(60);
const TIME_COLLECTIBLE_GAIN: Duration = Duration::from_secs(15);

pub const BASE_INVENTORY_CAPACITY: usize = 20;

// NOTE: the scale constants are just random decimals, no special significance
const GROUND_NOISE_INPUT_SCALE: f64 = 0.6123123;

const SKY_NOISE_INPUT_SCALE: f64 = 0.121314;
const SKY_NOISE_OFFSET: f64 = 100.3412;

const FOG_NOISE_INPUT_SCALE: f64 = 0.102934;
const FOG_NOISE_OFFSET: f64 = 30.3412;

const ROCK_Y_THRESHOLD: i64 = SKY_HEIGHT + 5;
const ROCK_NOISE_INPUT_SCALE_X: f64 = 0.252934;
const ROCK_NOISE_INPUT_SCALE_Y: f64 = 0.102934;
const ROCK_NOISE_OFFSET: f64 = 11.31412;
const ROCK_NOISE_THRESHOLD: f64 = 0.6;
const ROCK_DEPTH_MULTIPLIER: f64 = 0.0005;

const COLLECTIBLE_NOISE_INPUT_SCALE1: f64 = 0.43923;
const COLLECTIBLE_NOISE_INPUT_SCALE2: f64 = 2. * COLLECTIBLE_NOISE_INPUT_SCALE1;
const COLLECTIBLE_NOISE_OFFSET: f64 = 200.3412;
const COLLECTIBLE_THRESHOLD: f64 = 0.70;
const COLLECTIBLE_DEPTH_MULTIPLIER: f64 = 0.0005;

const COLLECTIBLE_KIND_NOISE_INPUT_SCALE: f64 = 0.23923;
const COLLECTIBLE_KIND_NOISE_OFFSET: f64 = 77.3412;
const RUBY_NOISE_THRESHOLD: f64 = 0.95;
const DIAMOND_NOISE_THRESHOLD: f64 = 0.9;
const TIME_NOISE_THRESHOLD: f64 = 0.7;
const FUEL_NOISE_THRESHOLD: f64 = 0.47;
const GOLD_NOISE_THRESHOLD: f64 = 0.0;
const RUBY_DEPTH_THRESHOLD: i64 = 500;
const DIAMOND_DEPTH_THRESHOLD: i64 = 200;
const TIME_DEPTH_THRESHOLD: i64 = 50;
const FUEL_DEPTH_THRESHOLD: i64 = 25;
const GOLD_DEPTH_THRESHOLD: i64 = 10;
const COLLECTIBLE_KIND_DEPTH_MULTIPLIER: f64 = 0.0015;

const SILVER_VALUE: u64 = 1;
const GOLD_VALUE: u64 = 5;
const DIAMOND_VALUE: u64 = 200;
const RUBY_VALUE: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectibleKind {
    Silver,
    Gold,
    Diamond,
    Ruby,
    Fuel,
    Time,
}

impl CollectibleKind {
    fn value(&self) -> u64 {
        match self {
            CollectibleKind::Silver => SILVER_VALUE,
            CollectibleKind::Gold => GOLD_VALUE,
            CollectibleKind::Diamond => DIAMOND_VALUE,
            CollectibleKind::Ruby => RUBY_VALUE,
            CollectibleKind::Fuel => 0,
            CollectibleKind::Time => 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Tile {
    Visible(TileKind),
    Fog { inner: TileKind, noise: f64 },
}

impl Tile {
    pub fn is_rock(&self) -> bool {
        matches!(
            self,
            Self::Fog {
                inner: TileKind::Rock { .. },
                ..
            } | Self::Visible(TileKind::Rock { .. })
        )
    }

    pub fn is_ground(&self) -> bool {
        matches!(
            self,
            Self::Fog {
                inner: TileKind::Ground { .. },
                ..
            } | Self::Visible(TileKind::Ground { .. })
        )
    }
}

#[derive(Clone, Copy)]
pub enum TileKind {
    Ground { depth: i64, noise: f64 },
    Sky { noise: f64 },
    Collectible { kind: CollectibleKind },
    Tunnel,
    Rock { noise: f64 },
}

#[derive(PartialEq, Eq)]
pub enum Message {
    Start,
    FuelLow,
    TimeLow,
    FuelAdded,
    TimeAdded,
    InventoryFull,
    SurfaceRefill(u64),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Start => write!(f, "Press an arrow key to move."),
            Message::FuelAdded => write!(f, "Found some extra fuel!"),
            Message::TimeAdded => write!(f, "Extra time added!"),
            Message::FuelLow => write!(f, "Warning: low fuel!"),
            Message::TimeLow => write!(f, "You're running out of time!"),
            Message::SurfaceRefill(v) => write!(f, "Fuel refilled! Banked {v} P!"),
            Message::InventoryFull => write!(f, "Your inventory is full!"),
        }
    }
}

/// per-run configuration produced by the store
pub struct Loadout {
    pub reveal_radius: i64,
    pub starting_fuel: f64,
    pub starting_time: Duration,
    pub inventory_capacity: usize,
}

impl Loadout {
    /// the default, un-upgraded loadout
    pub fn base() -> Self {
        Self {
            reveal_radius: BASE_REVEAL_RADIUS,
            starting_fuel: BASE_STARTING_FUEL,
            starting_time: BASE_STARTING_TIME,
            inventory_capacity: BASE_INVENTORY_CAPACITY,
        }
    }
}

pub struct Game {
    /// canvas width in pixels
    width: usize,

    /// (x, y) in pixels
    pos: (i64, i64),

    /// deepest depth reached this run
    max_depth: i64,

    /// fog reveal radius (from loadout)
    reveal_radius: i64,

    /// starting and max fuel (from loadout)
    starting_fuel: f64,

    /// max inventory size (from loadout)
    inventory_capacity: usize,

    /// store the input that will be consume next tick
    next_input: Option<Input>,

    /// whether timer has started yet
    started: bool,

    /// used to generate random stuff
    noise: Perlin,

    /// stores generated tiles in the game world
    tiles: HashMap<(i64, i64), Tile>,

    /// collectibles that the player is carrying
    inventory: HashMap<CollectibleKind, usize>,

    /// collectibles that the player has secured by
    /// returning to the surface
    bank: HashMap<CollectibleKind, usize>,

    /// remainging fuel
    fuel: f64,

    /// remaining time
    time: Duration,

    /// the message to be displayed right now (if any)
    /// and when it was posted
    message: Option<(Message, Instant)>,
}

impl Game {
    pub fn new(width: usize, loadout: Loadout) -> Self {
        let seed = rand::rng().random();

        Self {
            width,
            pos: (width as i64 / 2, SKY_HEIGHT - 1),
            max_depth: 0,
            reveal_radius: loadout.reveal_radius,
            starting_fuel: loadout.starting_fuel,
            inventory_capacity: loadout.inventory_capacity,
            next_input: None,
            started: false,
            noise: Perlin::new(seed),
            tiles: HashMap::new(),
            inventory: HashMap::new(),
            bank: HashMap::new(),
            fuel: loadout.starting_fuel,
            time: loadout.starting_time,
            message: Some((Message::Start, Instant::now())),
        }
    }

    pub fn handle(&mut self, input: Input) {
        self.next_input = Some(input)
    }

    fn handle_stored_input(&mut self) {
        if let Some(input) = self.next_input.take() {
            if !self.started {
                self.started = true
            }

            let pos = match input {
                Input::Up => (self.pos.0, (self.pos.1 - 1).max(SKY_HEIGHT - 1)),
                Input::Down => (self.pos.0, self.pos.1 + 1),
                Input::Right => ((self.pos.0 + 1).min(self.width as i64 - 1), self.pos.1),
                Input::Left => ((self.pos.0 - 1).max(0), self.pos.1),
            };

            self.move_or_dig(pos);
        }
    }

    /// moves the player into a position if they've already dug it out,
    /// or digs it, revealing a collectible if there is one
    fn move_or_dig(&mut self, pos: (i64, i64)) {
        if self.fuel <= 0. || self.time <= Duration::ZERO {
            return;
        }

        // store current depth for later comparison
        let prev_depth = self.current_depth();

        let tile = self.tiles.get(&pos).cloned();

        // if the tile is a rock, don't move or do anything (including consuming fuel)
        if tile.as_ref().is_some_and(|t| t.is_rock()) {
            return;
        }

        let is_up = pos.1 == self.pos.1 - 1;

        // if the tile is a ground tile, dig it (ie. replace it with a tunnel)
        //   *as long as it's not an upwards move*
        // otherwise move into the tile
        if tile.as_ref().is_some_and(|t| t.is_ground()) {
            if !is_up {
                self.tiles.insert(pos, Tile::Visible(TileKind::Tunnel));
                self.subtract_fuel(DIG_FUEL_COST);
            }
        } else {
            self.pos = pos;
            self.max_depth = self.max_depth.max(self.current_depth());
            self.reveal_fog();
            self.subtract_fuel(MOVE_FUEL_COST);
        }

        // collect a collectible if there's one where we just moved
        if let Some(Tile::Visible(TileKind::Collectible { kind })) = tile {
            let collected = match kind {
                CollectibleKind::Fuel => {
                    self.add_fuel();
                    true
                }
                CollectibleKind::Time => {
                    self.add_time();
                    true
                }
                _ => self.add_to_inventory(kind),
            };
            if collected {
                self.tiles.insert(pos, Tile::Visible(TileKind::Tunnel));
            }
        }

        // refill if we just got to the surface
        if self.current_depth() <= 0 && prev_depth > 0 {
            self.surface_refill();
        }
    }

    fn subtract_fuel(&mut self, fuel: f64) {
        let prev_fuel_prop = self.fuel_proportion();
        self.fuel -= fuel;

        if prev_fuel_prop >= LOW_FUEL_MESSAGE_THRESHOLD
            && self.fuel_proportion() < LOW_FUEL_MESSAGE_THRESHOLD
        {
            self.post_message(Message::FuelLow);
        }
    }

    fn subtract_time(&mut self, time: Duration) {
        let prev_remaining = self.remaining_time();
        self.time = self.time.saturating_sub(time);

        if prev_remaining >= LOW_TIME_MESSAGE_THRESHOLD
            && self.remaining_time() < LOW_TIME_MESSAGE_THRESHOLD
        {
            self.post_message(Message::TimeLow);
        }
    }

    /// ensures that all tiles within a radius of the player are revealed
    fn reveal_fog(&mut self) {
        // scan the square within the radius of the player
        let x_min = (self.pos.0 - self.reveal_radius).max(0);
        let x_max = (self.pos.0 + self.reveal_radius).min(self.width as i64);
        let y_min = (self.pos.1 - self.reveal_radius).max(0);
        let y_max = self.pos.1 + self.reveal_radius;

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                // actually check the radius though so we get a circular reveal pattern.
                // we add 0.5 to the radius to get a slightly smoother circle
                if (x - self.pos.0).pow(2) as f64 + (y - self.pos.1).pow(2) as f64
                    > (self.reveal_radius as f64 + 0.5).powi(2)
                {
                    continue;
                }

                if let Some(tile) = self.tiles.get_mut(&(x, y))
                    && let Tile::Fog { inner, .. } = tile
                {
                    *tile = Tile::Visible(*inner)
                }
            }
        }
    }

    pub fn tick(&mut self, elapsed: Duration) {
        self.handle_stored_input();
        if self.started {
            self.subtract_time(elapsed);

            // clear a message if there is one and it's been posted long enough
            if let Some((_, start)) = self.message
                && start.elapsed() > MESSAGE_DISPLAY_TIME
            {
                self.message = None;
            }
        }
    }

    pub fn pos(&self) -> (i64, i64) {
        self.pos
    }

    /// gets the tile at the given location, randomly generating it if
    /// that hasn't been done previously
    pub fn get_tile(&mut self, pos: (i64, i64)) -> Tile {
        if let Some(tile) = self.tiles.get(&pos) {
            *tile
        } else {
            let tile = if pos.1 < SKY_HEIGHT {
                let tile_kind = TileKind::Sky {
                    noise: self.noise.get([
                        pos.0 as f64 * SKY_NOISE_INPUT_SCALE + SKY_NOISE_OFFSET,
                        pos.1 as f64 * SKY_NOISE_INPUT_SCALE + SKY_NOISE_OFFSET,
                    ]),
                };

                Tile::Visible(tile_kind)
            } else {
                let tile_kind = if self.roll_rock(pos) {
                    TileKind::Rock {
                        noise: self.noise.get([
                            pos.0 as f64 * GROUND_NOISE_INPUT_SCALE,
                            pos.1 as f64 * GROUND_NOISE_INPUT_SCALE,
                        ]),
                    }
                } else if let Some(collectible) = self.roll_collectible(pos) {
                    TileKind::Collectible { kind: collectible }
                } else {
                    TileKind::Ground {
                        depth: pos.1 - SKY_HEIGHT,
                        noise: self.noise.get([
                            pos.0 as f64 * GROUND_NOISE_INPUT_SCALE,
                            pos.1 as f64 * GROUND_NOISE_INPUT_SCALE,
                        ]),
                    }
                };

                Tile::Fog {
                    inner: tile_kind,
                    noise: self.noise.get([
                        pos.0 as f64 * FOG_NOISE_INPUT_SCALE + FOG_NOISE_OFFSET,
                        pos.1 as f64 * FOG_NOISE_INPUT_SCALE + FOG_NOISE_OFFSET,
                    ]),
                }
            };

            self.tiles.insert(pos, tile);

            tile
        }
    }

    fn roll_collectible(&self, pos: (i64, i64)) -> Option<CollectibleKind> {
        // roll to determine if there's even a collectible here
        //
        // combining noise like this to try to get "peaks" so that collectibles appear mostly in isolation
        let roll =
            0.5 * self.noise.get([
                pos.0 as f64 * COLLECTIBLE_NOISE_INPUT_SCALE1 + COLLECTIBLE_NOISE_OFFSET,
                pos.1 as f64 * COLLECTIBLE_NOISE_INPUT_SCALE1 + COLLECTIBLE_NOISE_OFFSET,
            ]) + 0.5
                * self.noise.get([
                    pos.0 as f64 * COLLECTIBLE_NOISE_INPUT_SCALE2 + COLLECTIBLE_NOISE_OFFSET,
                    pos.1 as f64 * COLLECTIBLE_NOISE_INPUT_SCALE2 + COLLECTIBLE_NOISE_OFFSET,
                ]);

        // increase density of collectibles gradually with depth
        let depth_mod = COLLECTIBLE_DEPTH_MULTIPLIER * pos.1 as f64;

        if roll + depth_mod > COLLECTIBLE_THRESHOLD {
            // roll again to determine collectible kind
            let roll = self.noise.get([
                pos.0 as f64 * COLLECTIBLE_KIND_NOISE_INPUT_SCALE + COLLECTIBLE_KIND_NOISE_OFFSET,
                pos.1 as f64 * COLLECTIBLE_KIND_NOISE_INPUT_SCALE + COLLECTIBLE_KIND_NOISE_OFFSET,
            ]);

            // increase quality of collectibles gradually with depth
            let depth = self.current_depth();
            let depth_mod = COLLECTIBLE_KIND_DEPTH_MULTIPLIER * depth as f64;

            let roll = roll + depth_mod;

            if roll > RUBY_NOISE_THRESHOLD && depth > RUBY_DEPTH_THRESHOLD {
                Some(CollectibleKind::Ruby)
            } else if roll > DIAMOND_NOISE_THRESHOLD && depth > DIAMOND_DEPTH_THRESHOLD {
                Some(CollectibleKind::Diamond)
            } else if roll > TIME_NOISE_THRESHOLD && depth > TIME_DEPTH_THRESHOLD {
                Some(CollectibleKind::Time)
            } else if roll > FUEL_NOISE_THRESHOLD && depth > FUEL_DEPTH_THRESHOLD {
                Some(CollectibleKind::Fuel)
            } else if roll > GOLD_NOISE_THRESHOLD && depth > GOLD_DEPTH_THRESHOLD {
                Some(CollectibleKind::Gold)
            } else {
                Some(CollectibleKind::Silver)
            }
        } else {
            None
        }
    }

    fn roll_rock(&self, pos: (i64, i64)) -> bool {
        // increase density of collectibles gradually with depth
        let depth_mod = ROCK_DEPTH_MULTIPLIER * pos.1 as f64;

        // don't put rocks near the surface
        pos.1 > ROCK_Y_THRESHOLD
            && self.noise.get([
                pos.0 as f64 * ROCK_NOISE_INPUT_SCALE_X + ROCK_NOISE_OFFSET,
                pos.1 as f64 * ROCK_NOISE_INPUT_SCALE_Y + ROCK_NOISE_OFFSET,
            ]) + depth_mod
                > ROCK_NOISE_THRESHOLD
    }

    // returns whether or not the item was collected
    // (it may not have been if the player is at max capacity)
    fn add_to_inventory(&mut self, collectible: CollectibleKind) -> bool {
        if self.inventory_size() < self.inventory_capacity {
            let count = self.inventory.entry(collectible).or_insert(0);
            *count += 1;
            true
        } else {
            self.post_message(Message::InventoryFull);
            false
        }
    }

    pub fn inventory(&self) -> &HashMap<CollectibleKind, usize> {
        &self.inventory
    }

    fn surface_refill(&mut self) {
        let amount_stored = self.store_inventory();
        self.refill_fuel();

        self.post_message(Message::SurfaceRefill(amount_stored));
    }

    pub fn inventory_size(&self) -> usize {
        self.inventory.values().sum()
    }

    /// returns the amount stored
    fn store_inventory(&mut self) -> u64 {
        let inventory = self.inventory.clone();

        let mut total_val = 0;
        for (coll, inv_count) in &inventory {
            let bank_count = self.bank.entry(*coll).or_insert(0);
            *bank_count += inv_count;
            total_val += *inv_count as u64 * coll.value();
        }

        self.inventory = HashMap::new();

        total_val
    }

    pub fn bank_value(&self) -> u64 {
        self.bank
            .iter()
            .map(|(coll, count)| *count as u64 * coll.value())
            .sum()
    }

    pub fn fuel_proportion(&self) -> f64 {
        (self.fuel / self.starting_fuel).max(0.)
    }

    /// max fuel is capped at the initial fuel capacity
    fn add_fuel(&mut self) {
        self.fuel = (self.fuel + FUEL_COLLECTIBLE_GAIN).min(self.starting_fuel);
        self.post_message(Message::FuelAdded);
    }

    fn refill_fuel(&mut self) {
        self.fuel = self.starting_fuel;
    }

    pub fn remaining_time(&self) -> Duration {
        self.time
    }

    fn add_time(&mut self) {
        self.time += TIME_COLLECTIBLE_GAIN;
        self.post_message(Message::TimeAdded);
    }

    pub fn current_depth(&self) -> i64 {
        self.pos().1 - SKY_HEIGHT + 1
    }

    pub fn max_depth(&self) -> i64 {
        self.max_depth
    }

    pub fn inventory_capacity(&self) -> usize {
        self.inventory_capacity
    }

    /// the run is over once fuel or time runs out
    pub fn is_over(&self) -> bool {
        self.fuel <= 0. || self.time == Duration::ZERO
    }

    fn post_message(&mut self, message: Message) {
        // don't post an existing message
        if let Some((m, _)) = &self.message
            && *m == message
        {
            return;
        }

        self.message = Some((message, Instant::now()));
    }

    pub fn message(&self) -> Option<String> {
        self.message.as_ref().map(|(m, _)| m.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 60;

    fn game() -> Game {
        Game::new(WIDTH, Loadout::base())
    }

    #[test]
    fn new_game_is_not_over() {
        assert!(!game().is_over());
    }

    #[test]
    fn new_game_max_depth_is_zero() {
        assert_eq!(game().max_depth(), 0);
    }

    #[test]
    fn moving_down_increases_max_depth() {
        let mut game = game();

        game.handle(Input::Down);
        game.tick(Duration::from_millis(1));

        assert_eq!(game.current_depth(), 1);
        assert_eq!(game.max_depth(), 1);
    }

    #[test]
    fn moving_back_up_keeps_max_depth() {
        let mut game = game();

        game.handle(Input::Down);
        game.tick(Duration::from_millis(1));
        game.handle(Input::Up);
        game.tick(Duration::from_millis(1));

        assert_eq!(game.current_depth(), 0);
        assert_eq!(game.max_depth(), 1);
    }

    #[test]
    fn out_of_fuel_ends_run() {
        let mut game = game();
        game.fuel = 0.;
        assert!(game.is_over());
    }

    #[test]
    fn out_of_time_ends_run() {
        let mut game = game();
        game.time = Duration::ZERO;
        assert!(game.is_over());
    }

    #[test]
    fn banked_value_survives_end_of_run() {
        let mut game = game();
        game.bank.insert(CollectibleKind::Gold, 3);
        game.fuel = 0.;

        assert!(game.is_over());
        assert_eq!(game.bank_value(), 3 * GOLD_VALUE);
    }

    #[test]
    fn loadout_sets_starting_resources() {
        let game = Game::new(
            WIDTH,
            Loadout {
                reveal_radius: 9,
                starting_fuel: 3000.,
                starting_time: Duration::from_secs(180),
                inventory_capacity: 60,
            },
        );

        assert_eq!(game.reveal_radius, 9);
        assert_eq!(game.starting_fuel, 3000.);
        assert_eq!(game.inventory_capacity(), 60);
        assert_eq!(game.remaining_time(), Duration::from_secs(180));
        // fuel starts full relative to the upgraded tank
        assert_eq!(game.fuel_proportion(), 1.0);
    }
}
