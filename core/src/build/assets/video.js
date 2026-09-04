const player = document.querySelector('.player');
const togglePlaylistButton = document.querySelector('#toggle_playlist');
const toggleTranscriptButton = document.querySelector('#toggle_transcript');

// If there are no media files, or if all of them have been restricted for
// offline usage, we don't have a player on the page (only a video
// placeholder). In this case we need to skip almost all of the
// initialization.
if (player) {
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
    const togglePlaybackButton = document.querySelector('.toggle_playback');
    const versionSelectorControl = document.querySelector('.selector_control.version');
    const versionSlices = document.querySelectorAll('.versions_ring g[data-src]');
    const versionsRing = document.querySelector('.versions_ring');
    const video = document.querySelector('video');

    // Before we even begin playback video.duration is NaN, so we use our own data throughout.
    const duration = parseFloat(video.dataset.duration);

    let bufferedRangeUpdateInterval = null;
    // Although multiple, separate buffered ranges can exist for a video at
    // runtime, we only ever display the buffered range that overlaps and
    // expands rightwards from the current playback point. Leftwards of the
    // current playback point this range is always visually obscured by the
    // progress bar, so we only need to dynamically adjust the end of this
    // buffered range
    let lastBufferedRangeEnd = 0;

    let onPauseStateOverride = null;

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
        iPhoneMode: !player.requestFullscreen
    };

    const seekState = {};

    // We internally manage speed as (integer) percent values to avoid having to
    // deal with float rounding issues.
    let speed = 100;

    let playerInit = false;

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

            volumeProbe = null;
        };
        volumeProbe.addEventListener('volumechange', volumeProbeHandler, { once: true });
        volumeProbe.volume = 0.123;
    }

    initializeVolume();

    function playbackUpdate(forcedTime = null) {
        const time = forcedTime ?? currentTime();
        const timeFormatted = formatTime(time);

        timelineInput.value = time;
        timelineInput.setAttribute('aria-valuenow', time);
        timelineInput.setAttribute('aria-valuetext', timeFormatted);
        document.querySelector('.bar.progress').style.width = `${(time / duration) * 100}%`;
        document.querySelector('#time_current').innerHTML = timeFormatted;

        updateBufferedRange();

        const { hideInterfaceBy } = player.dataset;

        if (hideInterfaceBy && (new Date()).getTime() > hideInterfaceBy) {
            player.classList.add('hide_interface');
        }
    }

    function revealControls() {
        player.dataset.hideInterfaceBy = (new Date()).getTime() + 3000;
        player.classList.remove('hide_interface');
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

    // Invoked when playback is started from the intro version ring. Receives
    // the slice group from the version ring.
    function selectVersion(slice) {
        const option = versionOptions.find(option => option.slice === slice);

        // Change active version (to a different format/resolution) if needed
        if (video.getAttribute('src') !== option.button.dataset.src) {
            activeVersionOption.button.classList.remove('active');

            video.src = option.button.dataset.src;

            cycleVersionButton.textContent = option.button.dataset.label;
            activeVersionOption = option;
            activeVersionOption.button.classList.add('active');
        }

        play();
    }

    function activateVersionOption(option) {
        // If the requested version is already active we can return early
        if (option === activeVersionOption) {
            return;
        }

        if (!playerInit) {
            // If we haven't played back anything yet, we can simply activate
            // the requested version, trigger preloading and return early

            activeVersionOption.button.classList.remove('active');

            video.src = option.button.dataset.src;

            cycleVersionButton.textContent = option.button.dataset.label;
            activeVersionOption = option;
            activeVersionOption.button.classList.add('active');

            ensurePlayerInit();
        } else {
            // If we have alrady played back (or are playing back) a different
            // version, we memorize the current playback state and time, stop
            // playback (if necessary), switch to the new version, and then
            // restore the previous playback state and time

            activeVersionOption.button.classList.remove('active');

            const lastPlaying = pause();

            const lastTime = currentTime();

            video.src = option.button.dataset.src;

            // Resets the element and rescans the available resources, thereby
            // causing the changes [to the src attribute] to take effect.
            // https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/load
            // Note that things also work without this call, but it probably
            // doesn't hurt to be explicit about this and give the runtime an
            // extra nudge.
            video.load();

            seek(lastTime, lastPlaying);

            cycleVersionButton.textContent = option.button.dataset.label;

            activeVersionOption = option;
            activeVersionOption.button.classList.add('active');
        }
    }

    for (const option of versionOptions) {
        option.button.addEventListener('click', () => {
            activateVersionOption(option);
            versionSelectorControl.classList.remove('active');
            option.button.blur();
        });
    }

    // Cycling the version through clicking the (e.g.) "1080p" button left to
    // the fullscreen toggle
    cycleVersionButton.addEventListener('click', () => {
        const activeVersionOptionIndex = versionOptions.indexOf(activeVersionOption);
        const option = versionOptions[(activeVersionOptionIndex + 1) % versionOptions.length];
        activateVersionOption(option);
    });

    // While we seek we communicate the target time we are seeking to as the
    // current time, so the user knows their input was registered. Otherwise
    // our current time is the current time in the video.
    function currentTime() {
        if (seekState.seeking) {
            return seekState.seeking.targetTime;
        } else {
            return video.currentTime;
        }
    }

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

    // Performs global, one-time initialization of various player attributes
    // (or skips it if already initialized).
    function ensurePlayerInit() {
        if (!playerInit) {
            // We initially render the video element with preload="none" and
            // only begin any and all preloading once the user explicitly
            // requests playback.
            video.preload = 'auto';

            // Probably not needed but we are explicit about it anyway.
            // https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/load
            video.load();

            // We remove the poster because it would otherwise briefly appear
            // again as a flicker when we change resolution (i.e. the src
            // attribute) during playback.
            video.removeAttribute('poster');

            bufferedRangeUpdateInterval = setInterval(updateBufferedRange, 500);

            playerInit = true;
        }
    }

    // Pauses the video if it is playing and returns whether the video was
    // playing.
    function pause() {
        if (seekState.seeking) {
            seekState.seeking.cancel();
        } else if (!video.paused) {
            video.pause();
            return true;
        }

        return false;
    }

    function play() {
        ensurePlayerInit();

        if (seekState.initialSeek === undefined) {
            video.play();
        } else {
            const initialSeek = seekState.initialSeek;
            delete seekState.initialSeek;
            seek(initialSeek, true);
        }
    }

    function toggleFullscreen() {
        if (fullscreen.iPhoneMode) {
            if (video.webkitDisplayingFullscreen) {
                video.webkitExitFullscreen();
            } else {
                if (!fullscreen.alreadyPlayed) {
                    play();
                }
                video.webkitEnterFullscreen();
            }
        } else {
            if (document.fullscreenElement) {
                document.exitFullscreen();
            } else {
                player.requestFullscreen();
            }
        }
    }

    function togglePlayback() {
        if (seekState.seeking) {
            seekState.seeking.cancel();
        } else if (video.paused) {
            play();
        } else {
            video.pause();
        }
    }

    video.addEventListener('click', () => {
        revealControls();

        // On touch devices the first touch event registered while the
        // interface is hidden only serves to unhide the interface. The
        // touchstart handler sets a flag in this case which we check and
        // consume in the click handler, informing us to skip the playback
        // toggle this one time.
        if (window.inhibitPlaybackToggle) {
            delete window.inhibitPlaybackToggle;
        } else {
            togglePlayback();
        }
    });

    video.addEventListener('ended', () => {
        player.dataset.state = 'outro';
    });

    video.addEventListener('pause', () => {
        if (onPauseStateOverride === null) {
            // Normally we enter paused state on pause
            player.dataset.state = 'paused';
        }  else {
            // If an overide is specified we enter a different state
            player.dataset.state = onPauseStateOverride;
            onPauseStateOverride = null;
        }

        clearInterval(playbackUpdateInterval);
    });

    video.addEventListener('play', () => {
        fullscreen.alreadyPlayed = true;
        player.dataset.state = 'playing';

        // TODO: Dynamic, small interval when the video is really short, otherwise can be a fixed higher value
        //       Formula should/can take into account width of video on screen vs duration of video.
        playbackUpdateInterval = setInterval(playbackUpdate, 80);
    });

    video.addEventListener('playing', () => {
        player.dataset.state = 'playing';
    });

    video.addEventListener('progress', () => {
        updateBufferedRange();
    });

    video.addEventListener('seeked', () => {
        console.log('on seeked');
        if (!video.paused) {
            player.dataset.state = 'playing';
        }
    });

    video.addEventListener('touchstart', () => {
        if (player.classList.contains('hide_interface')) {
            // On touch devices the interface can not be unhidden by moving
            // the mouse cursor (there is none), therefore the first touch
            // event occuring while the interface is hidden is used as the
            // trigger for unhiding the interface instead. To prevent this
            // from additionally toggling video playback (which is otherwise
            // the primary function of clicking) we register a one-shot
            // override that is consumed by the click handler and tells it to
            // not toggle playback this one time.
            window.inhibitPlaybackToggle = true;
        }
    });

    video.addEventListener('waiting', () => {
        player.dataset.state = 'loading';
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

    // Check if the end of the single buffered range we display¹ has
    // shifted, and if so update it in the DOM.
    //
    // ¹ The range that overlaps and expands rightwards from the current
    // playback point
    function updateBufferedRange() {
        const buffered = video.buffered;

        let currentBufferedRangeEnd = 0;

        const current = currentTime();

        for (let index = 0; index < buffered.length; index++) {
            const end = buffered.end(index);

            if (end >= current) {
                if (buffered.start(index) <= current) {
                    currentBufferedRangeEnd = end;
                } else {
                    // This range already starts after the current video time,
                    // the others will too, we can return early
                    break;
                }
            }
        }

        if (currentBufferedRangeEnd !== lastBufferedRangeEnd) {
            document.querySelector('.bar.buffered').style.width = `${(currentBufferedRangeEnd / duration) * 100}%`;
            lastBufferedRangeEnd = currentBufferedRangeEnd;
        }
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
            player.classList.add('fullscreen');
        });
        video.addEventListener('webkitendfullscreen', () => {
            player.classList.remove('fullscreen');
        });
    } else {
        player.addEventListener('fullscreenchange', () => {
            player.classList.toggle('fullscreen', document.fullscreenElement !== null);
        });
    }

    player.addEventListener('mousemove', () => {
        revealControls();
    });

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
            if (currentTime() - 5 < 0) {
                seek(0);
            } else {
                seek(currentTime() - 5);
            }
            event.preventDefault();
        } else if (event.key === 'ArrowRight') {
            if (player.dataset.state !== 'outro') {
                if (currentTime() + 5 > duration) {
                   seek(duration);
                } else {
                    seek(currentTime() + 5);
                }
            }

            event.preventDefault();
        }
    });

    timeline.addEventListener('click', () => {
        const bounds = timeline.getBoundingClientRect();
        const factor = (event.clientX - bounds.x) / bounds.width;
        const newTime = factor * timelineInput.max;
        seek(newTime);
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
    resizeObserver.observe(player);

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

    // If possible seeks directly to target time and triggers playback if
    // requested. If we still need to load more metadata to perform the seek,
    // this enters/registers a seeking state that continously tries to finish
    // the seek, available through the global `seekState.seeking` property.
    // After seeking finishes, playback is triggered if requested.
    function seek(targetTime, playOnComplete = null) {
        // If a seek is requested before playback we need to ensure we are
        // actually preloading the video
        ensurePlayerInit();

        // This primarily serves to prevent seeking exactly to the end, and
        // only secondarily to prevent seeking beyond the end. We adjust the
        // seek to just before the end of the video, so there can still be a
        // predictable play->pause->end event cascade, leading to a clean
        // outro state
        if (targetTime >= duration) {
            targetTime = duration - 0.1;
        }

        // Update timeline progress to immediately reflect the requested
        // target time, regardless of whether we can instantly seek there or
        // need to initiate a consecutive asynchronous seeking process.
        playbackUpdate(targetTime);

        if (seekState.seeking) {
            seekState.seeking.targetTime = targetTime;
        } else if (trySeek(targetTime)) {
            // The seek immediately succeeded, so we don't change state and
            // just trigger playback right away (if it was requested)

            if (playOnComplete) {
                video.play();
            } else if (video.paused) {
                // If we are not in playback we could be both on the intro or
                // outro screen when the seek is requested, so we need to
                // ensure a paused state after seeking in this case
                player.dataset.state = 'paused';
            }
        } else {
            // If the seek did not immediately succeed we enter/register a
            // seeking state, that can be asynchronously altered/accessed
            // through its cancel/onComplete/targetTime properties.

            if (playOnComplete === null) {
                playOnComplete = !video.paused;
            }

            const seeking = { playOnComplete, targetTime };

            function tryFinishSeeking() {
                if (trySeek(seeking.targetTime)) {
                    delete seekState.seeking;

                    clearInterval(seeking.interval);

                    if (playOnComplete) {
                        video.play();
                    } else {
                        player.dataset.state = 'paused';
                    }
                }
            }

            seeking.interval = setInterval(tryFinishSeeking, 30);

            seeking.cancel = () => {
                clearInterval(seeking.interval);
                delete seekState.seeking;
                player.dataset.state = 'paused';
                playbackUpdate();
            };

            // We expose `cancel`, `playOnComplete` and `targetTime` so that consecutive
            // parallel playback/seek requests may either cancel seeking
            // (by calling `seeking.cancel()`) or reconfigure up to which
            // time seeking should occur (by setting `seeking.targetTime =
            // newTargetTime`), or reconfigure what should happen after seeking
            // completes (by setting `seeking.playOnComplete = true|false).
            seekState.seeking = seeking;

            if (video.paused) {
                player.dataset.state = 'loading';
            } else {
                onPauseStateOverride = 'loading';
                video.pause();
            }
        }
    }

    // Seeks directly to targetTime if it lies within an already seekable
    // range and then returns true. Otherwise returns false.
    function trySeek(targetTime) {
        const seekable = video.seekable;

        for (let index = 0; index < seekable.length; index++) {
            if (seekable.end(index) >= targetTime) {
                if (seekable.start(index) <= targetTime) {
                    console.log('trySeek can do', targetTime)
                    video.currentTime = targetTime;
                    return true;
                } else {
                    break;
                }
            }
        }

        return false;
    }

    // Seek to a given timecode if it was specified in a hash parameter
    const params = parseHashParams();
    if (params) {
        if (params.time !== undefined) {
            seekState.initialSeek = params.time;
        }
    }

    window.addEventListener('hashchange', event => {
        const params = parseHashParams();
        if (params) {
            if (params.time !== undefined) {
                seek(params.time, true);
                player.focus();
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
