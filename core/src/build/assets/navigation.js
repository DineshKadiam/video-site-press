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

            // Thumbnails in the site navigation tree are currently treated as
            // decorative elements (hidden from screen readers). Some not yet
            // conclusive considerations for this:
            // - The navigation serves primarily as a quick way to navigate
            //   the page, putting thumbnails out of the way for screeen
            //   readers might better fulfill that purpose (?)
            // - For sighted users thumbnails are shown at a very small size,
            //   which also already goes into decorative usage rather than
            //   conveying actual detailed insights into the content, so this
            //   possibly mirrors that well for screen readers
            // - It could as well be argued that for screen reader users we
            //   could provide a better experience by actually including the
            //   image descriptions here, providing additional insight that
            //   sighted users can not get due to the requirement for very
            //   small thumbnails in the limited navigation space – this can
            //   absolutely be re-evaluated
            image.ariaHidden = 'true';

            image.loading = 'lazy';
            image.src = window.hyper8.rootPrefix + video.sitePath + video.image;

            if (video.lqip) {
                image.style.background = lqipToRadialGradient(video.lqip);
            }
        } else {
            image = document.createElement('span');
            image.classList.add('placeholder');
        }

        const divTitleDetails = document.createElement('div');

        const spanTitle = document.createElement('span');
        spanTitle.textContent = video.title;
        divTitleDetails.appendChild(spanTitle);

        const spanDetails = document.createElement('span');
        if (video.date) {
            if (video.duration) {
                spanDetails.textContent = `${video.date}, ${video.duration}`;
            } else {
                spanDetails.textContent = video.date;
            }
        } else if (video.duration) {
            spanDetails.textContent = video.duration;
        }
        divTitleDetails.appendChild(spanDetails);

        video.element = document.createElement('a');
        video.element.classList.add('video');
        video.element.href = window.hyper8.rootPrefix + video.sitePath + indexSuffix;
        video.element.appendChild(image);
        video.element.appendChild(divTitleDetails);

        if (video.sitePath === window.hyper8.sitePath) {
            video.element.classList.add('current');
        }

        siteTreeElements.appendChild(video.element);
    }

    // Create the search result element for this container (collection or
    // playlist) inside the DOM and store a reference to it in the siteContent
    // tree
    function initializeContainer(container, ancestors = undefined) {
        const traversal = ancestors ? [...ancestors, container] : [];

        container.expandToggle = document.createElement('span');

        const spanTitle = document.createElement('span');
        if (ancestors) {
            for (const [index, ancestor] of traversal.entries()) {
                if (index > 0) {
                    const separator = document.createElement('span');
                    separator.appendChild(chevronRightIcon.cloneNode(true));
                    // TODO: Clarify what label works best for screen reader
                    // users to succinctly express "separator indicating
                    // going down the hierarchy" (respectively use a semantic
                    // construct around the "breadcrumbs" in which the
                    // separators are hidden from screenreaders instead).
                    separator.setAttribute('aria-label', '>');

                    spanTitle.appendChild(separator);
                }

                spanTitle.append(ancestor.title);
            }
        } else {
            spanTitle.textContent = container.title;
        }

        container.element = document.createElement('a');
        container.element.classList.add('container');
        container.element.href = window.hyper8.rootPrefix + container.sitePath + indexSuffix;
        container.element.appendChild(container.expandToggle);
        container.element.appendChild(spanTitle);

        // If there are one or more videos, or if there is absolutely nothing at all
        // in the collection or playlist, we display the number of videos
        if (container.videos.length || !container.containers?.length) {
            const spanInfo = document.createElement('span');
            spanInfo.textContent = t.xxxVideos(container.videos.length);
            container.element.appendChild(spanInfo);
        }

        if (container.videos.length) {
            container.expandToggle.appendChild(chevronRightIcon.cloneNode(true));
            container.expandToggle.setAttribute('aria-label', t.expand);
            container.expandToggle.setAttribute('title', t.expand);
            container.expandToggle.addEventListener('click', event => {
                collapseExpand(container.element);
                event.preventDefault();
            });
        }

        if (container.sitePath === window.hyper8.sitePath) {
            container.element.classList.add('current');
        }

        siteTreeElements.appendChild(container.element);

        for (const video of container.videos) {
            initializeVideo(video);
        }

        if (container.containers) {
            for (const subcontainer of container.containers) {
                initializeContainer(subcontainer, traversal);
            }
        }
    }

    initializeContainer(siteContent);

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
    function updateContainer(container) {
        for (const video of container.videos) {
            updateVideo(video);
        }

        if (container.containers) {
            for (const subcontainer of container.containers) {
                updateContainer(subcontainer);
            }
        }

        container.element.classList.add('visible');
        container.element.classList.remove('expanded');
        container.expandToggle.setAttribute('aria-label', t.expand);
        container.expandToggle.setAttribute('title', t.expand);
    }

    function updateVideo(video) {
        const isCurrent = video.element.classList.contains('current');
        video.element.classList.toggle('visible', isCurrent);
    }

    siteTreeStatus.textContent = '';

    if (siteTreeElements.dataset.mode === 'browse') {
        siteTree.classList.remove('open');
        delete siteTreeElements.dataset.mode;
    } else {
        updateContainer(siteContent);

        siteTreeElements.style.setProperty('display', null);
        siteTree.classList.add('open');
        siteTreeElements.dataset.mode = 'browse';

        // TODO: Do we need to announce the browser opening with a custom status/text?
        // siteTreeStatus.setAttribute('aria-label', t.showingXxxResultsForXxx(shown, query));
    }
}

