// SPDX-FileCopyrightText: 2024-2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{
    Build,
    Site,
    ThemePaint,
    ThemeParams
};

fn theme_vars(paint: &Option<ThemePaint>) -> String {
    let paint = match paint {
        Some(paint) => paint,
        None => &ThemePaint::default()
    };

    let mut props = Vec::new();

    let mut add = |name, value| {
        let js = format!("{name}:{value}");
        props.push(js);
    };

    add("accent_brightening", paint.accent_brightening().to_string());
    add("accent_chroma", paint.accent_chroma().to_string());
    add("accent_hue", paint.accent_hue().to_string());
    add("base_chroma", paint.base_chroma().to_string());
    add("base_hue", paint.base_hue().to_string());
    add("contrast", paint.contrast().to_string());
    add("layered_chroma", paint.layered_chroma().to_string());

    let props = props.join(",");

    format!("{{{props}}}")
}

pub fn design_overlay(
    build: &Build,
    site: &Site
) -> String {
    let build_invocation_time = build.invocation_time;
    let roundness = site.roundness();

    let dark = theme_vars(&site.dark_theme);
    let light = theme_vars(&site.light_theme);

    let dark_theme_params = ThemeParams::DARK.js();
    let light_theme_params = ThemeParams::LIGHT.js();

    let mut script = formatdoc!(r#"
        const BUILD_SETTINGS = {{
            buildInvocationTime:'{build_invocation_time}',
            dark:{dark},
            light:{light},
            roundness:{roundness}
        }};
        const DARK_THEME = {dark_theme_params};
        const LIGHT_THEME = {light_theme_params};
    "#);

    script.push_str(include_str!("assets/design_overlay.js"));

    let copy_icon = hyper8_icons::copy(None);

    formatdoc!(r#"
        <div autofocus class="design_overlay" data-placement="c" tabindex="0">
            <div class="settings"></div>
            <div class="controls">
                <div class="placement">
                    <button data-placement="nw"></button>
                    <button data-placement="n"></button>
                    <button data-placement="ne"></button>
                    <button data-placement="w"></button>
                    <button data-placement="c"></button>
                    <button data-placement="e"></button>
                    <button data-placement="sw"></button>
                    <button data-placement="s"></button>
                    <button data-placement="se"></button>
                </div>
                <div>Middle-click a slider to reset its value</div>
            </div>
            <div class="controls">
                <button data-show-changes>Show changes</button>
                <button data-copy-changes><span class="icon">{copy_icon}</span> Copy changes</button>
            </div>
            <div class="palette">
                <div class="tone">
                    <span>acc</span>
                    <div style="background: var(--acc);"></div>
                    <div class="monochrome" data-name="acc"></div>
                </div>
                <div class="tone">
                    <span>b1</span>
                    <div style="background: var(--b1);"></div>
                    <div class="monochrome" data-name="b1"></div>
                </div>
                <div class="tone">
                    <span>b2</span>
                    <div style="background: var(--b2);"></div>
                    <div class="monochrome" data-name="b2"></div>
                </div>
                <div class="tone">
                    <span>b3</span>
                    <div style="background: var(--b3);"></div>
                    <div class="monochrome" data-name="b3"></div>
                </div>
                <div class="tone">
                    <span>bm</span>
                    <div style="background: var(--bm);"></div>
                    <div class="monochrome" data-name="bm"></div>
                </div>
                <div class="tone">
                    <span>m</span>
                    <div style="background: var(--m);"></div>
                    <div class="monochrome" data-name="m"></div>
                </div>
                <div class="tone">
                    <span>fm</span>
                    <div style="background: var(--fm);"></div>
                    <div class="monochrome" data-name="fm"></div>
                </div>
                <div class="tone">
                    <span>f3</span>
                    <div style="background: var(--f3);"></div>
                    <div class="monochrome" data-name="f3"></div>
                </div>
                <div class="tone">
                    <span>f2</span>
                    <div style="background: var(--f2);"></div>
                    <div class="monochrome" data-name="f2"></div>
                </div>
                <div class="tone">
                    <span>f1</span>
                    <div style="background: var(--f1);"></div>
                    <div class="monochrome" data-name="f1"></div>
                </div>
            </div>
            <div class="levels">
                <div class="beam"></div>
                <div class="level" data-name="b1">
                    <div>b1</div>
                </div>
                <div class="level" data-name="b2">
                    <div>b2</div>
                </div>
                <div class="level" data-name="b3">
                    <div>b3</div>
                </div>
                <div class="level" data-name="bm">
                    <div>bm</div>
                </div>
                <div class="level" data-name="f1">
                    <div>f1</div>
                </div>
                <div class="level" data-name="f2">
                    <div>f2</div>
                </div>
                <div class="level" data-name="f3">
                    <div>f3</div>
                </div>
                <div class="level" data-name="fm">
                    <div>fm</div>
                </div>
                <div class="level" data-name="m">
                    <div>m</div>
                </div>
            </div>
            <textarea class="manifest" readonly></textarea>
        </div>
        <script>{script}</script>
    "#)
}
