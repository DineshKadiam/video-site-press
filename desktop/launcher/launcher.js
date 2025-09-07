const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

listen('update-body', updateBody);

// TODO: In addSite() & deleteSite() (and coming endpoints) we need to await
// the actual updates to state before running the update, (with addSite consider
// that there is actually asyncness coming from us waiting for the user to choose
// their folder!

async function addSite() {
    await invoke('add_site');
    updateBody();
}

async function changeLanguage(code) {
    await invoke('change_language', { code });
    document.documentElement.lang = code;
    updateBody();
}

function close() {
    invoke('close');
}

async function deleteSite(siteDir) {
    await invoke('delete_site', { siteDir });
}

function openInFileExplorer(siteDir) {
    invoke('open_in_file_explorer', { siteDir });
}

function quickStart() {
    invoke('quick_start');
}

function startEditor(siteDir) {
    invoke('start_editor', { siteDir });
}

async function updateBody() {
    const body = await invoke('render');

    document.body.innerHTML = body;

    const addSiteButton = document.querySelector('button.add_site');
    const closeButton = document.querySelector('button.close');
    const languageSelect = document.querySelector('.language select');
    const quickStartButton = document.querySelector('button.quick_start');
    const sites = document.querySelectorAll('.site');

    addSiteButton.addEventListener('click', event => {
        event.preventDefault();
        addSite();
    });

    closeButton.addEventListener('click', event => {
        event.preventDefault();
        close();
    });

    languageSelect.addEventListener('change', event => {
        event.preventDefault();
        changeLanguage(languageSelect.value);
    });

    quickStartButton.addEventListener('click', event => {
        event.preventDefault();
        quickStart();
    });

    for (const site of sites) {
        const configureButton = site.querySelector('.configure');
        const deleteButton = site.querySelector('.delete');
        const openInFileExplorerButton = site.querySelector('.open_in_file_explorer');
        const pane = site.querySelector('.pane');
        const siteDir = site.dataset.siteDir;

        pane.addEventListener('click', event => {
            event.preventDefault();

            if (configureButton.contains(event.target)) {
                site.classList.toggle('expanded');
            } else if (!site.classList.contains('not_found')) {
                startEditor(siteDir);
            }
        });

        deleteButton.addEventListener('click', event => {
            event.preventDefault();
            deleteSite(siteDir);
        });

        openInFileExplorerButton.addEventListener('click', event => {
            event.preventDefault();
            openInFileExplorer(siteDir);
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
