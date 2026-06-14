use tixel::Color;

use super::game::{
    BASE_INVENTORY_CAPACITY, BASE_REVEAL_RADIUS, BASE_STARTING_FUEL, BASE_STARTING_TIME, Loadout,
};
use super::menu::{BORDER_COLOR, center, draw_box, write_text};

const TIER_COUNT: usize = 3;

const INNER_WIDTH: usize = 50;

const TITLE_COLOR: Color = Color::Rgb(250, 230, 120);
const CURSOR_COLOR: Color = Color::Rgb(250, 230, 120);
const LABEL_SELECTED: Color = Color::Rgb(255, 255, 255);
const LABEL_COLOR: Color = Color::Rgb(170, 170, 170);
const SLIDER_FILL: Color = Color::Rgb(90, 220, 120);
const SLIDER_EMPTY: Color = Color::Rgb(70, 70, 70);
const COST_COLOR: Color = Color::Rgb(230, 230, 230);
const COST_FREE: Color = Color::Rgb(110, 110, 110);
const TEXT_COLOR: Color = Color::Rgb(210, 210, 210);
const REMAINING_COLOR: Color = Color::Rgb(90, 220, 120);
const FOOTER_COLOR: Color = Color::Rgb(140, 140, 140);

/// indices into UPGRADES / Store::tiers
const VISIBILITY: usize = 0;
const START_TIME: usize = 1;
const FUEL_TANK: usize = 2;
const INVENTORY: usize = 3;

struct UpgradeDef {
    label: &'static str,
    /// total cost (in party points) to own each tier; costs[0] is always 0
    costs: [u64; TIER_COUNT],
}

const UPGRADES: [UpgradeDef; 4] = [
    UpgradeDef {
        label: "Visibility",
        costs: [0, 100, 200],
    },
    UpgradeDef {
        label: "Start Time",
        costs: [0, 500, 3000],
    },
    UpgradeDef {
        label: "Fuel Tank",
        costs: [0, 1000, 5000],
    },
    UpgradeDef {
        label: "Inventory",
        costs: [0, 1000, 2000],
    },
];

pub struct Store {
    /// the player's available party points; the spending cap
    budget: u64,
    /// which upgrade row is highlighted
    selected: usize,
    /// selected tier index per upgrade
    tiers: [usize; UPGRADES.len()],
}

impl Store {
    /// the budget is the player's current party-point balance
    pub fn new(budget: u64) -> Self {
        Self {
            budget,
            selected: 0,
            tiers: [0; UPGRADES.len()],
        }
    }

