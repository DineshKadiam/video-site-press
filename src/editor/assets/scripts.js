const splashWrapper = document.querySelector('.splash_wrapper');

const processing = {};

function fetchProcessing() {
    fetch('/processing')
        .then(response => response.json())
        .then(response => {
            // TODO: This lastUpdate check is maybe not really needed anymore (what purpose does it serve now?)
            if (processing.lastUpdate === undefined || response.lastUpdate !== processing.lastUpdate) {
                const processingElement = document.querySelector('.processing');
                const processingIcon = document.querySelector('#processing_icon');
                const text = document.createElement('span');

                if (response.operations.length) {
                    processingElement.classList.add('active');
                    // TODO: This should also give feedback how many jobs are waiting to be processed
                    const progress = response.operations
                        .map(op => {
                            if (op.lastProgress) {
                                return `${Math.min(100, Math.trunc(op.lastProgress.frame / op.lastProgress.estimatedTotalFrames * 100))}%`;
                            } else {
                                return '--%'
                            }
                        })
                        .join(' | ');
                    text.textContent = `${t.processing} ${progress}`;
                } else {
                    processingElement.classList.remove('active');
                    text.textContent = t.idle;
                }

                processingElement.replaceChildren(
                    processingIcon.content.cloneNode(true),
                    text
                );

                processing.lastUpdate = response.lastUpdate;
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

const changeEditorLanguage = document.querySelector('#change_editor_language');

if (changeEditorLanguage) {
    changeEditorLanguage.addEventListener('change', () => {
        localStorage.removeItem('hyper8SplashSeen');
        changeEditorLanguage.form.redirect.value = location.pathname;
        changeEditorLanguage.form.submit();
    });
}

if (splashWrapper) {
    const version = document.body.dataset.version;
    if (localStorage.getItem('hyper8SplashSeen') !== version) {
        splashWrapper.show();
        localStorage.setItem('hyper8SplashSeen', version);
    }
}
