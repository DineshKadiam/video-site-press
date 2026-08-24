const modal = document.querySelector('#modal').content;
const processingElement = document.querySelector('.processing');
const processingIcon = document.querySelector('#processing_icon').content;
const reloadIcon = document.querySelector('#reload_icon').content;
const splashWrapper = document.querySelector('.splash_wrapper');
const syncLog = document.querySelector('#sync_log');
const syncMenu = document.querySelector('#sync');
const syncStatus = document.querySelector('.sync_status');

// Memorizes previously seen states for ongoing background processing so we
// can determine whether to re-render live update widgets.
const processing = {};

const copyButtons = document.querySelectorAll('button[data-copy]');
const copyNotificationTimeouts = {};
if (copyButtons.length) {
    const copyIcon = document.querySelector('#copy_icon').content;
    const failureIcon = document.querySelector('#failure_icon').content;
    const successIcon = document.querySelector('#success_icon').content;

    function copyToClipboard(button) {
        const notify = (success, content) => {
            if (content in copyNotificationTimeouts) {
                clearTimeout(copyNotificationTimeouts[content]);
                delete copyNotificationTimeouts[content];
            }

            const feedbackIcon = (success ? successIcon : failureIcon).cloneNode(true);

            const iconContainer = button.querySelector('.icon');

            iconContainer.replaceChildren(feedbackIcon);

            copyNotificationTimeouts[content] = setTimeout(
                () => iconContainer.replaceChildren(copyIcon.cloneNode(true)),
                3000
            );
        };

        const content = button.dataset.copy;

        navigator.clipboard
            .writeText(content)
            .then(() => notify(true, content))
            .catch(_err => notify(false, content));
    };

    for (const copyButton of copyButtons) {
        copyButton.addEventListener('click', () => copyToClipboard(copyButton));
    }
}

function formatTime(seconds) {
    if (seconds < 60) {
        return `0:${Math.floor(seconds).toString().padStart(2, '0')}`;
    } else if (seconds < 3600) {
        return `${Math.floor(seconds / 60)}:${Math.floor(seconds % 60).toString().padStart(2, '0')}`;
    } else {
        return `${Math.floor(seconds / 3600)}:${Math.floor((seconds % 3600) / 60).toString().padStart(2, '0')}:${Math.floor(seconds % 60).toString().padStart(2, '0')}`;
    }
}

function fetchProcessing() {
    const jobs = document.querySelector('div[data-jobs]');
    const mediaJobs = document.querySelector('div[data-media-jobs]');

    let processingUrl = '/processing';

    if (mediaJobs) {
        processingUrl += '/media/' + mediaJobs.dataset.mediaJobs;
    }

    fetch(processingUrl)
        .then(response => response.json())
        .then(response => {
            const {
                globalJobReport,
                localJobReport,
                syncState
            } = response;

            if (jobs) {
                // When on the jobs page, update the local processing indicator widget
                updateJobs(jobs, globalJobReport);
            } else if (mediaJobs) {
                // When on a video's media page, update the local processing indicator widget
                updateMediaJobs(mediaJobs, localJobReport);
            }

            if (syncState !== null) {
                // Update the status line in the server menu ("Syncing/Sync finished")
                updateSyncStatus(syncState);

                // Update the sync log (if we are on the sync log page)
                if (syncLog) {
                    updateSyncLog(syncState);
                }
            }

            // Update the processing information (Idle/Progress/etc.) in the header
            updateProcessingWidget(globalJobReport);
        });
}

