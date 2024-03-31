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
