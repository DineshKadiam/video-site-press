// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::io::Write;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal;

/// Obtains password input from the user while hiding the typed characters.
/// Err(()) indicates that the user aborted with Ctrl+C.
pub fn password() -> Result<String, ()> {
    let mut stdout = io::stdout();

    stdout.flush().unwrap();

    terminal::enable_raw_mode().unwrap();

    let mut password = String::new();

    let success = loop {
        match event::read().unwrap() {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Backspace => {
                        password.pop();
                    }
                    KeyCode::Char(c) => {
                        if event.modifiers.intersects(KeyModifiers::CONTROL) && c == 'c' {
                            break false;
                        }

                        password.push(c);
                    }
                    KeyCode::Enter => {
                        // On Windows Command Prompt and PowerShell we observed
                        // the following event processing/timing issue:
                        //
                        // 1. User *presses* Enter to confirm either a sicuit
                        //    invocation or a password entry in sicuit
                        // 2. sicuit initializes the first or second password
                        //    prompt before the user *releases* Enter
                        // 3. When the user releases Enter sicuit instantly
                        //    interprets it as confirmation of the prompt
                        //
                        // For this reason we only confirm the prompt on Enter
                        // being pressed, but not on it being released. (Note:
                        // On Linux we observed no release events being emitted
                        // or caught at all)
                        if event.is_press() {
                            break true;
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    };

    terminal::disable_raw_mode().unwrap();

    print!("\n");
    stdout.flush().unwrap();

    if success {
        Ok(password)
    } else {
        Err(())
    }
}
