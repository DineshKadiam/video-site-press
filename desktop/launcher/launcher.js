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

async function deleteSite(path) {
    await invoke('delete_site', { path });
}

function startEditor(path) {
    invoke('start_editor', { siteDir: path });
}

async function updateBody() {
    const body = await invoke('render');

    document.body.innerHTML = body;

    const addSiteButton = document.querySelector('button.add_site');
    const languageSelect = document.querySelector('.language select');
    const sites = document.querySelectorAll('.site');

    addSiteButton.addEventListener('click', event => {
        event.preventDefault();
        addSite();
    });

    languageSelect.addEventListener('change', event => {
        event.preventDefault();
        changeLanguage(languageSelect.value);
    });

    for (const site of sites) {
        const configureButton = site.querySelector('.configure');
        const deleteButton = site.querySelector('.delete');
        const pane = site.querySelector('.pane');
        const sitePath = site.dataset.sitePath;

        pane.addEventListener('click', event => {
            event.preventDefault();

            if (configureButton.contains(event.target)) {
                site.classList.toggle('expanded');
            } else {
                startEditor(sitePath);
            }
        });

        deleteButton.addEventListener('click', event => {
            event.preventDefault();
            deleteSite(sitePath);
        });
    }
}

async function updateLanguage() {
    const code = await invoke('get_language');
    document.documentElement.lang = code;
}

window.addEventListener('DOMContentLoaded', () => {
    updateBody();
    updateLanguage();
});
