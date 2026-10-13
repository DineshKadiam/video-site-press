const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

listen('show-overlay', ({ payload }) => showOverlay(payload));
listen('update-body', updateBody);
listen('update-language', ({ payload }) => updateLanguage(payload));
listen('update-theme', updateTheme);

function changeLanguage(code) {
    invoke('change_language', { code });
}

function changeMode(key) {
    invoke('change_mode', { key });
}

function changeTheme(key) {
    invoke('change_theme', { key });
}

function close() {
    invoke('close');
}

function createNewSite() {
    invoke('create_new_site');
}

function openInFileManager(siteDir) {
    invoke('open_in_file_manager', { siteDir });
}

function openBookmarkedSite(siteDir) {
    invoke('open_bookmarked_site', { siteDir });
}

async function openExistingSite() {
    await invoke('open_existing_site');
    updateBody();
}

function removeSiteBookmark(siteDir) {
    invoke('remove_site_bookmark', { siteDir });
}

function showOverlay(content) {
    const overlay = document.querySelector('.overlay')
    overlay.innerHTML = content;
    overlay.showModal();

    const quitButton = overlay.querySelector('button.quit');
    quitButton.addEventListener('click', close);

    const copyIcon = document.querySelector('template[data-copy-icon]').content;
    const failedIcon = document.querySelector('template[data-failed-icon]').content;
    const successIcon = document.querySelector('template[data-success-icon]').content;

    let endFeedbackEarly = null;

    const copyButton = document.querySelector('.overlay button[data-url]');

    function copyFeedback(content, feedbackIcon) {
        if (endFeedbackEarly) { endFeedbackEarly(); }

        copyButton.replaceChildren(feedbackIcon.cloneNode(true));

        function endFeedback() {
            copyButton.replaceChildren(copyIcon.cloneNode(true));
            endFeedbackEarly = null;
        }

        const endFeedbackScheduled = setTimeout(endFeedback, 3000);

        endFeedbackEarly = () => {
            clearTimeout(endFeedbackScheduled);
            endFeedback();
        };
    }
    copyButton.addEventListener('click', () => {
        const url = copyButton.dataset.url;
        navigator.clipboard
            .writeText(url)
            .then(() => copyFeedback(content, successIcon))
            .catch(_err => copyFeedback(content, failedIcon));
    });

    document.querySelector('.overlay').children[0].focus();
}

async function updateBody() {
    const body = await invoke('render');

    document.body.innerHTML = body;

    const closeButton = document.querySelector('button.close');
    const createNewSiteButton = document.querySelector('button.create_new_site');
    const languageSelector = document.querySelector('.language + .selector');
    const modeSelector = document.querySelector('.mode + .selector');
    const openExistingSiteButton = document.querySelector('button.open_existing_site');
    const themeSelector = document.querySelector('.theme + .selector');

    closeButton.addEventListener('click', close);

    createNewSiteButton.addEventListener('click', createNewSite);

    for (const button of languageSelector.querySelectorAll('button')) {
        button.addEventListener('click', () => {
            changeLanguage(button.dataset.code);
            button.blur();
        });
    }

    for (const button of modeSelector.querySelectorAll('button')) {
        button.addEventListener('click', () => {
            changeMode(button.dataset.key);
            button.blur();
        });
    }

    openExistingSiteButton.addEventListener('click', () => {
        openExistingSite();
    });

    for (const button of themeSelector.querySelectorAll('button')) {
        button.addEventListener('click', () => {
            changeTheme(button.dataset.key);
            button.blur();
        });
    }

    for (const site of document.querySelectorAll('.site')) {
        const configureButton = site.querySelector('.configure');
        const openInFileManagerButton = site.querySelector('.open_in_file_manager');
        const pane = site.querySelector('.pane');
        const removeSiteBookmarkButton = site.querySelector('.remove_site_bookmark');
        const siteDir = site.dataset.siteDir;

        pane.addEventListener('click', () => {
            if (configureButton.contains(event.target)) {
                site.classList.toggle('expanded');
            } else if (!site.classList.contains('not_found')) {
                openBookmarkedSite(siteDir);
            }
        });

        openInFileManagerButton.addEventListener('click', () => {
            openInFileManager(siteDir);
        });

        removeSiteBookmarkButton.addEventListener('click', () => {
            removeSiteBookmark(siteDir);
        });
    }
}

async function updateLanguage(code = null) {
    if (code === null) {
        document.documentElement.lang = await invoke('get_language_code');
    } else {
        document.documentElement.lang = code;
    }
}

async function updateTheme() {
    const [css, icon] = await invoke('get_theme');
    document.querySelector('button.theme').innerHTML = icon;
    document.querySelector('style[data-theme]').textContent = css;
}

window.addEventListener('DOMContentLoaded', () => {
    updateBody();
    updateLanguage();
    updateTheme();
});