// Example options:
// {
//     callback: () => { ... },   // mandatory
//     confirmLabel: 'Submit',    // mandatory
//     text: 'Really do this?',   // mandatory
//     type: 'password'           // if specified callback receives a prompted password
// }
function hyperModal(options) {
    const {
        callback,
        confirmLabel,
        text,
        type
    } = options;

    const dialogClone = modal.cloneNode(true);

    document.body.appendChild(dialogClone);

    // For some reason .showModal and .remove would not work on dialogClone itself,
    // so we "re-query" the element from the DOM after having appended it.
    // Possibly follow up at some point why this happens.
    const dialog = document.querySelector('.modal');

    dialog.querySelector('.confirm').textContent = confirmLabel;
    dialog.querySelector('.text').textContent = text;


    function cancel() {
        dialog.remove();
    }

    function confirm() {
        dialog.remove();

        if (type === 'password') {
            const password = dialog.querySelector('input').value;

            callback(password);
        } else {
            callback();
        }
    }

    if (type === 'password') {
        const input = document.createElement('input');

        input.type = 'password';

        input.addEventListener('keydown', event => {
            if (event.key === 'Enter' ) {
                confirm();
            }
        });

        const buttons = dialog.querySelector('.buttons');
        buttons.parentNode.insertBefore(input, buttons);
    }

    dialog.querySelector('.cancel').addEventListener('click', cancel);
    dialog.querySelector('.confirm').addEventListener('click', confirm);

    dialog.addEventListener('close', () => dialog.remove());

    dialog.showModal();

    if (options.type === 'password') {
        dialog.querySelector('input').focus();
    } else {
        // Force an explicit focus on confirm instead of an implicit one on cancel
        dialog.querySelector('.confirm').focus();
    }
}

function updateJobs(jobs, globalJobReport) {
    const {
        activeJobs,
        completedCount,
        failedCount,
        pendingCount
    } = globalJobReport;

    // Stores the last (currently displayed) render state, so we can make the
    // determination to re-render (or not).
    if (processing.state === undefined) {
        processing.state = {
            completedCount: {
                initial: completedCount,
                last: completedCount
            },
            failedCount: {
                initial: failedCount,
                last: failedCount
            }
        };
    }

    if (
        activeJobs.length !== processing.state.activeCount ||
        completedCount !== processing.state.completedCount.last ||
        failedCount !== processing.state.failedCount.last ||
        pendingCount !== processing.state.pendingCount
    ) {
        const content = [];

        if (pendingCount > 0) {
            const span = document.createElement('span');
            span.textContent = t.xxxPendingJobs(pendingCount);
            content.push(span);
        }

        if (activeJobs.length > 0) {
            const span = document.createElement('span');
            span.textContent = t.xxxActiveJobs(activeJobs.length);
            content.push(span);
        }

        if (
            completedCount !== processing.state.completedCount.initial ||
            failedCount !== processing.state.failedCount.initial
        ) {
            const span = document.createElement('span');

            const spanInner = document.createElement('span');
            spanInner.textContent = t.newResultsAvailable;
            span.append(spanInner);

            const a = document.createElement('a');
            a.href = '';
            a.append(reloadIcon.cloneNode(true), t.refresh);
            span.append(a);

            content.push(span);
        }

        processing.state.activeCount = activeJobs.length;
        processing.state.completedCount.last = completedCount;
        processing.state.failedCount.last = failedCount;
        processing.state.pendingCount = pendingCount;

        jobs.replaceChildren(...content);
    }
}

function updateMediaJobs(mediaJobs, localJobReport) {
    const {
        activeJobs,
        completedCount,
        pendingCount
    } = localJobReport;

    // Stores the last (currently displayed) render state, so we can make the
    // determination to re-render (or not).
    if (processing.state === undefined) {
        processing.state = {
            activeCount: 0,
            completedCount: {
                initial: completedCount,
                last: completedCount
            },
            pendingCount: 0
        };
    }

    if (
        activeJobs.length !== processing.state.activeCount ||
        completedCount !== processing.state.completedCount.last ||
        pendingCount !== processing.state.pendingCount
    ) {
        const content = [];

        if (pendingCount > 0) {
            const span = document.createElement('span');
            span.textContent = t.xxxPendingJobs(pendingCount);
            content.push(span);
        }

        if (activeJobs.length > 0) {
            const span = document.createElement('span');
            span.textContent = t.xxxActiveJobs(activeJobs.length);
            content.push(span);
        }

        if (completedCount !== processing.state.completedCount.initial) {
            const span = document.createElement('span');

            const spanInner = document.createElement('span');
            spanInner.textContent = t.newResultsAvailable;
            span.append(spanInner);

            const a = document.createElement('a');
            a.href = '';
            a.append(reloadIcon.cloneNode(true), t.refresh);
            span.append(a);

            content.push(span);
        }

        processing.state.activeCount = activeJobs.length;
        processing.state.completedCount.last = completedCount;
        processing.state.pendingCount = pendingCount;

        mediaJobs.replaceChildren(...content);
    }
}

