const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

listen('update-body', updateBody);

async function changeLanguage(code) {
    await invoke('change_language', { code });
    document.documentElement.lang = code;
    updateBody();
}

function close() {
    invoke('close');
}

function createNewSite() {
    invoke('create_new_site');
}

function openInFileExplorer(siteDir) {
    invoke('open_in_file_explorer', { siteDir });
}

function openBookmarkedSite(siteDir) {
    invoke('open_bookmarked_site', { siteDir });
}

async function openExistingSite() {
    await invoke('open_existing_site');
    updateBody();
}

async function removeSiteBookmark(siteDir) {
    await invoke('remove_site_bookmark', { siteDir });
}

async function updateBody() {
    const body = await invoke('render');

    document.body.innerHTML = body;

    const closeButton = document.querySelector('button.close');
    const createNewSiteButton = document.querySelector('button.create_new_site');
    const languageSelect = document.querySelector('.language select');
    const openExistingSiteButton = document.querySelector('button.open_existing_site');

    closeButton.addEventListener('click', event => {
        event.preventDefault();
        close();
    });

    createNewSiteButton.addEventListener('click', event => {
        event.preventDefault();
        createNewSite();
    });

    languageSelect.addEventListener('change', event => {
        event.preventDefault();
        changeLanguage(languageSelect.value);
    });

    openExistingSiteButton.addEventListener('click', event => {
        event.preventDefault();
        openExistingSite();
    });

    for (const site of document.querySelectorAll('.site')) {
        const configureButton = site.querySelector('.configure');
        const openInFileExplorerButton = site.querySelector('.open_in_file_explorer');
        const pane = site.querySelector('.pane');
        const removeSiteBookmarkButton = site.querySelector('.remove_site_bookmark');
        const siteDir = site.dataset.siteDir;

        pane.addEventListener('click', event => {
            event.preventDefault();

            if (configureButton.contains(event.target)) {
                site.classList.toggle('expanded');
            } else if (!site.classList.contains('not_found')) {
                openBookmarkedSite(siteDir);
            }
        });

        openInFileExplorerButton.addEventListener('click', event => {
            event.preventDefault();
            openInFileExplorer(siteDir);
        });

        removeSiteBookmarkButton.addEventListener('click', event => {
            event.preventDefault();
            removeSiteBookmark(siteDir);
        });
    }
}

async function updateLanguage() {
    const code = await invoke('get_language');
    document.documentElement.lang = code;
}

async function updateTheme() {
    const preferDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const css = await invoke('get_theme', { preferDark });
    document.querySelector('style[data-theme]').textContent = css;
}

window.addEventListener('DOMContentLoaded', () => {
    updateBody();
    updateLanguage();
    updateTheme();
});
