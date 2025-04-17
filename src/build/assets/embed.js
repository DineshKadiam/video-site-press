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

const intro = document.querySelector('.player_intro');
const outro = document.querySelector('.player_outro');
const playerControls = document.querySelector('.player_controls');
const playerWrapper = document.querySelector('.player_wrapper');
const replayButton = document.querySelector('#replay');
const timelineInput = document.querySelector('.timeline input');
const selectResolutionButton = document.querySelector('#select_resolution');
const selectSubtitlesButton = document.querySelector('#select_subtitles');
const speedControl = document.querySelector('#speed');
const speedMultiplier = document.querySelector('#speed .multiplier');
const toggleFullscreenButton = document.querySelector('#toggle_fullscreen');
const togglePlaybackButton = document.querySelector('#toggle_playback');
const toggleVolumeButton = document.querySelector('#toggle_volume');

// We internally manage speed as (integer) percent values to avoid having to
// deal with float rounding issues.
let speed = 100;

const persistedVolume = localStorage.getItem('hyper8Volume');

if (persistedVolume !== null) {
    video.volume = parseFloat(persistedVolume);

    if (video.volume > 0.666) {
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_loud_icon').content.cloneNode(true));
    } else if (video.volume < 0.333) {
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_muted_icon').content.cloneNode(true));
    } else {
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_medium_icon').content.cloneNode(true));
    }
}

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
    togglePlaybackButton.replaceChildren(document.querySelector('#pause_icon').content.cloneNode(true));
    playbackUpdateInterval = setInterval(playbackUpdate, 80); // TODO: Dynamic, small interval when the video is really short, otherwise can be a fixed higher value
                                                              //       Formula should/can take into account width of video on screen vs duration of video.
}

function toggleFullscreen() {
    if (document.fullscreenElement) {
        document.exitFullscreen();
        toggleFullscreenButton.replaceChildren(document.querySelector('#maximize_icon').content.cloneNode(true));
    } else {
        document.querySelector('.player_wrapper').requestFullscreen();
        toggleFullscreenButton.replaceChildren(document.querySelector('#minimize_icon').content.cloneNode(true));
    }
}

function togglePlayback() {
    if (video.paused) {
        intro.style.display = 'none';
        outro.classList.remove('active');
        video.play();
        togglePlaybackButton.replaceChildren(document.querySelector('#pause_icon').content.cloneNode(true));
        playbackUpdateInterval = setInterval(playbackUpdate, 80);
    } else {
        video.pause();
        clearInterval(playbackUpdateInterval);
        togglePlaybackButton.replaceChildren(document.querySelector('#play_icon').content.cloneNode(true));
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
    togglePlaybackButton.replaceChildren(document.querySelector('#pause_icon').content.cloneNode(true));
    playbackUpdateInterval = setInterval(playbackUpdate, 80);
});

selectResolutionButton.addEventListener('click', () => {
    if (intro.style.display === 'none') {
        outro.classList.remove('active');
        intro.style.display = null;
        if (!video.paused) {
            video.pause();
            clearInterval(playbackUpdateInterval);
            togglePlaybackButton.replaceChildren(document.querySelector('#play_icon').content.cloneNode(true));
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
                document.querySelector('#subtitles_icon').content.cloneNode(true),
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
            selectSubtitlesButton.replaceChildren(document.querySelector('#subtitles_icon').content.cloneNode(true));
        } else {
            enableTrack(video.textTracks[0]);
        }
    });
}

toggleFullscreenButton.addEventListener('click', toggleFullscreen);

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

// Volume button handlers

toggleVolumeButton.addEventListener('click', () => {
    if (video.volume > 0.666) {
        video.volume = 0;
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_muted_icon').content.cloneNode(true));
        localStorage.setItem('hyper8Volume', '0');
    } else if (video.volume < 0.333) {
        video.volume = 0.5;
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_medium_icon').content.cloneNode(true));
        localStorage.setItem('hyper8Volume', '0.5');
    } else {
        video.volume = 1;
        toggleVolumeButton.replaceChildren(document.querySelector('#volume_loud_icon').content.cloneNode(true));
        localStorage.setItem('hyper8Volume', '1');
    }
});

togglePlaybackButton.addEventListener('click', togglePlayback);

video.addEventListener('ended', () => {
    outro.classList.add('active');
    clearInterval(playbackUpdateInterval);
    togglePlaybackButton.replaceChildren(document.querySelector('#play_icon').content.cloneNode(true));
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
