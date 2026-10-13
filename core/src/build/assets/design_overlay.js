const copyIcon = document.querySelector('template[data-copy-icon]').content;
const failedIcon = document.querySelector('template[data-failed-icon]').content;
const successIcon = document.querySelector('template[data-success-icon]').content;
const supportedColorSchemes = document.querySelector('meta[name=color-scheme]').content;

let theme;

if (supportedColorSchemes === 'dark' || supportedColorSchemes === 'light') {
    // In this case adaptive theming is off, so we can just work with a
    // single, static theme
    theme = supportedColorSchemes;
} else {
    // In this case adaptive theming is on, so we determine what the current
    // preferred or pinned theme is, and register watchers/hook to update
    // this later at runtime. This notably interacts with our
    // adaptive_theme.js implementation, so make sure to keep the two in
    // sync.

    const PREFERS_DARK_COLOR_SCHEME = matchMedia('(prefers-color-scheme: dark)');

    function preferredColorScheme() {
        return PREFERS_DARK_COLOR_SCHEME.matches ? 'dark' : 'light';
    }

    let pinnedTheme = localStorage.getItem('HYPER8_PINNED_THEME');

    theme = pinnedTheme || preferredColorScheme();

    PREFERS_DARK_COLOR_SCHEME.addEventListener('change', ({ matches }) => {
        if (!pinnedTheme) {
            theme = matches ? 'dark' : 'light';
            updateState(true);
        }
    });

    // Register a hook that (if present) adaptive_theme.js calls to notify us of a
    // theme being pinned/unpinned
    window.designOverlayNotifyPinned = newPinnedTheme => {
        pinnedTheme = newPinnedTheme;

        if (pinnedTheme) {
            theme = pinnedTheme;
        } else {
            theme = preferredColorScheme();
        }

        updateState(true);
    };
}

const designOverlay = document.querySelector('.design_overlay');
const copyChangesButton = designOverlay.querySelector('button[data-copy-changes');
const copyChangesButtonIconContainer = copyChangesButton.querySelector('.icon');
const levels = designOverlay.querySelector('.levels');
const manifest = designOverlay.querySelector('.manifest');
const palette = designOverlay.querySelector('.palette');
const settings = designOverlay.querySelector('.settings')
const showChangesButton = designOverlay.querySelector('button[data-show-changes');

let placement = 'c';

const SETTINGS_MAP = {
    accentBrightening: {
        defaultValue: 50,
        key: 'accent_brightening',
        label: 'Accent Brightening',
        range: [0, 100],
        tooltip: 'Controls additional brightening for the accent color (0-100%, Default 50%)',
        unit: '%'
    },
    accentChroma: {
        defaultValue: 0,
        key: 'accent_chroma',
        label: 'Accent Chroma',
        range: [0, 100],
        tooltip: 'Sets the chroma (colorfulness) for the accent color (0-100%, Default 0%)',
        unit: '%'
    },
    accentHue: {
        defaultValue: 0,
        key: 'accent_hue',
        label: 'Accent Hue',
        range: [0, 360],
        tooltip: 'Sets the hue (color) for the accent color (0-360 degrees, Default 0)',
        unit: 'deg'
    },
    baseChroma: {
        defaultValue: 0,
        key: 'base_chroma',
        label: 'Base Chroma',
        range: [0, 100],
        tooltip: 'Sets the chroma (colorfulness) for the base layer (0-100%, Default 0%)',
        unit: '%'
    },
    baseHue: {
        defaultValue: 0,
        key: 'base_hue',
        label: 'Base Hue',
        range: [0, 360],
        tooltip: 'Sets the hue (color) for the base layer (0-360 degrees, Default 0)',
        unit: 'deg'
    },
    contrast: {
        defaultValue: 0,
        key: 'contrast',
        label: 'Contrast',
        range: [0, 100],
        tooltip: 'Controls how far the dynamic range of this theme extends towards black (with a dark base) or white (with a light base) (0-100%, Default 0%)',
        unit: '%'
    },
    layeredChroma: {
        defaultValue: 0,
        key: 'layered_chroma',
        label: 'Layered Chroma',
        range: [0, 100],
        tooltip: 'Sets the chroma (colorfulness) for elements above the base layer (0-100%, Default 0%)',
        unit: '%'
    },
    roundness: {
        defaultValue: 80,
        key: 'roundness',
        label: 'Roundness',
        range: [0, 100],
        tooltip: 'Controls how angular or round the interface looks and feels (0-100%, Default 80%)',
        unit: '%',
        universal: true
    }
};

