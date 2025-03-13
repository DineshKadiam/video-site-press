const maximizeIcon = document.querySelector('#maximize_icon').content;
const minimizeIcon = document.querySelector('#minimize_icon').content;
const playIcon = document.querySelector('#play_icon').content;
const pauseIcon = document.querySelector('#pause_icon').content;
const subtitlesIcon = document.querySelector('#subtitles_icon').content;
const volumeLoudIcon = document.querySelector('#volume_loud_icon').content;
const volumeMediumIcon = document.querySelector('#volume_medium_icon').content;
const volumeMutedIcon = document.querySelector('#volume_muted_icon').content;

const intro = document.querySelector('.player_intro');
const outro = document.querySelector('.player_outro');
const playerControls = document.querySelector('.player_controls');
const playerWrapper = document.querySelector('.player_wrapper');
const replayButton = document.querySelector('#replay');
const timelineInput = document.querySelector('.timeline input');
const selectResolutionButton = document.querySelector('#select_resolution');
const selectSubtitlesButton = document.querySelector('#select_subtitles');
const toggleFullscreenButton = document.querySelector('#toggle_fullscreen');
const togglePlaybackButton = document.querySelector('#toggle_playback');
const toggleSpeedButton = document.querySelector('#toggle_speed');
const toggleVolumeButton = document.querySelector('#toggle_volume');
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

const volume = {
    interactive: false,
    value: 1
};

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

    toggleVolumeButton.replaceChildren(volumeIcon.cloneNode(true));
}

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

function selectResolution(group) {
    intro.style.display = 'none';

    if (video.getAttribute('src') !== group.dataset.src) {
        video.src = group.dataset.src;
        selectResolutionButton.innerHTML = group.dataset.label;
    }

    video.play();
    togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
    playbackUpdateInterval = setInterval(playbackUpdate, 80); // TODO: Dynamic, small interval when the video is really short, otherwise can be a fixed higher value
                                                              //       Formula should/can take into account width of video on screen vs duration of video.
}

function toggleFullscreen() {
    if (document.fullscreenElement) {
        document.exitFullscreen();
        toggleFullscreenButton.replaceChildren(maximizeIcon.cloneNode(true));
    } else {
        document.querySelector('.player_wrapper').requestFullscreen();
        toggleFullscreenButton.replaceChildren(minimizeIcon.cloneNode(true));
    }
}

function togglePlayback() {
    if (video.paused) {
        intro.style.display = 'none';
        outro.classList.remove('active');
        video.play();
        togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
        playbackUpdateInterval = setInterval(playbackUpdate, 80);
    } else {
        video.pause();
        clearInterval(playbackUpdateInterval);
        togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
    }
}

for (const group of document.querySelectorAll('.versions_ring g[data-src]')) {
    group.addEventListener('click', () =>  selectResolution(group));
    group.addEventListener('keypress', event =>  {
        if (event.key === 'Enter' || event.key === ' ') {
            selectResolution(group);
            event.preventDefault();
        }
    });
}

window.addEventListener('keydown', revealControls);
playerWrapper.addEventListener('mousemove', revealControls);

replayButton.addEventListener('click', () => {
    outro.classList.remove('active');
    video.play();
    togglePlaybackButton.replaceChildren(pauseIcon.cloneNode(true));
    playbackUpdateInterval = setInterval(playbackUpdate, 80);
});

selectResolutionButton.addEventListener('click', () => {
    if (intro.style.display === 'none') {
        outro.classList.remove('active');
        intro.style.display = null;
        if (!video.paused) {
            video.pause();
            clearInterval(playbackUpdateInterval);
            togglePlaybackButton.replaceChildren(playIcon.cloneNode(true));
        }
    } else {
        intro.style.display = 'none';
    }
});

if (selectSubtitlesButton) {
    selectSubtitlesButton.addEventListener('click', () => {
        const enableTrack = track => {
            track.mode = 'showing';
            const text = document.createElement('span');
            text.textContent = ' ' + (track.language || track.label);
            selectSubtitlesButton.replaceChildren(
                subtitlesIcon.cloneNode(true),
                text
            );
        };

        let prevTrackDisabled = false;
        for (const track of video.textTracks) {
            if (prevTrackDisabled) {
                enableTrack(track);
                return;
            }

            if (track.mode === 'showing') {
                track.mode = 'disabled';
                prevTrackDisabled = true;
            }
        }

        if (prevTrackDisabled) {
            selectSubtitlesButton.replaceChildren(subtitlesIcon.cloneNode(true));
        } else {
            enableTrack(video.textTracks[0]);
        }
    });
}

toggleFullscreenButton.addEventListener('click', toggleFullscreen);

toggleSpeedButton.addEventListener('click', () => {
    if (toggleSpeedButton.innerHTML === '1×') {
        video.playbackRate = 1.25;
        toggleSpeedButton.innerHTML = '1.25×'
    } else if (toggleSpeedButton.innerHTML === '1.25×') {
        video.playbackRate = 1.5;
        toggleSpeedButton.innerHTML = '1.5×'
    } else if (toggleSpeedButton.innerHTML === '1.5×') {
        video.playbackRate = 1.75;
        toggleSpeedButton.innerHTML = '1.75×'
    } else if (toggleSpeedButton.innerHTML === '1.75×') {
        video.playbackRate = 2;
        toggleSpeedButton.innerHTML = '2×'
    } else if (toggleSpeedButton.innerHTML === '2×') {
        video.playbackRate = 0.5;
        toggleSpeedButton.innerHTML = '0.5×'
    } else if (toggleSpeedButton.innerHTML === '0.5×') {
        video.playbackRate = 0.75;
        toggleSpeedButton.innerHTML = '0.75×'
    } else {
        video.playbackRate = 1;
        toggleSpeedButton.innerHTML = '1×'
    }
});

toggleVolumeButton.addEventListener('click', () => {
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
});

togglePlaybackButton.addEventListener('click', togglePlayback);

video.addEventListener('ended', () => {
    outro.classList.add('active');
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
    intro.style.display = 'none';
    video.currentTime = timelineInput.value;
    playbackUpdate();
});
