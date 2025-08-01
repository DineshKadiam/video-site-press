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
    // Create the search result element for the video inside the DOM, store a
    // reference to it in the siteContent tree
    function initializeVideo(video) {
        let image;
        if (video.image) {
            image = document.createElement('img');
            image.src = rootPrefix + video.sitePath + video.image;
        } else {
            image = document.createElement('span');
            image.classList.add('placeholder');
        }

        const spanText = document.createElement('span');
        spanText.textContent = video.title;

        video.element = document.createElement('a');
        video.element.classList.add('video');
        video.element.href = rootPrefix + video.sitePath + indexSuffix;
        video.element.appendChild(image);
        video.element.appendChild(spanText);

        searchResults.appendChild(video.element);
    }

    // Create the search result element for the playlist inside the DOM, store
    // a reference to it in the siteContent tree
    function initializePlaylist(playlist, ancestors = []) {
        const traversal = [...ancestors, playlist];

        const spanText = document.createElement('span');
        spanText.textContent = traversal.map(ancestor => ancestor.title).join(' > ');

        playlist.element = document.createElement('a');
        playlist.element.classList.add('playlist');
        playlist.element.href = rootPrefix + playlist.sitePath + indexSuffix;
        playlist.element.appendChild(spanText);

        searchResults.appendChild(playlist.element);

        for (const video of playlist.videos) {
            initializeVideo(video);
        }

    }

    // Create the search result element for the collection inside the DOM,
    // store a reference to it in the siteContent tree
    function initializeCollection(collection, ancestors = []) {
        const traversal = [...ancestors, collection];

        const spanText = document.createElement('span');
        spanText.textContent = traversal.map(ancestor => ancestor.title).join(' > ');

        collection.element = document.createElement('a');
        collection.element.classList.add('collection');
        collection.element.href = rootPrefix + collection.sitePath + indexSuffix;
        collection.element.appendChild(spanText);

        searchResults.appendChild(collection.element);

        for (const video of collection.videos) {
            initializeVideo(video);
        }

        for (const playlist of collection.playlists) {
            initializePlaylist(playlist, traversal);
        }

        for (const subcollection of collection.subcollections) {
            initializeCollection(subcollection, traversal);
        }
    }

    if (siteContent.collection) {
        for (const video of siteContent.collection.videos) {
            initializeVideo(video);
        }

        for (const playlist of siteContent.collection.playlists) {
            initializePlaylist(playlist);
        }

        for (const subcollection of siteContent.collection.subcollections) {
            initializeCollection(subcollection);
        }
    } else /* if (siteContent.playlist) */ {
        for (const video of siteContent.playlist.videos) {
            initializeVideo(video);
        }
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

        function updateVideo(video) {
            const display = regexp.test(video.title);
            video.element.classList.toggle('visible', display);
            if (display) { shown += 1; }
            return display;
        }

        function updatePlaylist(playlist) {
            let display = regexp.test(playlist.title);
            if (display) { shown += 1; }

            for (const video of playlist.videos) {
                display = updateVideo(video) || display;
            }

            playlist.element.classList.toggle('visible', display);
            return display;
        }

        function updateCollection(collection) {
            let display = regexp.test(collection.title);
            if (display) { shown += 1; }

            for (const video of collection.videos) {
                display = updateVideo(video) || display;
            }

            for (const playlist of collection.playlists) {
                updatePlaylist(playlist);
            }

            for (const subcollection of collection.subcollections) {
                updateCollection(subcollection);
            }

            collection.element.classList.toggle('visible', display);
            return display;
        }

        if (siteContent.collection) {
            for (const video of siteContent.collection.videos) {
                updateVideo(video);
            }

            for (const playlist of siteContent.collection.playlists) {
                updatePlaylist(playlist);
            }

            for (const subcollection of siteContent.collection.subcollections) {
                updateCollection(subcollection);
            }
        } else {
            for (const video of siteContent.playlist.videos) {
                updateVideo(video);
            }
        }

        if (shown === 0) {
            searchResults.style.setProperty('display', 'none');
            statusField.removeAttribute('aria-label');
            statusField.textContent = SEARCH_JS_T.nothingFoundForXxx(query);
        } else {
            searchResults.style.setProperty('display', null);
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
        const visibleResults = [...searchResults.querySelectorAll('a.visible')];

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
        const visibleResults = [...searchResults.querySelectorAll('a.visible')];

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
