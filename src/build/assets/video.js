const maximizeIcon = document.querySelector('#maximize_icon').content;
const minimizeIcon = document.querySelector('#minimize_icon').content;
const playIcon = document.querySelector('#play_icon').content;
const pauseIcon = document.querySelector('#pause_icon').content;
const subtitlesIcon = document.querySelector('#subtitles_icon').content;
const volumeLoudIcon = document.querySelector('#volume_loud_icon').content;
const volumeMediumIcon = document.querySelector('#volume_medium_icon').content;
const volumeMutedIcon = document.querySelector('#volume_muted_icon').content;

const cycleVersionButton = document.querySelector('#cycle_versions');
const playbackSubtitlesSelectorControl = document.querySelector('.right_playback_controls .selector_control.subtitles');
const playbackToggleFullscreenButton = document.querySelector('.right_playback_controls .toggle_fullscreen');
const playbackToggleVolumeButton = document.querySelector('.left_playback_controls .toggle_volume');
const playerControls = document.querySelector('.player_controls');
const playerWrapper = document.querySelector('.player_wrapper');
const replayButton = document.querySelector('#replay');
const timelineInput = document.querySelector('.timeline input');
const speedControl = document.querySelector('#speed');
const speedMultiplier = document.querySelector('#speed .multiplier');
const togglePlaybackButton = document.querySelector('#toggle_playback');
const versionSelectorControl = document.querySelector('.selector_control.version');
const versionSlices = document.querySelectorAll('.versions_ring g[data-src]');
const versionsRing = document.querySelector('.versions_ring');
const video = document.querySelector('video');

// Before we even begin playback video.duration is NaN, so we use our own data throughout
const duration = parseFloat(video.dataset.duration);
let playbackUpdateInterval;

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

// We internally manage speed as (integer) percent values to avoid having to
// deal with float rounding issues.
let speed = 100;

const volume = {
    interactive: false,
    value: 1
};

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

function playbackUpdate() {
    const time = video.currentTime;
    const timeFormatted = formatTime(time);

    timelineInput.value = time;
    timelineInput.setAttribute('aria-valuenow', time);
    timelineInput.setAttribute('aria-valuetext', timeFormatted);
    document.querySelector('.bar.progress').style.width = `${(time / duration) * 100}%`;
    document.querySelector('#time_current').innerHTML = timeFormatted;

    if (playerControls.dataset.hideBy && (new Date()).getTime() > playerControls.dataset.hideBy) {
        playerControls.style.display = 'none';
    }
}

function revealControls() {
    playerControls.dataset.hideBy = (new Date()).getTime() + 3000;
    playerControls.style.display = null;
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

function toggleFullscreen() {
    if (document.fullscreenElement) {
        document.exitFullscreen();
    } else {
        playerWrapper.requestFullscreen();
    }
}

function togglePlayback() {
    if (video.paused) {
        playerWrapper.dataset.state = 'playing';
        video.play();
        togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
        // TODO: Dynamic, small interval when the video is really short, otherwise can be a fixed higher value
        //       Formula should/can take into account width of video on screen vs duration of video.
        playbackUpdateInterval = setInterval(playbackUpdate, 80);
    } else {
        video.pause();
        clearInterval(playbackUpdateInterval);
        togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
    }
}

function toggleVolume() {
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

    playbackToggleVolumeButton.classList.toggle('dimmed', video.muted);
    playbackToggleVolumeButton.replaceChildren(volumeIcon.cloneNode(true));
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

playbackToggleFullscreenButton.addEventListener('click', toggleFullscreen);
playbackToggleVolumeButton.addEventListener('click', toggleVolume);

playerWrapper.addEventListener('fullscreenchange', () => {
    const fullscreenIcon = document.fullscreenElement ? minimizeIcon : maximizeIcon;
    playbackToggleFullscreenButton.replaceChildren(fullscreenIcon.cloneNode(true));
});

playerWrapper.addEventListener('mousemove', revealControls);

replayButton.addEventListener('click', () => {
    playerWrapper.dataset.state = 'playing';
    video.play();
    togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
    playbackUpdateInterval = setInterval(playbackUpdate, 80);
});

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

video.addEventListener('ended', () => {
    playerWrapper.dataset.state = 'outro';
    clearInterval(playbackUpdateInterval);
    togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
});

video.addEventListener('click', togglePlayback);

window.addEventListener('keydown', event => {
    if (event.key === ' ') {
        togglePlayback();
        event.preventDefault();
    } else if (event.key === 'f') {
        toggleFullscreen();
    } else if (event.key === 'ArrowLeft') {
        if (video.currentTime - 5 < 0) {
            video.currentTime = 0;
        } else {
            video.currentTime -= 5;
        }
    } else if (event.key === 'ArrowRight') {
        if (video.currentTime + 5 > duration) {
            video.currentTime = duration;
        } else {
            video.currentTime += 5;
        }
    }
});

timelineInput.addEventListener('input', _ => {
    playerWrapper.dataset.state = 'playing';
    video.currentTime = timelineInput.value;
    playbackUpdate();
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
