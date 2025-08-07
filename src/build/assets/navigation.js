const chevronRightIcon = document.querySelector('#chevron_right_icon').content;

const browseButton = document.querySelector('button.browse');

const navigation = document.querySelector('header .navigation');

const searchContainer = document.querySelector('.search');
const clearSearchButton = searchContainer.querySelector('button');
const searchInput = searchContainer.querySelector('input');

const siteTree = document.querySelector('.site_tree');
const siteTreeElements = siteTree.querySelector('.elements');
const siteTreeStatus = siteTree.querySelector('[role="status"]');

const indexSuffix = window.location.pathname.endsWith('index.html') ? 'index.html' : '';
const rootPrefix = document.querySelector('.home').dataset.rootPrefix;
const sitePath = document.querySelector('.home').dataset.sitePath;

// We only render the site tree elements (which are displayed during
// browsing/searching) into the DOM when, or shortly before, we need them,
// for instance when the browse button or search input first gains focus.
// (null = uninitialized, false = initializing, true = initialized)
let siteTreeInitialized = null;

// If browsing/searching is initiated before the site tree has been
// initialized, we use this as the interval handle for repeatedly retrying
// the operation until the site tree has finished initializing.
let accessSiteTreeCallback = null;
let accessSiteTreeInterval = null;

function accessSiteTree(callback) {
    if (siteTreeInitialized === true) {
        callback();
    } else if (accessSiteTreeInterval === null) {
        if (siteTreeInitialized === null) {
            siteTreeInitialized = false;
            initializeSiteTree();
        }

        accessSiteTreeCallback = callback;
        accessSiteTreeInterval = setInterval(
            () => {
                if (siteTreeInitialized) {
                    clearInterval(accessSiteTreeInterval);
                    accessSiteTreeInterval = null;
                    accessSiteTreeCallback();
                }
            },
            20
        );
    } else {
        accessSiteTreeCallback = callback;
    }
}

function clearSearch() {
    searchContainer.classList.remove('query_present');
    searchInput.value = '';
    siteTree.classList.remove('open');
    siteTreeStatus.removeAttribute('aria-label');
    siteTreeStatus.textContent = '';
    searchInput.focus();
}

