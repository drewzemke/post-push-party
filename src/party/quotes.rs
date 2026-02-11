use crate::party::ITALICS;

use super::{random_pick, Party, PartyColor, RenderContext, RESET};

const QUOTES: &[(&str, &str)] = &[
// simplicity / craft:
("Simplicity is prerequisite for reliability." , "Edsger Dijkstra"),
("Simplicity is a great virtue but it requires hard work to achieve it and education to appreciate it." , "Edsger Dijkstra"),
("The competent programmer is fully aware of the limited size of his own skull." , "Edsger Dijkstra"),
("Always remember, however, that there's usually a simpler and better way to do something than the first way that pops into your head." , "Donald Knuth"),
("The enjoyment of one's tools is an essential ingredient of successful work." , "Donald Knuth"),
("Data dominates. If you've chosen the right data structures and organized things well, the algorithms will almost always be self-evident." , "Rob Pike"),

// code for humans:
("Any fool can write code that a computer can understand. Good programmers write code that humans can understand." , "Martin Fowler"),
("Programming is the art of telling another human being what one wants the computer to do." , "Donald Knuth"),
("When you feel the need to write a comment, first try to refactor the code so that any comment becomes superfluous." , "Martin Fowler"),

// humor
("If debugging is the process of removing software bugs, then programming must be the process of putting them in." , "Edsger Dijkstra"),
("Beware of bugs in the above code; I have only proved it correct, not tried it." , "Donald Knuth"),
("One of my most productive days was throwing away 1000 lines of code." , "Ken Thompson"),
("Theory and practice sometimes clash. And when that happens, theory loses. Every single time." , "Linus Torvalds"),

// testing / debugging:
("Testing shows the presence, not the absence of bugs." , "Edsger Dijkstra"),
("Thinking before debugging is extremely important. If you dive into the bug, you tend to fix the local issue in the code, but if you think about the bug first, you often find and correct a higher-level problem." , "Rob Pike"),

// habits:
("I'm not a great programmer; I'm just a good programmer with great habits." , "Kent Beck"),
("Optimism is an occupational hazard of programming: feedback is the treatment." , "Kent Beck"),
];

/// the most basic party that shows how many points were earned
pub struct Quotes;

impl Party for Quotes {
    fn id(&self) -> &'static str {
        "quotes"
    }

    fn name(&self) -> &'static str {
        "Quotes Party"
    }

    fn description(&self) -> &'static str {
        "Shares a nerdy quote after you push."
    }

    fn cost(&self) -> u64 {
        500
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &RenderContext, color: &PartyColor) {
        let offset = color.random_offset();
        let color0 = color.get(offset);
        let color1 = color.get(offset + 1);

        let (quote, author) = random_pick(QUOTES);

        // FIXME: print this out more intelligently so that it word-wraps
        // in the terminal
        println!("{color0}{ITALICS}\"{quote}\"{RESET} — {color1}{author}{RESET}");
    }
}
