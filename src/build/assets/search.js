const searchContainer = document.querySelector('.search');

const closeButton = searchContainer.querySelector('button');
const searchInput = searchContainer.querySelector('input');
const searchResults = searchContainer.querySelector('.results');
const statusField = searchContainer.querySelector('[role="status"]');

const indexSuffix = window.location.pathname.endsWith('index.html') ? 'index.html' : '';
const rootPrefix = searchContainer.dataset.rootPrefix;

// We only render the searchable content (which are used as the displayed
// search results) into the DOM when the search field first gains focus.
let searchInitialized = false;

// If a search query is entered before the search has been initialized, we use
// this as the interval handle for repeatedly retrying the search until
// search has finished initializing.
let delayedUpdateInterval = null;

function initializeResults() {
    for (const item of SITE_ITEMS) {
        let image;
        if (item.image) {
            image = document.createElement('img');
            image.src = rootPrefix + item.url + item.image;
        } else {
            image = document.createElement('span');
            image.classList.add('placeholder');
        }

        const spanText = document.createElement('span');
        spanText.dataset.searchable = 'true';
        spanText.textContent = item.title;

        const aRow = document.createElement('a');
        aRow.href = rootPrefix + item.url + indexSuffix;
        aRow.appendChild(image);
        aRow.appendChild(spanText);

        searchResults.appendChild(aRow);
    }

    searchInitialized = true;
}

function hideResults() {
    searchInput.value = '';
    searchContainer.classList.remove('open');
    statusField.removeAttribute('aria-label');
    statusField.textContent = '';
    searchInput.focus();
}

function updateResults() {
    const query = searchInput.value.trim();

    if (query.length) {
        const regexp = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
        let shown = 0;

        for (const row of searchResults.children) {
            const title = row.querySelector('[data-searchable]').textContent;
            const display = regexp.test(title);
            row.style.setProperty('display', display ? null : 'none');
            if (display) { shown += 1; }
        }

        if (shown === 0) {
            statusField.removeAttribute('aria-label');
            statusField.textContent = SEARCH_JS_T.nothingFoundForXxx(query);
        } else {
            statusField.setAttribute('aria-label', SEARCH_JS_T.showingXxxResultsForXxx(shown, query));
            statusField.textContent = '';
        }

        searchContainer.classList.add('open');
    } else {
        searchContainer.classList.remove('open');
        statusField.removeAttribute('aria-label');
        statusField.textContent = '';
    }
}

closeButton.addEventListener('click', hideResults);

const handleFirstSearchFocus = () => {
    initializeResults();
    searchInput.removeEventListener('focus', handleFirstSearchFocus);
};

searchInput.addEventListener('focus', handleFirstSearchFocus);

searchInput.addEventListener('input', () => {
    if (searchInitialized) {
        updateResults();
    } else if (delayedUpdateInterval == null) {
        delayedUpdateInterval = setInterval(
            () => {
                if (searchInitialized) {
                    clearInterval(delayedUpdateInterval);
                    delayedUpdateInterval = null;
                    updateResults();
                }
            },
            20
        );
    }
});

searchContainer.addEventListener('keydown', event => {
    if (event.key === 'ArrowUp') {
        const visibleResults = [...searchResults.children].filter(row => row.style.display !== 'none');

        if (visibleResults.length) {
            const targetRowIndex = visibleResults.indexOf(event.target);

            if (targetRowIndex === -1) {
                visibleResults[visibleResults.length - 1].focus();
            } else if (targetRowIndex > 0) {
                visibleResults[targetRowIndex - 1].focus();
            } else {
                searchInput.focus();
            }
        }
        event.preventDefault();
    } else if (event.key === 'ArrowDown') {
        const visibleResults = [...searchResults.children].filter(row => row.style.display !== 'none');

        if (visibleResults.length) {
            const targetRowIndex = visibleResults.indexOf(event.target);

            if (targetRowIndex === -1) {
                visibleResults[0].focus();
            } else if (targetRowIndex + 1 < visibleResults.length) {
                visibleResults[targetRowIndex + 1].focus();
            } else {
                searchInput.focus();
            }
        }
        event.preventDefault();
    } else if (event.key === 'Escape') {
        hideResults();
        event.preventDefault();
    }
});

window.addEventListener('keydown', event => {
    if (event.target === searchInput) return;

    if (event.key === 's') {
        searchInput.focus();
        event.preventDefault();
    }
});