function initializeSiteTree() {
    browseButton.removeEventListener('focus', initializeSiteTree);
    searchInput.removeEventListener('focus', initializeSiteTree);

    // Create the search result element for the video inside the DOM, store a
    // reference to it in the siteContent tree
    function initializeVideo(video) {
        let image;
        if (video.image) {
            image = document.createElement('img');
            image.src = rootPrefix + video.sitePath + video.image;

            if (video.lqip) {
                image.style.background = lqipToRadialGradient(video.lqip);
            }
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

        if (video.sitePath === sitePath) {
            video.element.classList.add('current');
        }

        siteTreeElements.appendChild(video.element);
    }

    // Create the search result element for the playlist inside the DOM, store
    // a reference to it in the siteContent tree
    function initializePlaylist(playlist, ancestors = undefined) {
        const traversal = ancestors ? [...ancestors, playlist] : [];

        playlist.expandToggle = document.createElement('span');

        const spanTitle = document.createElement('span');
        if (ancestors) {
            for (const [index, ancestor] of traversal.entries()) {
                if (index > 0) {
                    const separator = document.createElement('span');
                    separator.appendChild(chevronRightIcon.cloneNode(true));
                    separator.setAttribute('aria-label', '>');

                    spanTitle.appendChild(separator);
                }

                spanTitle.append(ancestor.title);
            }
        } else {
            spanTitle.textContent = playlist.title;
        }

        const spanInfo = document.createElement('span');
        spanInfo.textContent = NAVIGATION_JS_T.xxxVideos(playlist.videos.length);

        playlist.element = document.createElement('a');
        playlist.element.classList.add('playlist');
        playlist.element.href = rootPrefix + playlist.sitePath + indexSuffix;
        playlist.element.appendChild(playlist.expandToggle);
        playlist.element.appendChild(spanTitle);
        playlist.element.appendChild(spanInfo);

        if (playlist.videos.length) {
            playlist.expandToggle.appendChild(chevronRightIcon.cloneNode(true));
            playlist.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
            playlist.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
            playlist.expandToggle.addEventListener('click', event => {
                collapseExpand(playlist.element);
                event.preventDefault();
            });
        }

        if (playlist.sitePath === sitePath) {
            playlist.element.classList.add('current');
        }

        siteTreeElements.appendChild(playlist.element);

        for (const video of playlist.videos) {
            initializeVideo(video);
        }
    }

    // Create the search result element for the collection inside the DOM,
    // store a reference to it in the siteContent tree
    function initializeCollection(collection, ancestors = undefined) {
        const traversal = ancestors ? [...ancestors, collection] : [];

        collection.expandToggle = document.createElement('span');

        const spanTitle = document.createElement('span');
        if (ancestors) {
            for (const [index, ancestor] of traversal.entries()) {
                if (index > 0) {
                    const separator = document.createElement('span');
                    separator.appendChild(chevronRightIcon.cloneNode(true));
                    separator.setAttribute('aria-label', '>');

                    spanTitle.appendChild(separator);
                }

                spanTitle.append(ancestor.title);
            }
        } else {
            spanTitle.textContent = collection.title;
        }

        collection.element = document.createElement('a');
        collection.element.classList.add('collection');
        collection.element.href = rootPrefix + collection.sitePath + indexSuffix;
        collection.element.appendChild(collection.expandToggle);

        collection.element.appendChild(spanTitle);

        if (collection.videos.length) {
            const spanInfo = document.createElement('span');
            spanInfo.textContent = NAVIGATION_JS_T.xxxVideos(collection.videos.length);
            collection.element.appendChild(spanInfo);

            collection.expandToggle.appendChild(chevronRightIcon.cloneNode(true));
            collection.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
            collection.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
            collection.expandToggle.addEventListener('click', event => {
                collapseExpand(collection.element);
                event.preventDefault();
            });
        }

        if (collection.sitePath === sitePath) {
            collection.element.classList.add('current');
        }

        siteTreeElements.appendChild(collection.element);

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
        initializeCollection(siteContent.collection);
    } else /* if (siteContent.playlist) */ {
        initializePlaylist(siteContent.playlist);
    }

    siteTreeInitialized = true;
}

function lqipToRadialGradient(lqip) {
    const [ne, nw, se, sw] = lqip;
    return `radial-gradient(ellipse at top right, rgb(${ne[0]},${ne[1]},${ne[2]}), transparent),` +
        `radial-gradient(ellipse at top left, rgb(${nw[0]},${nw[1]},${nw[2]}), transparent),` +
        `radial-gradient(ellipse at bottom right, rgb(${se[0]},${se[1]},${se[2]}), transparent),` +
        `radial-gradient(ellipse at bottom left, rgb(${sw[0]},${sw[1]},${sw[2]}), transparent)`;
}

function toggleBrowse() {
    function updateVideo(video) {
        const isCurrent = video.element.classList.contains('current');
        video.element.classList.toggle('visible', isCurrent);
    }

    function updatePlaylist(playlist) {
        for (const video of playlist.videos) {
            updateVideo(video);
        }

        playlist.element.classList.add('visible');
        playlist.element.classList.remove('expanded');
        playlist.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
        playlist.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
    }

    function updateCollection(collection) {
        for (const video of collection.videos) {
            updateVideo(video);
        }

        for (const playlist of collection.playlists) {
            updatePlaylist(playlist);
        }

        for (const subcollection of collection.subcollections) {
            updateCollection(subcollection);
        }

        collection.element.classList.add('visible');
        collection.element.classList.remove('expanded');
        collection.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
        collection.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
    }

    siteTreeStatus.textContent = '';

    if (siteTreeElements.dataset.mode === 'browse') {
        siteTree.classList.remove('open');
        siteTreeElements.dataset.mode = null;
    } else {
        if (siteContent.collection) {
            updateCollection(siteContent.collection);
        } else {
            updatePlaylist(siteContent.playlist);
        }

        siteTreeElements.style.setProperty('display', null);
        siteTree.classList.add('open');
        siteTreeElements.dataset.mode = 'browse';

        // TODO: Do we need to announce the browser opening with a custom status/text?
        // siteTreeStatus.setAttribute('aria-label', NAVIGATION_JS_T.showingXxxResultsForXxx(shown, query));
    }
}

function updateSearch() {
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

            playlist.element.classList.remove('expanded');
            playlist.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.collapse);
            playlist.expandToggle.setAttribute('title', NAVIGATION_JS_T.collapse);
            playlist.element.classList.toggle('visible', display);
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

            collection.element.classList.remove('expanded');
            collection.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.collapse);
            collection.expandToggle.setAttribute('title', NAVIGATION_JS_T.collapse);
            collection.element.classList.toggle('visible', display);
        }

        if (siteContent.collection) {
            updateCollection(siteContent.collection);
        } else {
            updatePlaylist(siteContent.playlist);
        }

        if (shown === 0) {
            siteTreeElements.style.setProperty('display', 'none');
            siteTreeStatus.removeAttribute('aria-label');
            siteTreeStatus.textContent = NAVIGATION_JS_T.nothingFoundForXxx(query);
        } else {
            siteTreeElements.style.setProperty('display', null);
            siteTreeStatus.setAttribute('aria-label', NAVIGATION_JS_T.showingXxxResultsForXxx(shown, query));
            siteTreeStatus.textContent = '';
        }

        siteTree.classList.add('open');
    } else {
        siteTree.classList.remove('open');
        siteTreeStatus.removeAttribute('aria-label');
        siteTreeStatus.textContent = '';
    }
}