const SETTINGS = [
    SETTINGS_MAP.contrast,
    SETTINGS_MAP.baseChroma,
    SETTINGS_MAP.baseHue,
    SETTINGS_MAP.layeredChroma,
    SETTINGS_MAP.accentBrightening,
    SETTINGS_MAP.accentChroma,
    SETTINGS_MAP.accentHue,
    SETTINGS_MAP.roundness
];

const snapshotKey = 'HYPER8_DESIGNER';
const snapshotJson = window.localStorage.getItem(snapshotKey);
const snapshot = snapshotJson ? JSON.parse(snapshotJson) : null;

if (snapshot?.placement) {
    placement = snapshot.placement;
    designOverlay.dataset.placement = placement;
}

// Every time a new build is made, the design overlay snapshot is discarded, and
// build time is the marker to determine a new build has happened.
if (snapshot && snapshot.buildInvocationTime === BUILD_SETTINGS.buildInvocationTime) {
    for (const setting of SETTINGS) {
        if (setting.universal) {
            setting.value = snapshot[setting.key];
        } else {
            setting.value = {
                dark: snapshot.dark[setting.key],
                light: snapshot.light[setting.key]
            };
        }
    }
} else {
    for (const setting of SETTINGS) {
        if (setting.universal) {
            setting.value = BUILD_SETTINGS[setting.key];
        } else {
            setting.value = {
                dark: BUILD_SETTINGS.dark[setting.key],
                light: BUILD_SETTINGS.light[setting.key]
            };
        }
    }
}

let persistTimeout = null;
function persistDebounced() {
    if (persistTimeout) { clearTimeout(persistTimeout); }

    persistTimeout = setTimeout(
        () => {
            const snapshot = {
                buildInvocationTime: BUILD_SETTINGS.buildInvocationTime,
                dark: {},
                light: {},
                placement
            };

            for (const setting of SETTINGS) {
                if (setting.universal) {
                    snapshot[setting.key] = setting.value;
                } else {
                    snapshot.dark[setting.key] = setting.value.dark;
                    snapshot.light[setting.key] = setting.value.light;
                }
            }

            persistTimeout = null;
            window.localStorage.setItem(snapshotKey, JSON.stringify(snapshot));
        },
        250
    );
}

function place(event) {
    placement = event.target.dataset.placement;
    designOverlay.dataset.placement = placement;
    persistDebounced();
}

for (const button of designOverlay.querySelectorAll('.placement button')) {
    button.addEventListener('click', place);
}

// lightness in percent (0-100)
// b3 --- high
// bm
// m
// fm
// f3 --- low
// f2
// f1 --- lowest
function chromaShaper(highChromaL, lowChromaL, lowestChromaL, theme) {
    // const ease = attenuator => Math.sin(attenuator * (Math.PI / 2));

    const HIGH_CHROMA = 0.5;
    const LOW_CHROMA = 0.03;
    const LOWEST_CHROMA = 0;

    if (theme === 'dark') {
        return lightness => {
            if (lightness <= lowChromaL) {
                return HIGH_CHROMA - (HIGH_CHROMA - LOW_CHROMA) * ((lightness - highChromaL) / (lowChromaL - highChromaL));
            } else if (lightness <= lowestChromaL) {
                return LOW_CHROMA - (LOW_CHROMA - LOWEST_CHROMA) * ((lightness - lowChromaL) / (lowestChromaL - lowChromaL));
            } else /* if (lightness > lowestChromaL) */ {
                return LOWEST_CHROMA;
            }
        };
    } else {
        return lightness => {
            if (lightness >= lowChromaL) {
                return LOW_CHROMA + (HIGH_CHROMA - LOW_CHROMA) * ((lightness - lowChromaL) / (highChromaL - lowChromaL));
            } else if (lightness >= lowestChromaL) {
                return LOWEST_CHROMA + (LOW_CHROMA - LOWEST_CHROMA) * ((lightness - lowestChromaL) / (lowChromaL - lowestChromaL));
            } else /* if (lightness < lowestChromaL) */ {
                return LOWEST_CHROMA;
            }
        };
    }
}

// Return a css oklch declaration for the passed lcha values
function oklch(l, c, h, a) {
    if (a === undefined) {
        return `oklch(${l}% ${c}% ${h})`;
    } else {
        return `oklch(${l}% ${c}% ${h} / ${a}%)`;
    }
}

