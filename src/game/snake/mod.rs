use crate::{game::Game, tui::Terminal};

/// Classic snake game in which users earn points as they grow their snake.
pub struct Snake;

impl Game for Snake {
    fn id(&self) -> &'static str {
        "snake"
    }

    fn name(&self) -> &'static str {
        "Snake"
    }

    fn description(&self) -> &'static str {
        "The classic game made popular by late-90s Nokia cell phones. Earn party points as you collect food and grow your snake."
    }

    fn cost(&self) -> u64 {
        25
    }

    fn run(&self, terminal: &mut Terminal) {
        todo!()
    }
}
