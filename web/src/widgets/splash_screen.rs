// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_core::SPLASH_SVG;
use hyper8_version::{VERSION_ADAPTIVE_PATCH, VERSION_WITH_PATCH};

use crate::SessionState;

pub fn splash_screen(session_state: &SessionState) -> String {
    let translations = &session_state.language.translations;

    let data_automatic = if session_state.automatic_splash { "data-automatic" } else { "" };

    let close_icon = hyper8_icons::close(translations.close);

    let t_donate = translations.donate;
    let t_website = translations.website;
    formatdoc!(r##"
        <dialog class="splash_wrapper" {data_automatic}>
            <div class="splash">
                <form class="close" method="dialog">
                    <button autofocus>{close_icon}</button>
                </form>
                {SPLASH_SVG}
                <div>
                    <a href="https://simonrepp.com/hyper8" target="_blank">
                        {t_website}
                    </a>
                    <a class="donate" href="https://simonrepp.com/hyper8/#donate" target="_blank">
                        {t_donate}
                    </a>
                    <a class="version" href="https://simonrepp.com/hyper8/changes/{VERSION_WITH_PATCH}/" target="_blank">
                        <span>{VERSION_ADAPTIVE_PATCH}</span>
                    </a>
                </div>
            </div>
        </dialog>
    "##)
}
