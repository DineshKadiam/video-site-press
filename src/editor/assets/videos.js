document.querySelector('input[type="search"]').addEventListener('input', event => {
    const query = event.target.value.trim();

    let regex = null;
    if (query.length) {
        regex = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
    }

    for (const video of document.querySelectorAll('.video')) {
        if (regex === null || regex.test(video.dataset.search)) {
            video.style.display = null;
        } else {
            video.style.display = 'none';
        }
    }
});