function updateSearch() {
    const query = searchInput.value.trim();

    if (query.length) {
        const regexp = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');

        // If the query is four digits we additionally try to match it against
        // video dates.
        const yearQuery = /^\d{4}$/.test(query);

        let shown = 0;

        function updateContainer(container) {
            let display = regexp.test(container.title);
            if (display) { shown += 1; }

            for (const video of container.videos) {
                display = updateVideo(video) || display;
            }

            if (container.containers) {
                for (const subcontainer of container.containers) {
                    updateContainer(subcontainer);
                }
            }

            container.element.classList.remove('expanded');
            container.expandToggle.setAttribute('aria-label', t.collapse);
            container.expandToggle.setAttribute('title', t.collapse);
            container.element.classList.toggle('visible', display);
        }

        function updateVideo(video) {
            const display = regexp.test(video.title) ||
                !!(yearQuery && video.date && video.date.includes(query));
            video.element.classList.toggle('visible', display);
            if (display) { shown += 1; }
            return display;
        }

        updateContainer(siteContent);

        if (shown === 0) {
            siteTreeElements.style.setProperty('display', 'none');
            siteTreeStatus.removeAttribute('aria-label');
            siteTreeStatus.textContent = t.nothingFoundForXxx(query);
        } else {
            siteTreeElements.style.setProperty('display', null);
            siteTreeStatus.setAttribute('aria-label', t.showingXxxResultsForXxx(shown, query));
            siteTreeStatus.textContent = '';
        }

        siteTree.classList.add('open');
    } else {
        siteTree.classList.remove('open');
        siteTreeStatus.removeAttribute('aria-label');
        siteTreeStatus.textContent = '';
    }

    delete siteTreeElements.dataset.mode;
}

clearSearchButton.addEventListener('click', clearSearch);

browseButton.addEventListener('focus', initializeSiteTree);
searchInput.addEventListener('focus', initializeSiteTree);

browseButton.addEventListener('click', () => accessSiteTree(toggleBrowse));

searchInput.addEventListener('input', () => {
    searchContainer.classList.toggle('query_present', searchInput.value.length);
    accessSiteTree(updateSearch);
});

function handleNavigationKeydown(event) {
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
            delete siteTreeElements.dataset.mode;
            browseButton.focus();
            event.preventDefault();
        }
    } else if (event.key === ' ') {
        if (event.target.classList.contains('container')) {
            collapseExpand(event.target);
            event.preventDefault();
        } else if (event.target.classList.contains('video')) {
            event.preventDefault();
        }
    } else if (event.key === 'ArrowLeft') {
        if (event.target.classList.contains('container') ||
            event.target.classList.contains('video')) {
            collapseExpand(event.target, false);
            event.preventDefault();
        }
    } else if (event.key === 'ArrowRight') {
        if (event.target.classList.contains('container')) {
            collapseExpand(event.target, true);
            event.preventDefault();
        }
    }

    // Prevent this from being picked up by other, global keydown handlers
    event.stopPropagation();
}

navigation.addEventListener('keydown', handleNavigationKeydown);
siteTree.addEventListener('keydown', handleNavigationKeydown);

// siteTreeElement is usually a collection or playlist that is to be collapsed
// or expanded, but can also be a video if the intent is to collapse its
// parent collection or playlist. The expand parameter dictates whether to
// expand (true), collapse (false) or toggle (undefined) the element (or its
// parent).
function collapseExpand(siteTreeElement, expand = undefined) {
    function visitContainer(container) {
        if (container.element === siteTreeElement) {
            if (expand === undefined) {
                expand = !container.element.classList.contains('expanded');
            }

            for (const video of container.videos) {
                video.element.classList.toggle('visible', expand);
            }

            container.element.classList.toggle('expanded', expand);
            const expandOrCollapse = t[expand ? 'collapse' : 'expand'];
            container.expandToggle.setAttribute('aria-label', expandOrCollapse);
            container.expandToggle.setAttribute('title', expandOrCollapse);

            return true;
        } else if (!expand && container.videos.some(video => video.element === siteTreeElement)) {
            for (const video of container.videos) {
                video.element.classList.remove('visible');
            }
            container.element.classList.remove('expanded');
            container.expandToggle.setAttribute('aria-label', t.expand);
            container.expandToggle.setAttribute('title', t.expand);
            container.element.focus();

            return true;
        } else if (container.containers) {
            for (const subcontainer of container.containers) {
                if (visitContainer(subcontainer)) {
                    return true;
                }
            }
        }

        return false;
    }

    visitContainer(siteContent);
}

// Register global keyboard shortcut handlers:
// - [b] Browse
// - [s] Search
window.addEventListener('keydown', event => {
    // Avoid conflicts with browser/system shortcuts where possible
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
        return;
    }

    if (event.key === 'b' || event.key === 'B') {
        browseButton.focus();
        accessSiteTree(toggleBrowse);
        event.preventDefault();
    } else if (event.key === 's' || event.key === 'S') {
        searchInput.focus();
        event.preventDefault();
    }
});