function updateProcessingWidget(globalJobReport) {
    let content = [processingIcon.cloneNode(true)];

    if (globalJobReport.activeJobs.length) {
        const text = document.createElement('div');

        processingElement.classList.add('active');
        for (const lastProgress of globalJobReport.activeJobs) {
            const badge = document.createElement('div');

            badge.classList.add('processing_badge');

            if (lastProgress) {
                const left = document.createElement('span');
                left.textContent = lastProgress.percent.toString() + '%';

                const right = document.createElement('span');
                right.classList.add('inverted');
                right.textContent = '-' + formatTime(lastProgress.durationRemaining);

                badge.append(left, right);
            } else {
                const text = document.createElement('span');
                text.textContent = 'MISC';
                badge.append(text);
            }

            content.push(badge);
        }
    } else {
        processingElement.classList.remove('active');

        const text = document.createElement('span');
        text.textContent = t.idle;

        content.push(text);
    }

    if (globalJobReport.failedCount > 0) {
        const text = document.createElement('span');
        text.textContent = `⚠${globalJobReport.failedCount}`;
        content.push(text);
    }

    if (globalJobReport.pendingCount > 0) {
        const text = document.createElement('span');
        text.textContent = `+${globalJobReport.pendingCount}`;
        content.push(text);
    }

    // TODO: We could make use of globalJobReport.completedCount if we want at some point

    processingElement.replaceChildren(...content);
}

function updateSyncLog(syncState) {
    const log = syncState.log;

    // We dynamically update the sync log only if the currently open log was
    // opened for/with the currently running sync session.
    if (syncLog.dataset.sessionId === syncState.sessionId) {
        const eventsRendered = parseInt(syncLog.dataset.eventsRendered);

        if (eventsRendered < log.length) {
            for (let index = eventsRendered; index < log.length; index++) {
                const event = log[index];

                const li = document.createElement('li');

                if (syncState.simulation) {
                    const span = document.createElement('span');
                    span.classList.add('simulation');
                    span.textContent = t.simulation;
                    li.append(span);
                }

                const startedAt = document.createElement('span');
                startedAt.textContent = `[${event.startedAt}]`;
                li.append(startedAt);

                if (event.step !== null) {
                    const progress = document.createElement('span');
                    progress.classList.add('progress');
                    progress.textContent = `(${event.step}/${syncState.steps})`;
                    li.append(progress);
                }

                const message = document.createElement('span');

                if (event.kind === 'add_file') {
                    message.classList.add('path');

                    const action = document.createElement('span');
                    action.classList.add('action', 'add_file');
                    action.textContent = t.addFile;
                    li.append(action);

                    const size = document.createElement('span');
                    size.classList.add('size');
                    size.textContent = event.size;
                    li.append(size);
                } else if (event.kind === 'create_dir') {
                    message.classList.add('path');

                    const action = document.createElement('span');
                    action.classList.add('action', 'create_dir');
                    action.textContent = t.createDirectory;
                    li.append(action);
                } else if (event.kind === 'delete_dir') {
                    message.classList.add('path');

                    const action = document.createElement('span');
                    action.classList.add('action', 'delete_dir');
                    action.textContent = t.deleteDirectory;
                    li.append(action);
                } else if (event.kind === 'delete_file') {
                    message.classList.add('path');

                    const action = document.createElement('span');
                    action.classList.add('action', 'delete_file');
                    action.textContent = t.deleteFile;
                    li.append(action);

                    const size = document.createElement('span');
                    size.classList.add('size');
                    size.textContent = event.size;
                    li.append(size);
                } else if (event.kind === 'update_file') {
                    message.classList.add('path');

                    const action = document.createElement('span');
                    action.classList.add('action', 'update_file');
                    action.textContent = t.updateFile;
                    li.append(action);

                    const size = document.createElement('span');
                    size.classList.add('size');
                    size.textContent = event.size;
                    li.append(size);
                } else if (event.kind === 'error') {
                    message.classList.add('err');
                } else {
                    message.classList.add('message');
                }

                message.textContent = event.message;
                li.append(message);

                syncLog.append(li);
            }

            syncLog.dataset.eventsRendered = log.length;
        }
    }
}

