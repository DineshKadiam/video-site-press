const PREFERRED_COLOR_SCHEME = window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
let currentColorScheme = localStorage.getItem('HYPER8_PINNED_COLOR_SCHEME') ?? PREFERRED_COLOR_SCHEME;

if (currentColorScheme === 'light') {
	document.body.classList.add('light');
}

function toggleColorScheme() {
	currentColorScheme = (currentColorScheme === 'dark' ? 'light' : 'dark');
	localStorage.setItem('HYPER8_PINNED_COLOR_SCHEME', currentColorScheme);
	document.body.classList.toggle('light', currentColorScheme === 'light');
}