// Given x percent (0-100), return the value that lies x percent on the
// numerical spectrum going from range[0] to range[1]. For instance, 50% on
// the range [0, 42] is 21.
function pickFromRange(percent, range) {
    return range[0] + (percent / 100) * (range[1] - range[0]);
}

// This function returns the limited maximum `contrast` that can be used to
// interpolate lightness values from the theme param ranges
function maxContrastForChroma(chroma, furthestBackgroundRange, theme) {
    // Noticable erratic shifts in the OKLCH color model appear for very dark
    // values with around 40%+ chroma, so we explicitly compensate for this
    if (theme === 'dark' && chroma > 40) {
        const compensationFactor = (chroma - 40) / 60;

        const darkestRangeBrightest = furthestBackgroundRange[0];
        const darkestRangeDarkest = furthestBackgroundRange[1];

        // Around <=20% lightness oklch produces very erratic hue shifts at
        // extreme chroma levels, therefore we shift lightness quite a bit
        // away from pure black when we're dealing with extreme chroma
        // levels
        const minLightness = compensationFactor * 20;

        if (darkestRangeDarkest >= minLightness) {
            // The full range can be used
            return 100;
        } else if (darkestRangeBrightest < minLightness) {
            // This means that the brightest lightness produced by the theme
            // at 0% contrast still doesn't suffice to compensate (raise
            // lightness enough to avoid chroma artifacts). Ideally the theme
            // should be set up to not cross this boundary, but minimal
            // overshoot is ok in practice.
            return 0;
        } else {
            const rangeLightnessDelta = darkestRangeBrightest - darkestRangeDarkest;
            const maxContrast = (1 - (minLightness - darkestRangeDarkest) / rangeLightnessDelta) * 100;

            return maxContrast;
        }
    }

    // OKLCH compensates conflicting color components (e.g. 100% lightness
    // with 100% chroma) very sensibly in the bright range, therefore we
    // don't apply any bespoke compensation for light colors

    return 100;
}