function updateSyncStatus(syncState) {
    if (syncState.finished) {
        if (syncMenu.classList.contains('active')) {
            const a = document.createElement('a');
            a.href = '/sync_log';

            // TODO: Maybe introduce a success icon here (checkmark)
            // a.append(successIcon.cloneNode(true));

            const text = document.createElement('span');
            text.textContent = t.syncFinished;
            a.append(text);

            const durationText = document.createElement('span');
            durationText.textContent = formatTime(syncState.duration);
            a.append(durationText);

            syncStatus.replaceChildren(a);

            syncMenu.classList.remove('active')
        }
    } else {
        syncMenu.classList.add('active');

        const a = document.createElement('a');
        a.href = '/sync_log';

        // TODO: Maybe introduce a progress icon here (spinner, clock, ?)
        // a.append(successIcon.cloneNode(true));

        const text = document.createElement('span');
        text.textContent = t.syncing;
        a.append(text);

        const durationText = document.createElement('span');
        durationText.textContent = formatTime(syncState.duration);
        a.append(durationText);

        syncStatus.replaceChildren(a);
    }
}

for (const form of document.querySelectorAll('form[data-confirm]')) {
    form.addEventListener('submit', event => {
        event.preventDefault();

        const modalOptions = {};

        modalOptions.callback = () => form.submit();
        modalOptions.confirmLabel = form.dataset.confirmLabel;
        modalOptions.text = form.dataset.confirmText;

        hyperModal(modalOptions);
    });
}

for (const form of document.querySelectorAll('form[data-sync-precheck]')) {
    form.addEventListener('submit', event => {
        event.preventDefault();

        const confirm = form.dataset.syncConfirm !== undefined;
        const protocol = form.dataset.syncProtocol;

        const configUrl = `/sync/${protocol}/config`;

        fetch(configUrl)
            .then(response => response.json())
            .then(response => {
                const {
                    passwordPrompt,
                    privateKey,
                    server,
                    user
                } = response;

                if (server === null || user === null) {
                    // TODO: Rare edge case where the page was rendered while
                    // the FTP or SSH config was still complete, but was made
                    // incomplete until this was triggered. Technically we
                    // should return a proper translated message, but
                    // realistcally speaking almost will no one will ever see
                    // it.
                    return;
                }

                function delegate() {
                    if (passwordPrompt) {
                        const modalOptions = {};

                        modalOptions.callback = password => {
                            const input = document.createElement('input');

                            input.name = 'password';
                            input.type = 'hidden';
                            input.value = password;

                            form.append(input);
                            form.submit();
                        };

                        modalOptions.confirmLabel = t.continue;

                        modalOptions.text = privateKey
                            ? t.enterPasswordForDecryptingThePrivateKeyInXxx(privateKey)
                            : t.enterPasswordForUserXxx(user);

                        modalOptions.type = 'password';

                        hyperModal(modalOptions)
                    } else {
                        form.submit();
                    }
                }

                if (confirm) {
                    const modalOptions = {}

                    modalOptions.callback = delegate;
                    modalOptions.confirmLabel = t.sync;
                    modalOptions.text = `${t.reallySync} (${server})`;

                    hyperModal(modalOptions);
                } else {
                    delegate();
                }
            });
    });
}

