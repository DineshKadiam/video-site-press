// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use anstyle::{AnsiColor, Color, Style};

pub const BRIGHT_BLACK: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
pub const BRIGHT_YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)));
pub const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
pub const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
