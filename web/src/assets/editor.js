const configureIcon = document.querySelector('#configure_icon').content;
const confirmationDialog = document.querySelector('#confirmation_dialog').content;
const deploymentElement = document.querySelector('#deployment');
const deploymentFeedback = document.querySelector('#deployment_feedback');
const deploymentIcon = document.querySelector('#deployment_icon').content;
const processingElement = document.querySelector('.processing');
const processingIcon = document.querySelector('#processing_icon').content;
const reloadIcon = document.querySelector('#reload_icon').content;
const splashWrapper = document.querySelector('.splash_wrapper');

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
                deploymentInfo,
                globalJobReport,
                localJobReport
            } = response;

            if (jobs) {
                // When on the jobs page, update the local processing indicator widget
                updateJobs(jobs, globalJobReport);
            } else if (mediaJobs) {
                // When on a video's media page, update the local processing indicator widget
                updateMediaJobs(mediaJobs, localJobReport);
            }

            // Update the "Configure Deployment/Deploy/Deploying since" button in the header
            updateDeploymentButton(deploymentInfo);

            // Update the live deployment feedback on the deployment page (if we are on it)
            if (deploymentFeedback) {
                updateDeploymentPage(deploymentInfo);
            }

            // Update the processing information (Idle/Progress/etc.) in the header
            updateProcessingWidget(globalJobReport);
        });
}

function hyperConfirm(callback, label, text) {
    const dialog = confirmationDialog.cloneNode(true);

    dialog.querySelector('.confirm').textContent = label;
    dialog.querySelector('.text').textContent = text;

    document.body.appendChild(dialog);

    // For some reason .showModal and .remove would not work on dialog itself,
    // so we "re-query" the element from the DOM after having appended it.
    // Possibly follow up at some point why this happens.
    const appendedDialog = document.querySelector('.confirmation_wrapper')

    appendedDialog.querySelector('.cancel').addEventListener('click', () => appendedDialog.remove());
    appendedDialog.querySelector('.confirm').addEventListener('click', () => {
        appendedDialog.remove();
        callback();
    });

    appendedDialog.showModal();

    appendedDialog.querySelector('.confirm').focus();
}

function updateDeploymentButton(deploymentInfo) {
    if (!deploymentInfo.configured) {
        if (!deploymentElement.querySelector('a[href="/site/edit"]')) {
            const a = document.createElement('a');
            a.href = '/site/edit/';

            a.append(configureIcon.cloneNode(true));

            const text = document.createElement('span');
            text.textContent = t.configureDeployment;
            a.append(text);

            deploymentElement.replaceChildren(a);
        }
    } else if (deploymentInfo.deployingSince !== null) {
        const a = document.createElement('a');
        a.href = '/deployment';

        a.append(deploymentIcon.cloneNode(true));

        const text = document.createElement('span');
        text.textContent = t.deploying;
        a.append(text);

        const durationText = document.createElement('span');
        durationText.textContent = formatTime(deploymentInfo.deployingSince);
        a.append(durationText);

        deploymentElement.replaceChildren(a);
    } else if (!deploymentElement.querySelector('form[action="/deploy"]')) {
        const form = document.createElement('form');
        form.action = '/deploy';
        form.method = 'post';

        form.addEventListener('submit', event => {
            event.preventDefault();
            const baseUrl = document.body.dataset.baseUrl;
            hyperConfirm(() => form.submit(), t.deploy, `${t.reallyDeploy} (${baseUrl})`);
        });

        const button = document.createElement('button');

        button.append(deploymentIcon.cloneNode(true));

        const text = document.createElement('span');
        text.textContent = t.deploy;
        button.append(text);

        form.append(button);

        deploymentElement.replaceChildren(form);
    }
}

function updateDeploymentPage(deploymentInfo) {
    let feedback = deploymentInfo.feedback;

    if (deploymentInfo.deployingSince !== null) {
        feedback += '\n…';
    }

    deploymentFeedback.textContent = feedback;
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

for (const form of document.querySelectorAll('form[data-confirm]')) {
    form.addEventListener('submit', event => {
        event.preventDefault();

        const label = form.dataset.confirmLabel;
        const text = form.dataset.confirmText;

        hyperConfirm(() => form.submit(), label, text);
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