clearSearchButton.addEventListener('click', clearSearch);

browseButton.addEventListener('focus', initializeSiteTree);
searchInput.addEventListener('focus', initializeSiteTree);

browseButton.addEventListener('click', () => accessSiteTree(toggleBrowse));

searchInput.addEventListener('input', () => {
    searchContainer.classList.toggle('query_present', searchInput.value.length);
    accessSiteTree(updateSearch);
});

navigation.addEventListener('keydown', event => {
    if (event.key === 'ArrowUp') {
        if (siteTree.classList.contains('open')) {
            const visibleResults = [...siteTreeElements.querySelectorAll('a.visible')];

            if (visibleResults.length) {
                const targetRowIndex = visibleResults.indexOf(event.target);

                if (targetRowIndex === -1) {
                    visibleResults[visibleResults.length - 1].focus();
                } else if (targetRowIndex > 0) {
                    visibleResults[targetRowIndex - 1].focus();
                } else if (searchContainer.classList.contains('query_present')) {
                    searchInput.focus();
                } else {
                    visibleResults[visibleResults.length - 1].focus();
                }
            }

            event.preventDefault();
        }
    } else if (event.key === 'ArrowDown') {
        if (siteTree.classList.contains('open')) {
            const visibleResults = [...siteTreeElements.querySelectorAll('a.visible')];

            if (visibleResults.length) {
                const targetRowIndex = visibleResults.indexOf(event.target);

                if (targetRowIndex === -1) {
                    visibleResults[0].focus();
                } else if (targetRowIndex + 1 < visibleResults.length) {
                    visibleResults[targetRowIndex + 1].focus();
                } else if (searchContainer.classList.contains('query_present')) {
                    searchInput.focus();
                } else {
                    visibleResults[0].focus();
                }
            }

            event.preventDefault();
        }
    } else if (event.key === 'Escape') {
        if (searchContainer.classList.contains('query_present')) {
            clearSearch();
            event.preventDefault();
        } else if (siteTreeElements.dataset.mode === 'browse') {
            siteTree.classList.remove('open');
            siteTreeElements.dataset.mode = null;
            browseButton.focus();
        }
    } else if (event.key === ' ') {
        if (event.target.classList.contains('collection') ||
            event.target.classList.contains('playlist')) {
            collapseExpand(event.target);
            event.preventDefault();
        } else if (event.target.classList.contains('video')) {
            event.preventDefault();
        }
    } else if (event.key === 'ArrowLeft') {
        if (event.target.classList.contains('collection') ||
            event.target.classList.contains('playlist') ||
            event.target.classList.contains('video')) {
            collapseExpand(event.target, false);
            event.preventDefault();
        }
    } else if (event.key === 'ArrowRight') {
        if (event.target.classList.contains('collection') ||
            event.target.classList.contains('playlist')) {
            collapseExpand(event.target, true);
            event.preventDefault();
        }
    }
});

