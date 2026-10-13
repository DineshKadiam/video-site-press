const themeToggles = document.querySelector('.theme_toggles');
const toggleDark = document.querySelector('.toggle_dark');
const toggleLight = document.querySelector('.toggle_light');

const PREFERS_DARK_COLOR_SCHEME = matchMedia('(prefers-color-scheme: dark)');

function preferredColorScheme() {
    return PREFERS_DARK_COLOR_SCHEME.matches ? 'dark' : 'light';
}

let pinnedTheme = localStorage.getItem('HYPER8_PINNED_THEME');

if (pinnedTheme && pinnedTheme !== preferredColorScheme()) {
    document.querySelector('body').dataset.pinnedTheme = pinnedTheme;
}

function toggleTheme(clickedTheme) {
    if (clickedTheme === preferredColorScheme()) {
        delete document.querySelector('body').dataset.pinnedTheme;

        if (pinnedTheme !== null) {
            pinnedTheme = null;
            localStorage.removeItem('HYPER8_PINNED_THEME');
        }
    } else {
        document.querySelector('body').dataset.pinnedTheme = clickedTheme;
        pinnedTheme = clickedTheme;
        localStorage.setItem('HYPER8_PINNED_THEME', clickedTheme);
    }

    // This interacts with the design overlay (when is is enabled). Make sure to
    // keep this in sync with the design_overlay.js implementation
    if (window.designOverlayNotifyPinned) {
        window.designOverlayNotifyPinned(pinnedTheme);
    }
}

toggleDark.addEventListener('click', () => toggleTheme('dark'));
toggleLight.addEventListener('click', () => toggleTheme('light'));
