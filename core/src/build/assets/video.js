const playerWrapper = document.querySelector('.player_wrapper');
const togglePlaylistButton = document.querySelector('#toggle_playlist');
const toggleTranscriptButton = document.querySelector('#toggle_transcript');

// If there are no media files, or if all of them have been restricted for
// offline usage, we don't have a player on the page (only a video
// placeholder). In this case we need to skip almost all of the
// initialization.
if (playerWrapper) {
    const maximizeIcon = document.querySelector('#maximize_icon').content;
    const minimizeIcon = document.querySelector('#minimize_icon').content;
    const playIcon = document.querySelector('#play_icon').content;
    const pauseIcon = document.querySelector('#pause_icon').content;
    const replayIcon = document.querySelector('#replay_icon').content;
    const subtitlesIcon = document.querySelector('#subtitles_icon').content;
    const volumeLoudIcon = document.querySelector('#volume_loud_icon').content;
    const volumeMediumIcon = document.querySelector('#volume_medium_icon').content;
    const volumeMutedIcon = document.querySelector('#volume_muted_icon').content;

    const cycleVersionButton = document.querySelector('#cycle_versions');
    const filterTranscriptInput = document.querySelector('.transcript input');
    const playbackCycleVolumeButton = document.querySelector('.left_playback_controls .cycle_volume');
    const playbackSubtitlesSelectorControl = document.querySelector('.right_playback_controls .selector_control.subtitles');
    const playbackToggleFullscreenButton = document.querySelector('.right_playback_controls .toggle_fullscreen');
    const playerControls = document.querySelector('.player_controls');
    const replayButton = document.querySelector('#replay');
    const searchContainer = document.querySelector('.search');
    const speedControl = document.querySelector('#speed');
    const speedMultiplier = document.querySelector('#speed .multiplier');
    const timeline = document.querySelector('.timeline');
    const timelineInput = document.querySelector('.timeline input');
    const togglePlaybackButton = document.querySelector('#toggle_playback');
    const versionSelectorControl = document.querySelector('.selector_control.version');
    const versionSlices = document.querySelectorAll('.versions_ring g[data-src]');
    const versionsRing = document.querySelector('.versions_ring');
    const video = document.querySelector('video');

    // Before we even begin playback video.duration is NaN, so we use our own data throughout.
    const duration = parseFloat(video.dataset.duration);

    let playbackUpdateInterval;

    const fullscreen = {
        // On iOS on iPhone, video playback must be started at least once before
        // entering fullscreen on the video element works, therefore we track
        // this state, and start playback when fullscreen is requested without a
        // prior playback request.
        alreadyPlayed: false,
        // If requestFullscreen is available on the player wrapper (a non-video
        // element), we know that we are on a device and browser that supports
        // the generally agreed upon fullscreen API for browsers. If we aren't,
        // it's reasonable to assume we are in the one environment that
        // (intentionally) doesn't - iOS on iPhone - and we consecutively employ
        // special handling for it.
        // See:
        // - https://caniuse.com/?search=requestfullscreen
        //   (General overview of support across browsers)
        // - https://developer.apple.com/forums/thread/133248
        //   (Timeline of apple failing to support requestFullscreen on iOS since 2020)
        // - https://developer.apple.com/documentation/webkitjs/htmlvideoelement/1633500-webkitenterfullscreen
        //   (Apple developer documentation for their non-standard fullscreen API)
        iPhoneMode: !playerWrapper.requestFullscreen
    };

    // We internally manage speed as (integer) percent values to avoid having to
    // deal with float rounding issues.
    let speed = 100;

    const volume = {
        interactive: false,
        value: 1
    };

    function formatTime(time) {
        const timeFloored = Math.floor(time);

        if (timeFloored >= 3600) {
            const hours = Math.floor(timeFloored / 3600).toString();
            const minutes = Math.floor((timeFloored % 3600) / 60).toString().padStart(2, '0');
            const seconds = (timeFloored % 60).toString().padStart(2, '0');
            return `${hours}:${minutes}:${seconds}`;
        } else if (timeFloored >= 60) {
            const minutes = Math.floor(timeFloored / 60).toString().padStart(2, '0');
            const seconds = (timeFloored % 60).toString().padStart(2, '0');
            return `${minutes}:${seconds}`;
        } else {
            const seconds = (timeFloored % 60).toString().padStart(2, '0');
            return `00:${seconds}`;
        }
    }

    function initializeVolume() {
        const persistedVolume = localStorage.getItem('hyper8Volume');
        if (persistedVolume !== null) {
            volume.value = parseFloat(persistedVolume);
            updateVolume();
        }

        // When a page loads we start with the assumption that volume is read-only,
        // but immediately run an asynchronous routine to determine if volume is
        // adjustable - if it is we register this in our global volume object and
        // restore any fine-grained volume adjustments that we previously persisted.
        // Also, by the time the visitor interacts with the volume control we know
        // whether to offer fine-grained volume control or just mute/unmute
        // functionality. The reason for this quirky stuff is that Apple's iOS
        // devices intentionally don't allow application-level volume control and
        // therefore the web audio API on these devices features a read-only volume
        // property on audio elements (the muted property however still works there
        // and we use it).
        let volumeProbe = new Audio();
        const volumeProbeHandler = () => {
            volume.interactive = true;
            updateVolume();

            volumeProbe.removeEventListener('volumechange', volumeProbeHandler);
            volumeProbe = null;
        };
        volumeProbe.addEventListener('volumechange', volumeProbeHandler);
        volumeProbe.volume = 0.123;
    }

    initializeVolume();

    function playbackUpdate() {
        const time = video.currentTime;
        const timeFormatted = formatTime(time);

        timelineInput.value = time;
        timelineInput.setAttribute('aria-valuenow', time);
        timelineInput.setAttribute('aria-valuetext', timeFormatted);
        document.querySelector('.bar.progress').style.width = `${(time / duration) * 100}%`;
        document.querySelector('#time_current').innerHTML = timeFormatted;

        if (playerControls.dataset.hideBy && (new Date()).getTime() > playerControls.dataset.hideBy) {
            playerControls.classList.add('hidden');
        }
    }

    function revealControls() {
        playerControls.dataset.hideBy = (new Date()).getTime() + 3000;
        playerControls.classList.remove('hidden');
    }

    versionSelectorControl.addEventListener('mouseenter', () => {
        versionSelectorControl.classList.add('active');
    });

    versionSelectorControl.addEventListener('mouseleave', () => {
        versionSelectorControl.classList.remove('active');
    });

    const versionButtons = versionSelectorControl.querySelectorAll('.selector_options button');

    const versionOptions = [];
    for (const [index, button] of versionButtons.entries()) {
        const option = { button };
        option.slice = [...versionSlices].find(slice => slice.dataset.src === button.dataset.src)
        versionOptions.push(option);
    }

    let activeVersionOption = versionOptions.find(option =>
        option.button.dataset.src === video.getAttribute('src')
    );
    activeVersionOption.button.classList.add('active');

    // Receives the slice group from the version ring. We always initially pass
    // through this function when the video is first started, so we also use it
    // to remove the poster (because this would otherwise briefly flash into
    // being visible when resolution is changed on-the-go).
    function selectVersion(slice) {
        video.removeAttribute('poster');

        const option = versionOptions.find(option => option.slice === slice);
        activateVersionOption(option);
    }

    function activateVersionOption(option) {
        if (activeVersionOption !== null) {
            activeVersionOption.button.classList.remove('active');
        }

        if (!video.paused) {
            togglePlayback();
        }

        const currentTimeMemorized = video.currentTime;
        video.src = option.button.dataset.src;
        video.currentTime = currentTimeMemorized;
        cycleVersionButton.textContent = option.button.dataset.label;

        togglePlayback();

        option.button.classList.add('active');
        activeVersionOption = option;
    }

    for (const option of versionOptions) {
        option.button.addEventListener('click', () => {
            if (option !== activeVersionOption) {
                activateVersionOption(option);
            }
            versionSelectorControl.classList.remove('active');
            option.button.blur();
        });
    }

    cycleVersionButton.addEventListener('click', () => {
        const activeVersionOptionIndex = versionOptions.indexOf(activeVersionOption);
        activateVersionOption(versionOptions[(activeVersionOptionIndex + 1) % versionOptions.length]);
    });

    function cycleVolume() {
        if (volume.interactive) {
            if (volume.value > 0.666) {
                volume.value = 0;
            } else if (volume.value > 0.333) {
                volume.value = 1;
            } else {
                volume.value = 0.5;
            }
        } else {
            if (volume.value > 0.5) {
                volume.value = 0;
            } else {
                volume.value = 1;
            }
        }

        updateVolume();
        localStorage.setItem('hyper8Volume', volume.value);
    }

    function toggleFullscreen() {
        if (fullscreen.iPhoneMode) {
            if (video.webkitDisplayingFullscreen) {
                video.webkitExitFullscreen();
            } else {
                if (!fullscreen.alreadyPlayed) {
                    video.play();
                }
                video.webkitEnterFullscreen();
            }
        } else {
            if (document.fullscreenElement) {
                document.exitFullscreen();
            } else {
                playerWrapper.requestFullscreen();
            }
        }
    }

    function togglePlayback() {
        if (video.paused) {
            video.play();
        } else {
            video.pause();
        }
    }

    video.addEventListener('pause', () => {
        togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
        clearInterval(playbackUpdateInterval);
    });

    video.addEventListener('play', () => {
        fullscreen.alreadyPlayed = true;
        playerWrapper.dataset.state = 'playing';
        togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
        // TODO: Dynamic, small interval when the video is really short, otherwise can be a fixed higher value
        //       Formula should/can take into account width of video on screen vs duration of video.
        playbackUpdateInterval = setInterval(playbackUpdate, 80);
    });

    function toggleVolume() {
        if (volume.value > 0) {
            volume.valueBeforeMute = volume.value;
            volume.value = 0;
        } else if (volume.valueBeforeMute !== undefined) {
            volume.value = volume.valueBeforeMute;
            delete volume.valueBeforeMute;
        } else {
            volume.value = 1;
        }

        updateVolume();
        localStorage.setItem('hyper8Volume', volume.value);
    }

    function updateVolume() {
        let volumeIcon;

        if (volume.interactive) {
            if (volume.value > 0.666) {
                video.muted = false;
                video.volume = 1;
                volumeIcon = volumeLoudIcon;
            } else if (volume.value > 0.333) {
                video.muted = false;
                video.volume = 0.5;
                volumeIcon = volumeMediumIcon;
            } else {
                video.muted = true;
                video.volume = 0;
                volumeIcon = volumeMutedIcon;
            }
        } else {
            if (volume.value > 0.5) {
                video.muted = false;
                volumeIcon = volumeLoudIcon
            } else {
                video.muted = true;
                volumeIcon = volumeMutedIcon;
            }
        }

        playbackCycleVolumeButton.classList.toggle('dimmed', video.muted);
        playbackCycleVolumeButton.replaceChildren(volumeIcon.cloneNode(true));
    }

    for (const slice of versionSlices) {
        slice.addEventListener('click', () => selectVersion(slice));

        slice.addEventListener('focus', () => {
            versionsRing.classList.add('interacting');
        });

        slice.addEventListener('blur', () => {
            versionsRing.classList.remove('interacting');
        });

        slice.addEventListener('keypress', event =>  {
            if (event.key === 'Enter' || event.key === ' ') {
                selectVersion(slice);
                event.preventDefault();
            }
        });

        slice.addEventListener('mouseenter', () => {
            versionsRing.classList.add('interacting');
        });

        slice.addEventListener('mouseleave', () => {
            versionsRing.classList.remove('interacting');
        });
    }

    window.addEventListener('keydown', revealControls);

    playbackCycleVolumeButton.addEventListener('click', cycleVolume);
    playbackToggleFullscreenButton.addEventListener('click', toggleFullscreen);

    if (fullscreen.iPhoneMode) {
        video.addEventListener('webkitbeginfullscreen', () => {
            playbackToggleFullscreenButton.replaceChildren(minimizeIcon.cloneNode(true));
        });
        video.addEventListener('webkitendfullscreen', () => {
            playbackToggleFullscreenButton.replaceChildren(maximizeIcon.cloneNode(true));
        });
    } else {
        playerWrapper.addEventListener('fullscreenchange', () => {
            const fullscreenIcon = document.fullscreenElement ? minimizeIcon : maximizeIcon;
            playbackToggleFullscreenButton.replaceChildren(fullscreenIcon.cloneNode(true));
        });
    }

    playerWrapper.addEventListener('mousemove', revealControls);

    // Unlisted videos are not included in the playlist listing in the player
    // outro screen, therefore there is no separate replay button for these
    // videos either.
    if (replayButton) {
        replayButton.addEventListener('click', event => {
            event.preventDefault();
            video.play();
        });
    }

    if (playbackSubtitlesSelectorControl) {
        const playbackCycleSubtitlesButton = document.querySelector('.right_playback_controls .cycle_subtitles');
        const playbackSubtitleButtons = playbackSubtitlesSelectorControl.querySelectorAll('.selector_options button');

        playbackSubtitlesSelectorControl.addEventListener('mouseenter', () => {
            playbackSubtitlesSelectorControl.classList.add('active');
        });

        playbackSubtitlesSelectorControl.addEventListener('mouseleave', () => {
            playbackSubtitlesSelectorControl.classList.remove('active');
        });

        const subtitleOptions = [];
        for (const [index, button] of playbackSubtitleButtons.entries()) {
            const option = { button };

            if (index > 0) {
                option.track = video.textTracks[index - 1];
            }

            subtitleOptions.push(option);
        }

        const noSubtitleOption = subtitleOptions[0];
        let activeSubtitleOption = noSubtitleOption;

        function activateOption(option) {
            // Reset previously active option
            activeSubtitleOption.button.classList.remove('active');
            if (activeSubtitleOption.track) {
                activeSubtitleOption.track.mode = 'disabled';
            }

            // Activate new option
            playbackCycleSubtitlesButton.classList.toggle('dimmed', option === noSubtitleOption);
            option.button.classList.add('active');
            if (option.track) {
                option.track.mode = 'showing';
            }
            activeSubtitleOption = option;
        }

        for (const option of subtitleOptions) {
            option.button.addEventListener('click', () => {
                if (option !== activeSubtitleOption) {
                    activateOption(option);
                }
                playbackSubtitlesSelectorControl.classList.remove('active');
                option.button.blur();
            });
        }

        function cycleSubtitles() {
            const activeSubtitleOptionIndex = subtitleOptions.indexOf(activeSubtitleOption);
            activateOption(subtitleOptions[(activeSubtitleOptionIndex + 1) % subtitleOptions.length]);
        }

        playbackCycleSubtitlesButton.addEventListener('click', cycleSubtitles);
    }

    // Filter transcript handler

    if (filterTranscriptInput) {
        filterTranscriptInput.addEventListener('input', () => {
            const query = filterTranscriptInput.value.trim();

            const entries = document.querySelector('.transcript .entries')

            if (query.length) {
                const regexp = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');

                for (const entry of entries.children) {
                    const display = regexp.test(entry.children[0].textContent);
                    entry.style.setProperty('display', display ? null : 'none');
                }
            } else {
                for (const entry of entries.children) {
                    entry.style.setProperty('display', null);
                }
            }
        });
    }

    // Speed button handlers

    function updateSpeed() {
        speedMultiplier.textContent = (speed / 100).toFixed(1);
        // Our internal speed representation is in percent so we translate to a
        // multiplication factor here
        video.playbackRate = speed / 100;
    }

    speedControl.addEventListener('auxclick', event => {
        speed = 100;
        updateSpeed();
        event.preventDefault();
    });

    speedControl.addEventListener('click', () => {
        if (speed < 100) {
            speed = 100;
        } else if (speed < 120) {
            speed = 120;
        } else if (speed < 140) {
            speed = 140;
        } else if (speed < 160) {
            speed = 160;
        } else if (speed < 180) {
            speed = 180;
        } else if (speed < 200) {
            speed = 200;
        } else {
            speed = 100;
        }

        updateSpeed();
    });

    // Prevent context menu opening when using right-click to reset speed
    speedControl.addEventListener('contextmenu', event => event.preventDefault());

    speedControl.addEventListener('keydown', event => {
        if (event.key === 'ArrowDown' && speed > 30) {
            speed -= 10;
        } else if (event.key === 'ArrowUp' && speed < 300) {
            speed += 10;
        } else {
            return;
        }

        updateSpeed();

        event.preventDefault();
        event.stopPropagation();
    });

    speedControl.addEventListener('wheel', event => {
        if (event.deltaY < 0 && speed < 300) {
            speed += 10;
        } else if (event.deltaY > 0 && speed > 30) {
            speed -= 10;
        }

        updateSpeed();

        event.preventDefault();
    });

    togglePlaybackButton.addEventListener('click', togglePlayback);

    if (toggleTranscriptButton) {
        const videoLayout = document.querySelector('.video_layout');

        function toggleTranscript() {
            if (videoLayout.dataset.panel === 'transcript') {
                toggleTranscriptButton.classList.remove('active');
                delete videoLayout.dataset.panel;
            } else {
                if (togglePlaylistButton) {
                    togglePlaylistButton.classList.remove('active');
                }

                toggleTranscriptButton.classList.add('active');
                videoLayout.dataset.panel = 'transcript';
            }
        }

        toggleTranscriptButton.addEventListener('click', toggleTranscript);
    }

    video.addEventListener('ended', () => {
        playerWrapper.dataset.state = 'outro';
        togglePlaybackButton.replaceChildren(replayIcon.cloneNode(true));
    });

    video.addEventListener('click', togglePlayback);

    window.addEventListener('keydown', event => {
        if (
            searchContainer?.contains(event.target) ||
            event.target === filterTranscriptInput ||
            event.altKey ||
            event.ctrlKey
        ) {
            return;
        }

        if (event.key === ' ') {
            togglePlayback();
            event.preventDefault();
        } else if (event.key === 'f' || event.key === 'F') {
            toggleFullscreen();
            event.preventDefault();
        } else if (event.key === 'm' || event.key === 'M') {
            toggleVolume();
            event.preventDefault();
        } else if (event.key === 'ArrowLeft') {
            if (video.currentTime - 5 < 0) {
                seekTo(0);
            } else {
                seekTo(video.currentTime - 5);
            }
            event.preventDefault();
        } else if (event.key === 'ArrowRight') {
            if (video.currentTime + 5 > duration) {
               seekTo(duration);
            } else {
                seekTo(video.currentTime + 5);
            }
            event.preventDefault();
        }
    });

    timeline.addEventListener('click', () => {
        const bounds = timeline.getBoundingClientRect();
        const factor = (event.clientX - bounds.x) / bounds.width;
        const newTime = factor * timelineInput.max;
        seekTo(newTime);
        timeline.classList.add('focus_from_click');
        timelineInput.focus();
    });

    timelineInput.addEventListener('blur', () => {
        timeline.classList.remove('focus', 'focus_from_click');
    });

    timelineInput.addEventListener('focus', () => {
        timeline.classList.add('focus');
    });

    const resizeObserver = new ResizeObserver(() => {
        const BACKDROP_PADDING = 5;

        for (const slice of versionSlices) {
            let minX;
            let minY;
            let maxX;
            let maxY;

            for (const text of slice.querySelectorAll('text')) {
                const svgRect = text.getBBox();
                if (minX === undefined || svgRect.x < minX) { minX = svgRect.x; }
                if (minY === undefined || svgRect.y < minY) { minY = svgRect.y; }
                if (maxX === undefined || svgRect.x + svgRect.width > maxX) { maxX = svgRect.x + svgRect.width; }
                if (maxY === undefined || svgRect.y + svgRect.height > maxY) { maxY = svgRect.y + svgRect.height; }
            }

            const backdrop = slice.querySelector('rect.backdrop');

            backdrop.setAttribute('x', minX - BACKDROP_PADDING);
            backdrop.setAttribute('y', minY - BACKDROP_PADDING);
            backdrop.setAttribute('width', BACKDROP_PADDING + (maxX - minX) + BACKDROP_PADDING);
            backdrop.setAttribute('height', BACKDROP_PADDING + (maxY - minY) + BACKDROP_PADDING);
        }
    });

    // Trigger version slice backdrop recomputation on resize (this also triggers
    // the first draw after the initial page load).
    resizeObserver.observe(playerWrapper);

    // Parses (and validates) a time parameter from the current url
    // (e.g. https://example.com/#time=4m12s) and returns it within an object
    // (e.g. { time: 252 }). Time can be specified as t=60 or time=60, but also
    // supports complex specifiers like t=1h, t=1m t=1s, t=1h1m, t=1h1m1s, etc.
    // In case of no known params being present or errors being encountered
    // (wrong syntax for params, out-of-bound timecodes, etc.) null is returned.
    function parseHashParams() {
        if (location.hash.length === 0) return null;

        const params = new URLSearchParams(location.hash.substring(1));

        const timeParam = params.get('t') ?? params.get('time');

        if (timeParam === null) return null;

        const result = {};

        // Match all of "", "1", "1ms", "1s", "1m" "1h" "1m1s", "1h1m1s", "1h1s", "1h1m", etc.
        const match = timeParam.match(/^(?:([0-9]+)h)?(?:([0-9]+)m)?(?:([0-9]+)s)?(?:([0-9]+)ms?)?(?:([0-9]+))?$/);

        if (match) {
            result.time = 0;

            // (h)our
            // (m)inute
            // (s)econd
            // (m…)illi(…s)econds
            // (u)nit-less
            const [_, h, m, s, ms, u] = match;

            if (h) { result.time += parseInt(h) * 3600; }
            if (m) { result.time += parseInt(m) * 60; }
            if (s) { result.time += parseInt(s); }
            if (ms) { result.time += parseInt(ms) / 1000; }

            if (u) {
                if (ms) {
                    // A unit-less number given after milliseconds is nonsensical
                    return null;
                } else if (s) {
                    // A unit-less number given after seconds is interpreted as milliseconds
                    result.time += parseInt(u) / 1000;
                } else if (m) {
                    // A unit-less number given after minutes is interpreted as seconds
                    result.time += parseInt(u);
                } else if (h) {
                    // A unit-less number given after the hour is interpreted as minutes
                    result.time += parseInt(u) * 60;
                } else {
                    // A unit-less number given on its own is interpreted as seconds
                    result.time += parseInt(u);
                }
            }

            if (result.time > duration) {
                return null;
            }
        } else {
            return null;
        }

        return result;
    }

    // Seek the video to a new timecode, reflect the change in the
    // timeline bar, update the playback button icon if needed
    function seekTo(newTime, andPlay = false) {
        video.currentTime = newTime;
        playbackUpdate();

        if (andPlay && video.paused) {
            togglePlayback();
        } else {
            // If we move the playback position backwards while we're on the outro
            // screen we need to make sure the replay icon gets replaced by a
            // play icon again.
            if (playerWrapper.dataset.state === 'outro' &&
                video.currentTime < duration - Number.EPSILON) {
                playerWrapper.dataset.state = 'playing';
                togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
            }
        }
    }

    // Seek to a given timecode if it was specified in a hash parameter
    const params = parseHashParams();
    if (params) {
        if (params.time !== undefined) {
            seekTo(params.time);
        }
    }

    window.addEventListener('hashchange', event => {
        const params = parseHashParams();
        if (params) {
            if (params.time !== undefined) {
                seekTo(params.time, true);
                playerWrapper.focus();
            }

            // TODO: This can be observed as a brief flicker in the address bar
            // (when you look for it). Possibly look for a more elegant way in
            // the future.
            history.replaceState(null, '', ' ');
            event.preventDefault();
        }
    });
}

if (togglePlaylistButton) {
    const videoLayout = document.querySelector('.video_layout');
    const hidePlaylistStorageKey = `hyper8HidePlaylist:${togglePlaylistButton.dataset.sitePath}`;

    function togglePlaylist() {
        if (videoLayout.dataset.panel === 'playlist') {
            sessionStorage.setItem(hidePlaylistStorageKey, '')
            togglePlaylistButton.classList.remove('active');
            delete videoLayout.dataset.panel;
        } else {
            if (toggleTranscriptButton) {
                toggleTranscriptButton.classList.remove('active');
            }

            sessionStorage.removeItem(hidePlaylistStorageKey);
            togglePlaylistButton.classList.add('active');
            videoLayout.dataset.panel = 'playlist';
        }
    }

    togglePlaylistButton.addEventListener('click', togglePlaylist);
}