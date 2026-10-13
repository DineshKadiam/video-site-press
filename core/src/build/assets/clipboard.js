const copyIcon = document.querySelector('template[data-copy-icon]').content;
const failedIcon = document.querySelector('template[data-failed-icon]').content;
const successIcon = document.querySelector('template[data-success-icon]').content;

let endFeedbackEarly = null;

function copyFeedback(feedbackIcon, iconContainer) {
    if (endFeedbackEarly) { endFeedbackEarly(); }

    iconContainer.replaceChildren(feedbackIcon.cloneNode(true));

    function endFeedback() {
        iconContainer.replaceChildren(copyIcon.cloneNode(true));
        endFeedbackEarly = null;
    }

    const endFeedbackScheduled = setTimeout(endFeedback, 3000);

    endFeedbackEarly = () => {
        clearTimeout(endFeedbackScheduled);
        endFeedback();
    };
}

function copyToClipboard(button) {
    const content = button.dataset.content;
    const iconContainer = button.querySelector('.icon');

    navigator.clipboard
        .writeText(content)
        .then(() => copyFeedback(successIcon, iconContainer))
        .catch(_err => copyFeedback(failedIcon, iconContainer));
};

if (navigator.clipboard) {
    for (const button of document.querySelectorAll('[data-copy]')) {
        if (button.dataset.dynamicUrl !== undefined) {
            const thisPageUrl = window.location.href.split('#')[0]; // discard hash if present

            if (button.dataset.dynamicUrl === '') {
                button.dataset.content = thisPageUrl;
            } else {
                button.dataset.content = thisPageUrl + (thisPageUrl.endsWith('/') ? '' : '/') + button.dataset.dynamicUrl;
            }
        }

        button.addEventListener('click', () => copyToClipboard(button));
    }
} else {
    for (const button of document.querySelectorAll('[data-copy]')) {
        button.remove();
    }
}
