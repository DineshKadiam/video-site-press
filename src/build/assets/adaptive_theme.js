// TODO: Watch for changes in device color scheme preference and update at runtime?
const PREFERRED_COLOR_SCHEME = window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

const themeToggles = document.querySelector('#theme-toggles');
const toggleDark = document.querySelector('#toggle-dark');
const toggleLight = document.querySelector('#toggle-light');

let pinnedColorScheme = localStorage.getItem('HYPER8_PINNED_COLOR_SCHEME');

if (pinnedColorScheme !== null) {
	themeToggles.classList.add('pinned');
}

if ((pinnedColorScheme ?? PREFERRED_COLOR_SCHEME) === 'light') {
	document.body.classList.add('light');
}

function toggleColorScheme(clickedColorScheme) {
	if (pinnedColorScheme === null) {
		themeToggles.classList.add('pinned');
		document.body.classList.toggle('light', clickedColorScheme === 'light');
		localStorage.setItem('HYPER8_PINNED_COLOR_SCHEME', clickedColorScheme);
		pinnedColorScheme = clickedColorScheme;
	} else {
		if (clickedColorScheme === pinnedColorScheme) {
			if (PREFERRED_COLOR_SCHEME !== pinnedColorScheme) {
				document.body.classList.toggle('light', PREFERRED_COLOR_SCHEME === 'light');
			}

			themeToggles.classList.remove('pinned');
			localStorage.removeItem('HYPER8_PINNED_COLOR_SCHEME');
			pinnedColorScheme = null;
		} else {
			document.body.classList.toggle('light', clickedColorScheme === 'light');
			localStorage.setItem('HYPER8_PINNED_COLOR_SCHEME', pinnedColorScheme);
			pinnedColorScheme = clickedColorScheme;
		}
	}
}

toggleDark.addEventListener('click', () => toggleColorScheme('dark'));
toggleLight.addEventListener('click', () => toggleColorScheme('light'));