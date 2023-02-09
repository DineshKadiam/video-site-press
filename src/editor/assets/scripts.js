const processing = {};

function fetchProcessing() {
	fetch('processing')
		.then(response => response.json())
		.then(response => {
			if (!processing.updateAnnounced) {
				if (processing.lastUpdate === undefined) {
					processing.lastUpdate = response.lastUpdate;
				} else if (response.lastUpdate !== processing.lastUpdate) {
					// document.querySelector('.updates').innerHTML = 'Updates available, refresh the page to load them.';
					processing.updateAnnounced = true;
				}
			}

			for (const video of document.querySelectorAll('.video')) {
				let operations = [];

				for (const operation of response.operations) {
					if (operation.videoId === video.dataset.id) {
						operations.push(operation.label);
					}
				}

				if (operations.length) {
					video.classList.add('processing');
					video.setAttribute('title', operations.join(', '));
				} else {
					video.classList.remove('processing');
					video.removeAttribute('title', null);
				}
			}
		});
}

if (window.location.pathname === '/') {
	fetchProcessing();
	setInterval(fetchProcessing, 2000);
}

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
	}
});
