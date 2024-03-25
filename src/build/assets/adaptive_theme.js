const themeToggles = document.querySelector('#theme-toggles');
const toggleDark = document.querySelector('#toggle-dark');
const toggleLight = document.querySelector('#toggle-light');

const prefersDarkColorSchemeMediaQuery = matchMedia('(prefers-color-scheme: dark)');

let deviceColorScheme = prefersDarkColorSchemeMediaQuery.matches ? 'dark' : 'light';
let pinnedColorScheme = localStorage.getItem('HYPER8_PINNED_COLOR_SCHEME');

if (pinnedColorScheme !== null) {
	themeToggles.classList.add('pinned');
}

if ((pinnedColorScheme ?? deviceColorScheme) === 'light') {
	document.body.classList.add('light');
}

prefersDarkColorSchemeMediaQuery.addEventListener('change', ({ matches }) => {
	deviceColorScheme = matches ? 'dark' : 'light';

	if (pinnedColorScheme === null) {
		document.body.classList.toggle('light', deviceColorScheme === 'light');
	}
});

function toggleColorScheme(clickedColorScheme) {
	if (pinnedColorScheme === null) {
		themeToggles.classList.add('pinned');
		document.body.classList.toggle('light', clickedColorScheme === 'light');
		localStorage.setItem('HYPER8_PINNED_COLOR_SCHEME', clickedColorScheme);
		pinnedColorScheme = clickedColorScheme;
	} else {
		if (clickedColorScheme === pinnedColorScheme) {
			if (deviceColorScheme !== pinnedColorScheme) {
				document.body.classList.toggle('light', deviceColorScheme === 'light');
			}

			themeToggles.classList.remove('pinned');
			localStorage.removeItem('HYPER8_PINNED_COLOR_SCHEME');
			pinnedColorScheme = null;
		} else {
			document.body.classList.toggle('light', clickedColorScheme === 'light');
			localStorage.setItem('HYPER8_PINNED_COLOR_SCHEME', clickedColorScheme);
			pinnedColorScheme = clickedColorScheme;
		}
	}
}

toggleDark.addEventListener('click', () => toggleColorScheme('dark'));
toggleLight.addEventListener('click', () => toggleColorScheme('light'));
