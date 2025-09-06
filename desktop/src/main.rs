// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    if let Some(_) = env::args().nth(1).as_deref() {
        // TODO: Integrate shared cli arg processing
        println!("For the time being please start the desktop application without any arguments, CLI support will be reintegrated shortly.");
    } else {
        hyper8_desktop_lib::run();
    }
}
