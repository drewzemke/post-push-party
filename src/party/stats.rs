use std::collections::HashSet;

use super::{Party, PartyColor, RenderContext, BOLD, CYAN, GRAY, MAGENTA, RESET, YELLOW};

/// the most basic party that shows how many points were earned
pub struct Stats;

impl Party for Stats {
    fn id(&self) -> &'static str {
        "stats"
    }

    fn name(&self) -> &'static str {
        "Stats Party"
    }

    fn description(&self) -> &'static str {
        "Shows information about your commit and push trends."
    }

    fn cost(&self) -> u64 {
        1000
    }

    fn supports_color(&self) -> bool {
        false
    }

    fn render(&self, ctx: &RenderContext, _color: &PartyColor) {
        let clock = ctx.clock;

        // this push
        let push_commits = ctx.push.commits();
        let push_commit_count = push_commits.len();
        let push_lines = ctx
            .push
            .commits()
            .iter()
            .map(|c| c.lines_changed())
            .sum::<u64>();
        let push_points = ctx.breakdown.total;

        // today
        let today_pushes = ctx
            .history
            .entries()
            .iter()
            .filter(|push| clock.is_today(push.timestamp()));
        let mut today_commit_count = 0;
        let mut today_lines = 0;
        let mut today_points = 0;

        for push in today_pushes {
            today_commit_count += push.commits();
            today_lines += push.lines_changed();
            today_points += push.points_earned();
        }

        // all time
        let all_pushes = ctx.history.entries().iter();

        let mut days: HashSet<i64> = HashSet::new();
        let mut all_time_commit_count = 0;
        let mut all_time_lines = 0;
        let mut all_time_points = 0;

        for push in all_pushes {
            let day = clock.day_of(push.timestamp());
            days.insert(day);

            all_time_commit_count += push.commits();
            all_time_lines += push.lines_changed();
            all_time_points += push.points_earned();
        }

        let num_days = days.len().max(1);

        // daily_average
        let daily_avg_commit_count = all_time_commit_count / num_days as u64;
        let daily_avg_lines = all_time_lines / num_days as u64;
        let daily_avg_points = all_time_points / num_days as u64;

        // calculate max widths for each column
        let w_commits = col_width([
            push_commit_count as u64,
            today_commit_count,
            daily_avg_commit_count,
            all_time_commit_count,
        ]);
        let w_lines = col_width([push_lines, today_lines, daily_avg_lines, all_time_lines]);
        let w_points = col_width([push_points, today_points, daily_avg_points, all_time_points]);

        // helper function to print each row of output
        let print_row = |header: &str, commits: u64, lines: u64, points: u64| {
            let commit_word = if commits == 1 { "commit" } else { "commits" };
            let point_word = if points == 1 { "point" } else { "points" };
            let line_word = if lines == 1 { "line" } else { "lines" };

            let header = format!("{BOLD}{header:>wh$}{RESET}", wh = 10);
            let commits = format!("{MAGENTA}{commits}{RESET} {commit_word},");
            let lines = format!("{YELLOW}{lines}{RESET} {line_word} changed,");
            let points = format!("{CYAN}{points}{RESET} {point_word}");

            println!(
                "{header}: {commits:<wc$} {lines:<wl$} {points:<wp$}",
                wc = w_commits + 18,
                wl = w_lines + 24,
                wp = w_points + 6
            );
        };

        println!(" Stats");
        println!("{GRAY} ─────{RESET}");

        // only show the "this push" row if there was more than one commit that counted
        if push_commit_count > 0 {
            print_row(
                "This Push",
                push_commit_count as u64,
                push_lines,
                push_points,
            );
        }

        print_row("Today", today_commit_count, today_lines, today_points);
        print_row(
            "Daily Avg",
            daily_avg_commit_count,
            daily_avg_lines,
            daily_avg_points,
        );
        print_row(
            "All Time",
            all_time_commit_count,
            all_time_lines,
            all_time_points,
        );
    }
}

/// helper function to compute the width of the output columns
fn col_width(vals: [u64; 4]) -> usize {
    vals.iter().map(|n| n.to_string().len()).max().unwrap_or(1)
}
