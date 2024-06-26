const configureIcon = document.querySelector('#configure_icon');
const confirmationDialog = document.querySelector('#confirmation_dialog');
const deploymentElement = document.querySelector('#deployment');
const deploymentFeedback = document.querySelector('#deployment_feedback');
const deploymentIcon = document.querySelector('#deployment_icon');
const processingElement = document.querySelector('.processing');
const processingIcon = document.querySelector('#processing_icon');
const splashWrapper = document.querySelector('.splash_wrapper');

const processing = {};

const copyButtons = document.querySelectorAll('button[data-copy]');
const copyNotificationTimeouts = {};
if (copyButtons.length) {
    function copyToClipboard(button) {
        const notify = (success, content) => {
            if (content in copyNotificationTimeouts) {
                clearTimeout(copyNotificationTimeouts[content]);
                delete copyNotificationTimeouts[content];
            }

            const copyIcon = document.querySelector('#copy_icon').content.cloneNode(true);
            const feedbackIcon = document.querySelector(success ? '#success_icon' : '#failure_icon').content.cloneNode(true);

            const iconContainer = button.querySelector('.icon');

            iconContainer.replaceChildren(feedbackIcon);

            copyNotificationTimeouts[content] = setTimeout(() => iconContainer.replaceChildren(copyIcon), 3000);
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
    fetch('/processing')
        .then(response => response.json())
        .then(response => {
            const { deploymentInfo } = response;

            if (!deploymentInfo.configured) {
                if (!deploymentElement.querySelector('a[href="/configure-deployment"]')) {
                    const a = document.createElement('a');
                    a.href = '/configure-deployment';

                    a.append(configureIcon.content.cloneNode(true));

                    const text = document.createElement('span');
                    text.textContent = t.configureDeployment;
                    a.append(text);

                    deploymentElement.replaceChildren(a);
                }
            } else if (deploymentInfo.deployingSince !== null) {
                const a = document.createElement('a');
                a.href = '/deployment';

                a.append(deploymentIcon.content.cloneNode(true));

                const text = document.createElement('span');
                text.textContent = t.deploying;
                a.append(text);

                const durationText = document.createElement('span');
                durationText.textContent = formatTime(deploymentInfo.deployingSince);
                a.append(durationText);

                deploymentElement.replaceChildren(a);
            } else {
                if (!deploymentElement.querySelector('form[action="/deploy"]')) {
                    const form = document.createElement('form');
                    form.action = '/deploy';
                    form.method = 'post';

                    form.addEventListener('submit', event => {
                        event.preventDefault();
                        const baseUrl = document.body.dataset.baseUrl;
                        hyperConfirm(() => form.submit(), t.deploy, `${t.reallyDeploy} (${baseUrl})`);
                    });

                    const button = document.createElement('button');

                    button.append(deploymentIcon.content.cloneNode(true));

                    const text = document.createElement('span');
                    text.textContent = t.deploy;
                    button.append(text);

                    form.append(button);

                    deploymentElement.replaceChildren(form);
                }
            }

            if (deploymentFeedback) {
                deploymentFeedback.replaceChildren(deploymentInfo.feedback);
            }

            // TODO: This lastUpdate check is maybe not really needed anymore (what purpose does it serve now?)
            if (processing.lastUpdate === undefined || response.lastUpdate !== processing.lastUpdate) {

                let content = [processingIcon.content.cloneNode(true)];

                if (response.operations.length) {
                    text = document.createElement('div');

                    processingElement.classList.add('active');
                    for (const lastProgress of response.operations) {
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

                if (response.pendingCount > 0) {
                    const text = document.createElement('span');
                    text.textContent = `+${response.pendingCount}`;
                    content.push(text);
                }

                processingElement.replaceChildren(...content);

                processing.lastUpdate = response.lastUpdate;
            }
        });
}

function hyperConfirm(callback, label, text) {
    const dialog = confirmationDialog.content.cloneNode(true);

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
        localStorage.removeItem('hyper8SplashSeen');
        changeEditorLanguage.form.redirect.value = location.pathname;
        changeEditorLanguage.form.submit();
    });
}

const changeEditorTheme = document.querySelector('#theme_key');

if (changeEditorTheme) {
    changeEditorTheme.addEventListener('change', () => {
        localStorage.removeItem('hyper8SplashSeen');
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
    const version = document.body.dataset.version;
    if (localStorage.getItem('hyper8SplashSeen') !== version) {
        splashWrapper.show();
        localStorage.setItem('hyper8SplashSeen', version);
    }
}
