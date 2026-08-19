// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use anstyle::{AnsiColor, Color, Style};

const fn bg_fg(bg_color: AnsiColor, fg_color: AnsiColor) -> Style {
    Style::new()
        .bg_color(Some(Color::Ansi(bg_color)))
        .fg_color(Some(Color::Ansi(fg_color)))
}

const fn fg(ansi_color: AnsiColor) -> Style {
    Style::new()
        .fg_color(Some(Color::Ansi(ansi_color)))
}

pub const BLACK_ON_BRIGHT_YELLOW: Style = bg_fg(AnsiColor::BrightYellow, AnsiColor::Black);
pub const BRIGHT_BLACK: Style = fg(AnsiColor::BrightBlack);
pub const BRIGHT_YELLOW: Style = fg(AnsiColor::BrightYellow);
pub const CYAN: Style = fg(AnsiColor::Cyan);
pub const GREEN: Style = fg(AnsiColor::Green);
pub const MAGENTA: Style = fg(AnsiColor::Magenta);
pub const RED: Style = fg(AnsiColor::Red);