const warnDiscardBeforeUnload = event => event.preventDefault();
let warnDiscardBound = [];
for (const form of document.querySelectorAll('form[data-warn-discard]')) {
    form.addEventListener('change', () => {
        if (!warnDiscardBound.length) {
            window.addEventListener('beforeunload', warnDiscardBeforeUnload);
        }
        if (!warnDiscardBound.includes(form)) {
            warnDiscardBound.push(form);
        }
    });
    form.addEventListener('submit', event => {
        if (warnDiscardBound.length === 1 && warnDiscardBound[0] === form) {
            window.removeEventListener('beforeunload', warnDiscardBeforeUnload)
        }
    });
}

fetchProcessing();
setInterval(fetchProcessing, 2000);

document.querySelector('#filter_tree').addEventListener('input', event => {
    const query = event.target.value.trim();

    const regex = query.length ?
        new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i') :
        null;

    for (const entry of document.querySelectorAll('nav a')) {
        if (
            regex === null ||
            regex.test(entry.querySelector('.title').innerHTML) ||
            regex.test(entry.querySelector('.path').innerHTML)
        ) {
            entry.style.display = null;
        } else {
            entry.style.display = 'none';
        }
    }
});

document.querySelector('#filter_tree').addEventListener('keydown', event => {
    if (event.key === 'Escape') {
        document.querySelector('#filter_tree').value = '';
    }
});

document.addEventListener('change', event => {
    const { target } = event;

    if (target.tagName == 'INPUT' && target.type == 'file') {
        target.form.submit();
    }
});

for (const usage of document.querySelectorAll('.usage')) {
    usage.addEventListener('change', () => {
        usage.form.submit();
    });
}

document.addEventListener('click', event => {
    const { target } = event;

    if (target.classList.contains('upload_button')) {
        target.form.querySelector('input').click();
        event.preventDefault();
    } else if (target.id == 'toggle-splash') {
        if (splashWrapper.open) {
            splashWrapper.close();
        } else {
            splashWrapper.show();
        }
    }
});

const qualityInput = document.querySelector('input#quality');

if (qualityInput) {
    qualityInput.addEventListener('input', () => {
        const qualityLabel = document.querySelector('#quality_wrapper span');
        const percent = Math.trunc(qualityInput.value * 100);
        qualityLabel.setAttribute('aria-valuenow', percent);
        qualityLabel.setAttribute('aria-valuetext', `${percent}%`);
        qualityLabel.style.left = `${percent * 0.95}%`;
        qualityLabel.textContent = `${percent}%`;
    });
}

const changeEditorLanguage = document.querySelector('#language_code');

if (changeEditorLanguage) {
    changeEditorLanguage.addEventListener('change', () => {
        if (splashWrapper?.open) {
            // When we switch language while the splash screen is open,
            // ensure the splash screen re-opens after the page refresh
            localStorage.setItem('hyper8ForceSplash', '');
        }
        changeEditorLanguage.form.redirect.value = location.pathname;
        changeEditorLanguage.form.submit();
    });
}

const changeEditorTheme = document.querySelector('#theme_key');

if (changeEditorTheme) {
    changeEditorTheme.addEventListener('change', () => {
        if (splashWrapper?.open) {
            // When we switch language while the splash screen is open,
            // ensure the splash screen re-opens after the page refresh
            localStorage.setItem('hyper8ForceSplash', '');
        }
        changeEditorTheme.form.redirect.value = location.pathname;
        changeEditorTheme.form.submit();
    });
}

const changeTreeViewOrder = document.querySelector('#tree-view-order');

if (changeTreeViewOrder) {
    changeTreeViewOrder.addEventListener('submit', () => {
        changeTreeViewOrder.redirect.value = location.pathname;
    });
}

if (splashWrapper) {
    const automaticSplash = splashWrapper.dataset.automatic !== undefined;
    const version = document.body.dataset.version;
    if ((automaticSplash && localStorage.getItem('hyper8SplashSeen') !== version) ||
        localStorage.getItem('hyper8ForceSplash') !== null) {
        splashWrapper.show();
        localStorage.removeItem('hyper8ForceSplash');
        localStorage.setItem('hyper8SplashSeen', version);
    }
}