// siteTreeElement is usually a collection or playlist that is to be collapsed
// or expanded, but can also be a video if the intent is to collapse its
// parent collection or playlist. The expand parameter dictates whether to
// expand (true), collapse (false) or toggle (undefined) the element (or its
// parent).
function collapseExpand(siteTreeElement, expand = undefined) {
    function visitPlaylist(playlist) {
        if (playlist.element === siteTreeElement) {
            if (expand === undefined) {
                expand = !playlist.element.classList.contains('expanded');
            }

            for (const video of playlist.videos) {
                video.element.classList.toggle('visible', expand);
            }

            playlist.element.classList.toggle('expanded', expand);
            const expandOrCollapse = NAVIGATION_JS_T[expand ? 'collapse' : 'expand'];
            playlist.expandToggle.setAttribute('aria-label', expandOrCollapse);
            playlist.expandToggle.setAttribute('title', expandOrCollapse);

            return true;
        } else if (!expand && playlist.videos.some(video => video.element === siteTreeElement)) {
            for (const video of playlist.videos) {
                video.element.classList.remove('visible');
            }
            playlist.element.classList.remove('expanded');
            playlist.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
            playlist.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
            playlist.element.focus();

            return true;
        }

        return false;
    }

    function visitCollection(collection) {
        if (collection.element === siteTreeElement) {
            if (expand === undefined) {
                expand = !collection.element.classList.contains('expanded');
            }

            for (const video of collection.videos) {
                video.element.classList.toggle('visible', expand);
            }

            collection.element.classList.toggle('expanded', expand);
            const expandOrCollapse = NAVIGATION_JS_T[expand ? 'collapse' : 'expand'];
            collection.expandToggle.setAttribute('aria-label', expandOrCollapse);
            collection.expandToggle.setAttribute('title', expandOrCollapse);

            return true;
        } else if (!expand && collection.videos.some(video => video.element === siteTreeElement)) {
            for (const video of collection.videos) {
                video.element.classList.remove('visible');
            }
            collection.element.classList.remove('expanded');
            collection.expandToggle.setAttribute('aria-label', NAVIGATION_JS_T.expand);
            collection.expandToggle.setAttribute('title', NAVIGATION_JS_T.expand);
            collection.element.focus();

            return true;
        } else {
            for (const playlist of collection.playlists) {
                if (visitPlaylist(playlist)) {
                    return true;
                }
            }

            for (const subcollection of collection.subcollections) {
                if (visitCollection(subcollection)) {
                    return true;
                }
            }
        }

        return false;
    }

    if (siteContent.collection) {
        visitCollection(siteContent.collection);
    } else {
        visitPlaylist(siteContent.playlist);
    }
}

window.addEventListener('keydown', event => {
    if (event.target === searchInput) return;

    if (event.key === 'b') {
        browseButton.focus();
        accessSiteTree(toggleBrowse);
        event.preventDefault();
    } else if (event.key === 's') {
        searchInput.focus();
        event.preventDefault();
    }
});