    /// move the highlight to the previous upgrade
    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// move the highlight to the next upgrade
    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1).min(UPGRADES.len() - 1);
    }

    /// raise the selected upgrade one tier, if affordable
    pub fn tier_up(&mut self) {
        let cur = self.tiers[self.selected];
        if cur + 1 >= TIER_COUNT {
            return;
        }

        let next = cur + 1;
        let costs = &UPGRADES[self.selected].costs;
        let new_spent = self.spent() - costs[cur] + costs[next];
        if new_spent <= self.budget {
            self.tiers[self.selected] = next;
        }
    }

    /// lower the selected upgrade one tier, refunding the difference
    pub fn tier_down(&mut self) {
        self.tiers[self.selected] = self.tiers[self.selected].saturating_sub(1);
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn tier(&self, upgrade: usize) -> usize {
        self.tiers[upgrade]
    }

    pub fn spent(&self) -> u64 {
        UPGRADES
            .iter()
            .zip(self.tiers)
            .map(|(u, t)| u.costs[t])
            .sum()
    }

    pub fn remaining(&self) -> u64 {
        self.budget - self.spent()
    }

    /// the loadout described by the current tier selections
    pub fn loadout(&self) -> Loadout {
        let mult = |upgrade: usize| self.tiers[upgrade] as u32 + 1;

        Loadout {
            reveal_radius: BASE_REVEAL_RADIUS * mult(VISIBILITY) as i64,
            starting_fuel: BASE_STARTING_FUEL * mult(FUEL_TANK) as f64,
            starting_time: BASE_STARTING_TIME * mult(START_TIME),
            inventory_capacity: BASE_INVENTORY_CAPACITY * mult(INVENTORY) as usize,
        }
    }

    pub fn render(&self, buf: &mut String, term: (usize, usize)) {
        // title, blank, 4 upgrades, blank, budget, blank, footer
        let inner_height = 10;
        let offset = center(term, INNER_WIDTH + 2, inner_height + 2);
        let (base, top) = offset;

        draw_box(buf, offset, INNER_WIDTH, inner_height);

        // title
        let title = "SHOP";
        let title_col = base + 1 + (INNER_WIDTH - title.len()) / 2;
        write_text(buf, top + 1, title_col, title, TITLE_COLOR);

        // upgrade rows
        let cursor_col = base + 3;
        let label_col = base + 5;
        let slider_col = base + 20;
        let tier_col = base + 27;

        for (i, def) in UPGRADES.iter().enumerate() {
            let row = top + 3 + i;
            let selected = i == self.selected();
            let tier = self.tier(i);

            let (cursor, label_color) = if selected {
                ("▶", LABEL_SELECTED)
            } else {
                (" ", LABEL_COLOR)
            };
            write_text(buf, row, cursor_col, cursor, CURSOR_COLOR);
            write_text(buf, row, label_col, def.label, label_color);

            // slider: [###] with `tier + 1` filled cells
            let filled = tier + 1;
            let empty = TIER_COUNT - filled;
            write_text(buf, row, slider_col, "[", BORDER_COLOR);
            write_text(buf, row, slider_col + 1, &"#".repeat(filled), SLIDER_FILL);
            write_text(
                buf,
                row,
                slider_col + 1 + filled,
                &"░".repeat(empty),
                SLIDER_EMPTY,
            );
            write_text(buf, row, slider_col + 1 + TIER_COUNT, "]", BORDER_COLOR);

            // tier multiplier
            write_text(buf, row, tier_col, &format!("{}x", tier + 1), TEXT_COLOR);

            // cost, right-aligned within the box
            let cost = def.costs[tier];
            let cost_str = format!("{cost} P");
            let cost_col = base + INNER_WIDTH - 3 - cost_str.len();
            let cost_color = if cost == 0 { COST_FREE } else { COST_COLOR };
            write_text(buf, row, cost_col, &cost_str, cost_color);
        }

        // budget summary
        let summary = format!("Spent {} P    Left ", self.spent());
        let summary_col = base + 15;
        write_text(buf, top + 8, summary_col, &summary, TEXT_COLOR);
        write_text(
            buf,
            top + 8,
            summary_col + summary.chars().count(),
            &format!("{} P", self.remaining()),
            REMAINING_COLOR,
        );

        // footer hint
        let footer = "[↑↓] move  [←→] buy  [Enter] start  [Q] quit";
        let footer_col = base + 1 + (INNER_WIDTH - footer.chars().count()) / 2;
        write_text(buf, top + 10, footer_col, footer, FOOTER_COLOR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BUDGET: u64 = 10_000;

    #[test]
    fn new_store_has_full_budget_and_no_tiers() {
        let store = Store::new(TEST_BUDGET);
        assert_eq!(store.budget, TEST_BUDGET);
        assert_eq!(store.spent(), 0);
        assert_eq!(store.remaining(), TEST_BUDGET);
    }

    #[test]
    fn selection_clamps_at_both_ends() {
        let mut store = Store::new(TEST_BUDGET);

        store.select_prev();
        assert_eq!(store.selected(), 0);

        for _ in 0..10 {
            store.select_next();
        }
        assert_eq!(store.selected(), UPGRADES.len() - 1);
    }

    #[test]
    fn raising_a_tier_spends_its_cost() {
        let mut store = Store::new(TEST_BUDGET);
        store.tier_up();

        assert_eq!(store.tier(VISIBILITY), 1);
        assert_eq!(store.spent(), UPGRADES[VISIBILITY].costs[1]);
    }

    #[test]
    fn lowering_a_tier_refunds() {
        let mut store = Store::new(TEST_BUDGET);
        store.tier_up();
        store.tier_up();
        assert_eq!(store.tier(VISIBILITY), 2);

        store.tier_down();
        assert_eq!(store.tier(VISIBILITY), 1);
        assert_eq!(store.spent(), UPGRADES[VISIBILITY].costs[1]);
    }

    #[test]
    fn tier_cannot_exceed_the_max() {
        let mut store = Store::new(TEST_BUDGET);
        for _ in 0..5 {
            store.tier_up();
        }
        assert_eq!(store.tier(VISIBILITY), TIER_COUNT - 1);
    }

    // a tight budget must block upgrades that would overspend
    #[test]
    fn tier_up_respects_a_tight_budget() {
        // only enough for the first visibility tier, nothing more
        let mut store = Store::new(UPGRADES[VISIBILITY].costs[1]);
        store.tier_up();
        assert_eq!(store.tier(VISIBILITY), 1);

        store.tier_up(); // tier 2 unaffordable
        assert_eq!(store.tier(VISIBILITY), 1);
        assert!(store.spent() <= store.budget);
    }

    // price-agnostic: derives expectations from UPGRADES so price tuning
    // doesn't break it
    #[test]
    fn tier_up_never_overspends() {
        // a budget tight enough that maxing everything is impossible
        let mut store = Store::new(2_000);

        for (upgrade, def) in UPGRADES.iter().enumerate() {
            if upgrade > 0 {
                store.select_next();
            }

            for _ in 0..TIER_COUNT {
                let before = store.tier(upgrade);
                store.tier_up();
                let after = store.tier(upgrade);

                // the invariant: never spend past the budget
                assert!(store.spent() <= store.budget);

                // a refused step that wasn't already maxed must be a budget block
                if after == before && before < TIER_COUNT - 1 {
                    let step = def.costs[before + 1] - def.costs[before];
                    assert!(step > store.remaining());
                }
            }
        }
    }

    #[test]
    fn loadout_applies_tier_multipliers() {
        let mut store = Store::new(TEST_BUDGET);
        store.tier_up(); // visibility -> 2x

        let loadout = store.loadout();
        assert_eq!(loadout.reveal_radius, BASE_REVEAL_RADIUS * 2);

        // untouched upgrades stay at base
        assert_eq!(loadout.starting_fuel, BASE_STARTING_FUEL);
        assert_eq!(loadout.starting_time, BASE_STARTING_TIME);
        assert_eq!(loadout.inventory_capacity, BASE_INVENTORY_CAPACITY);
    }

    #[test]
    fn loadout_multiplies_each_upgrade_independently() {
        let mut store = Store::new(TEST_BUDGET);

        // visibility 2x
        store.tier_up();
        // start time 2x
        store.select_next();
        store.tier_up();
        // inventory 2x
        store.select_next();
        store.select_next();
        store.tier_up();

        let loadout = store.loadout();
        assert_eq!(loadout.reveal_radius, BASE_REVEAL_RADIUS * 2);
        assert_eq!(loadout.starting_time, BASE_STARTING_TIME * 2);
        assert_eq!(loadout.inventory_capacity, BASE_INVENTORY_CAPACITY * 2);
        // fuel untouched
        assert_eq!(loadout.starting_fuel, BASE_STARTING_FUEL);
    }
}
