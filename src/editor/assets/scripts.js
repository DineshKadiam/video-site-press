const processing = {};

function fetchProcessing() {
	fetch('/processing')
		.then(response => response.json())
		.then(response => {
			if (processing.lastUpdate === undefined || response.lastUpdate !== processing.lastUpdate) {
				const processingElement = document.querySelector('.processing');

				if (response.operations.length) {
					// TODO: This should also give feedback how many jobs are waiting to be processed (?)
					processingElement.innerHTML = `${response.operations.length} active background jobs`;
					processingElement.setAttribute('title', response.operations.map(op => op.label).join(', '));
				} else {
					processingElement.innerHTML = '';
					processingElement.removeAttribute('title', null);
				}

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
	}
});