function updateCssVariables() {
    const themeParams = theme === 'dark' ? DARK_THEME : LIGHT_THEME;

    const BASE_CHROMA_DAMPENING = 0.4;
    const LAYERED_CHROMA_DAMPENING = 0.2;

    // User-supplied theme paint
    const accentBrightening = SETTINGS_MAP.accentBrightening.value[theme];
    const accentChroma = SETTINGS_MAP.accentChroma.value[theme];
    const accentHue = SETTINGS_MAP.accentHue.value[theme];
    const baseChroma = SETTINGS_MAP.baseChroma.value[theme] * BASE_CHROMA_DAMPENING;
    const baseHue = SETTINGS_MAP.baseHue.value[theme];
    const contrast = SETTINGS_MAP.contrast.value[theme];
    const layeredChroma = SETTINGS_MAP.layeredChroma.value[theme] * LAYERED_CHROMA_DAMPENING;
    const roundness = SETTINGS_MAP.roundness.value;

    // System-defined theme params
    const {
        b1LRange,
        b2LRange,
        b3LRange,
        bmL,
        d1LRange,
        d2LRange,
        d3LRange,
        dmL,
        f1LRange,
        f2LRange,
        f3LRange,
        fmL,
        l1LRange,
        l2LRange,
        l3LRange,
        mL
    } = themeParams;

    const maxContrast = maxContrastForChroma(baseChroma, b1LRange, theme);
    const limitedContrast = (contrast / 100) * maxContrast;

    const accL = 70 + (accentBrightening / 100) * 30;
    const b1L = pickFromRange(limitedContrast, b1LRange);
    const b2L = pickFromRange(limitedContrast, b2LRange);
    const b3L = pickFromRange(limitedContrast, b3LRange);
    const d1L = pickFromRange(limitedContrast, d1LRange);
    const d2L = pickFromRange(limitedContrast, d2LRange);
    const d3L = pickFromRange(limitedContrast, d3LRange);
    const f1L = pickFromRange(limitedContrast, f1LRange);
    const f2L = pickFromRange(limitedContrast, f2LRange);
    const f3L = pickFromRange(limitedContrast, f3LRange);
    const l1L = pickFromRange(limitedContrast, l1LRange);
    const l2L = pickFromRange(limitedContrast, l2LRange);
    const l3L = pickFromRange(limitedContrast, l3LRange);

    const setLevel = (name, lightness) => {
        const topOffset = theme === 'dark' ? 100 - lightness : lightness;
        levels.querySelector(`.level[data-name="${name}"]`).style.setProperty('top', `${topOffset}%`)
    };

    setLevel('b1', b1L);
    setLevel('b2', b2L);
    setLevel('b3', b3L);
    setLevel('bm', bmL);
    setLevel('f1', f1L);
    setLevel('f2', f2L);
    setLevel('f3', f3L);
    setLevel('fm', fmL);
    setLevel('m', mL);

    const setTone = (name, lightness) => {
        const tone = palette.querySelector(`.tone .monochrome[data-name="${name}"]`);
        tone.style.setProperty('background', `oklch(${lightness}% 0% 0)`);
    };

    setTone('acc', accL);
    setTone('b1', b1L);
    setTone('b2', b2L);
    setTone('b3', b3L);
    setTone('bm', bmL);
    setTone('f1', f1L);
    setTone('f2', f2L);
    setTone('f3', f3L);
    setTone('fm', fmL);
    setTone('m', mL);

    // We observed that in the light theme the base chroma needs to bleed more
    // into the detail chroma than in the dark theme in order to be perceived
    // as equally saturated.
    const baseChromaBleed = theme === 'dark' ? 0.5 : 0.75;

    // The final layered chroma is at least the amount of chroma bleeding over
    // from baseChroma, or higher if the user requested a higher
    // layeredChroma expliclity.
    const basePlusLayeredChroma = Math.max(baseChroma * baseChromaBleed, layeredChroma);

    // The returned function shapes chroma distribution across the lower
    // tones (b3-f1) by attenuating the chroma the closer the passed
    // lightness lies to the foreground lightness. For this it uses fixed
    // ratios (see chromaShaper function definition).
    const shapeC = chromaShaper(b3L, f3L, f1L, theme);

    const r = roundness / 100;

    const set = (key, value) => document.body.style.setProperty(`--${key}`, value);

    // Theme color overrides
    set('acc', oklch(accL, accentChroma, accentHue));
    set('b1', oklch(b1L, baseChroma, baseHue));
    set('b1-85', oklch(b1L, baseChroma, baseHue, 85));
    set('b1-92', oklch(b1L, baseChroma, baseHue, 92));
    set('b2', oklch(b2L, basePlusLayeredChroma, baseHue));
    set('b3', oklch(b3L, basePlusLayeredChroma, baseHue));
    set('bm', oklch(bmL, baseChroma * shapeC(bmL), baseHue));
    set('d1', oklch(d1L, baseChroma * shapeC(d1L), baseHue));
    set('d1-15', oklch(d1L, baseChroma * shapeC(d1L), baseHue, 15));
    set('d1-70-mono', oklch(d1L, 0, 0, 70));
    set('d1-85-mono', oklch(d1L, 0, 0, 85));
    set('d1-mono', oklch(d1L, 0, 0));
    set('d2', oklch(d2L, baseChroma * shapeC(d2L), baseHue));
    set('d3', oklch(d3L, baseChroma * shapeC(d3L), baseHue));
    set('d3-mono', oklch(d3L, 0, 0));
    set('dm', oklch(dmL, baseChroma * shapeC(dmL), baseHue));
    set('f1', oklch(f1L, baseChroma * shapeC(f1L), baseHue));
    set('f1-5', oklch(f1L, baseChroma * shapeC(f1L), baseHue, 5));
    set('f2', oklch(f2L, baseChroma * shapeC(f2L), baseHue));
    set('f3', oklch(f3L, baseChroma * shapeC(f3L), baseHue));
    set('fm', oklch(fmL, baseChroma * shapeC(fmL), baseHue));
    set('l1-mono', oklch(l1L, 0, 0));
    set('l1-10-mono', oklch(l1L, 0, 0, 10));
    set('l2-mono', oklch(l2L, 0, 0));
    set('l3-mono', oklch(l3L, 0, 0));
    set('m', oklch(mL, baseChroma * shapeC(mL), baseHue));
    set('r', r);

    // Designer widget only colors
    set('f1-mono', oklch(f1L, 0, 0));
    set('f2-mono', oklch(f2L, 0, 0));
    set('f3-mono', oklch(f3L, 0, 0));
    set('b1-80-mono', oklch(b1L, 0, 0, 80));
    set('b2-mono', oklch(b2L, 0, 0));
    set('b3-mono', oklch(b3L, 0, 0));
    set('m-mono', oklch(mL, 0, 0));
}

