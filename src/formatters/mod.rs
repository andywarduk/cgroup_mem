use std::iter::successors;

use ratatui::style::{Color, Style};
use ratatui::text::Span;

const POWERS: [&str; 7] = [" ", "k", "M", "G", "T", "P", "E"];
const COLOURS: [Color; 7] = [
    Color::LightGreen,
    Color::LightBlue,
    Color::LightYellow,
    Color::LightRed,
    Color::LightRed,
    Color::LightRed,
    Color::LightRed,
];

pub fn format_mem_qty(bytes: usize) -> Span<'static> {
    let mut fbytes = bytes as f64;
    let mut power = 0;

    while power < 6 && fbytes >= 1024_f64 {
        power += 1;
        fbytes /= 1024_f64;
    }

    let style = Style::default().fg(COLOURS[power]);

    let dp = if power > 1 {
        let digits = successors(Some(fbytes), |&n| (n >= 10_f64).then_some(n / 10_f64)).count();
        4 - digits
    } else {
        0
    };

    Span::styled(format!("{:>5.*} {}", dp, fbytes, POWERS[power]), style)
}

pub fn format_qty(qty: usize) -> Span<'static> {
    let mut fqty = qty as f64;
    let mut power = 0;

    while power < 6 && fqty >= 1000_f64 {
        power += 1;
        fqty /= 1000_f64;
    }

    let style = Style::default().fg(COLOURS[power]);

    let dp = if power > 0 {
        let digits = successors(Some(fqty), |&n| (n >= 10_f64).then_some(n / 10_f64)).count();
        3 - digits
    } else {
        0
    };

    Span::styled(format!("{:>4.*} {}", dp, fqty, POWERS[power]), style)
}
