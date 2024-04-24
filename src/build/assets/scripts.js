const copyLink = document.querySelector('#copy-link');
const copyNotificationTimeouts = {};

if (copyLink) {
    function copyToClipboard(button) {
        const notify = (success, content) => {
            if (content in copyNotificationTimeouts) {
                clearTimeout(copyNotificationTimeouts[content]);
                delete copyNotificationTimeouts[content];
            }

            button.classList.toggle('confirmed', success);
            button.classList.toggle('failed', !success);

            copyNotificationTimeouts[content] = setTimeout(() => {
                button.classList.remove(success ? 'confirmed' : 'failed');
            }, 3000);
        };

        const content = copyLink.dataset.url || window.location.href.split('#')[0]; // discard hash if present

        navigator.clipboard
            .writeText(content)
            .then(() => notify(true, content))
            .catch(_err => notify(false, content));
    };

    copyLink.addEventListener('click', event => copyToClipboard(event.target));
}

const video = document.querySelector('video');

// TODO: Potentially move in separate script file that only gets loaded on video pages
if (video) {
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
    const toggleFullscreenButton = document.querySelector('#toggle_fullscreen');
    const togglePlaybackButton = document.querySelector('#toggle_playback');
    const toggleSpeedButton = document.querySelector('#toggle_speed');
    const toggleVolumeButton = document.querySelector('#toggle_volume');

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

    function togglePlayback() {
        if (video.paused) {
            intro.style.display = 'none';
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

    toggleFullscreenButton.addEventListener('click', () => {
        if (document.fullscreenElement) {
            document.exitFullscreen();
            toggleFullscreenButton.replaceChildren(document.querySelector('#maximize_icon').content.cloneNode(true));
        } else {
            document.querySelector('.player_wrapper').requestFullscreen();
            toggleFullscreenButton.replaceChildren(document.querySelector('#minimize_icon').content.cloneNode(true));
        }
    });

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

    video.addEventListener('ended', () => {
        outro.classList.add('active');
        clearInterval(playbackUpdateInterval);
        togglePlaybackButton.replaceChildren(document.querySelector('#play_icon').content.cloneNode(true));
    });

    video.addEventListener('click', togglePlayback);
    togglePlaybackButton.addEventListener('click', togglePlayback);

    timelineInput.addEventListener('input', _ => {
        intro.style.display = 'none';
        video.currentTime = timelineInput.value;
        playbackUpdate();
    });
}