function updateSettings() {
    for (const setting of SETTINGS) {
        const { defaultValue, unit, universal } = setting;

        const value = universal ? setting.value : setting.value[theme];

        setting.input.value = value ?? setting.input.min;

        const defaultLabel = value === defaultValue ? ' (Default)' : '';
        const unitLabel = value !== null && unit ? unit : '';

        setting.spanValue.textContent = `${value ?? 'None'}${unitLabel}${defaultLabel}`;
    }
}

function updateState(includeSettings = false) {
    updateCssVariables();
    updateManifest();

    if (includeSettings) {
        updateSettings();
    }
}

function updateManifest() {
    let dark = [];
    let light = [];
    let universal = [];

    for (const setting of SETTINGS) {
        if (setting.universal) {
            if (setting.value !== setting.defaultValue) {
                universal.push(`${setting.key}: ${setting.value}`);
            }
        } else {
            if (setting.value.dark !== setting.defaultValue) {
                dark.push(`${setting.key} = ${setting.value.dark}`);
            }

            if (setting.value.light !== setting.defaultValue) {
                light.push(`${setting.key} = ${setting.value.light}`);
            }
        }
    }

    let content = '';

    if (universal.length > 0) {
        content += universal.join('\n');
        content += '\n\n';
    }

    if (dark.length > 0) {
        content += 'dark_theme:\n';
        content += dark.join('\n');
        content += '\n\n';
    }

    if (light.length > 0) {
        content += 'light_theme:\n';
        content += light.join('\n');
        content += '\n\n';
    }

    manifest.value = content;
}

for (const setting of SETTINGS) {
    const { defaultValue, label, key, range, tooltip, unit } = setting;

    const valueLabel = () => {
        const value = setting.universal ? setting.value : setting.value[theme];

        const defaultLabel = value === defaultValue ? ' (Default)' : '';
        const unitLabel = value !== null && unit ? unit : '';

        return `${value ?? 'None'}${unitLabel}${defaultLabel}`;
    };

    const spanValue = document.createElement('span');

    spanValue.classList.add('value');
    spanValue.textContent = valueLabel();

    const input = document.createElement('input');

    input.min = defaultValue === null ? range[0] - 1 : range[0];
    input.max = range[1];
    input.title = tooltip;
    input.type = 'range';

    const getValue = setting.universal
        ? () => setting.value
        : () => setting.value[theme];

    const setValue = setting.universal
        ? value => { setting.value = value; }
        : value => { setting.value[theme] = value; };

    input.value = getValue() ?? input.min;

    input.addEventListener('mousedown', event => {
        // Middle click
        if (event.button === 1) {
            event.preventDefault();
            setValue(defaultValue);

            input.value = defaultValue ?? input.min;

            updateState();

            spanValue.textContent = valueLabel();

            persistDebounced();
        }
    });

    input.addEventListener('input', () => {
        if (input.valueAsNumber < range[0]) {
            setValue(null);
        } else {
            setValue(input.valueAsNumber);
        }

        updateState();

        spanValue.textContent = valueLabel();

        persistDebounced();
    });

    const spanLabel = document.createElement('span');

    spanLabel.classList.add('label');
    spanLabel.textContent = label;
    spanLabel.title = tooltip;

    const settingHeaderDiv = document.createElement('div');

    settingHeaderDiv.appendChild(spanLabel);
    settingHeaderDiv.appendChild(spanValue);

    const settingDiv = document.createElement('div');

    settingDiv.classList.add('setting');
    settingDiv.dataset.name = key;
    settingDiv.appendChild(settingHeaderDiv);
    settingDiv.appendChild(input);

    settings.appendChild(settingDiv);

    setting.input = input;
    setting.spanValue = spanValue;
}

updateState();

let endFeedbackEarly = null;

function copyFeedback(feedbackIcon) {
    if (endFeedbackEarly) { endFeedbackEarly(); }

    copyChangesButtonIconContainer.replaceChildren(feedbackIcon.cloneNode(true));

    function endFeedback() {
        copyChangesButtonIconContainer.replaceChildren(copyIcon.cloneNode(true));
        endFeedbackEarly = null;
    }

    const endFeedbackScheduled = setTimeout(endFeedback, 3000);

    endFeedbackEarly = () => {
        clearTimeout(endFeedbackScheduled);
        endFeedback();
    };
}

copyChangesButton.addEventListener('click', () => {
    navigator.clipboard
        .writeText(manifest.value)
        .then(() => copyFeedback(successIcon))
        .catch(_err => copyFeedback(failedIcon));
});

showChangesButton.addEventListener('click', () => {
    manifest.classList.toggle('active');
});
