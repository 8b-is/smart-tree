/** @type {import('tailwindcss').Config} */
export default {
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'./src/**/*.{html,js,svelte,ts,jsx,tsx}',
		'./node_modules/@sveltejs/**/*.{html,js,svelte,ts}'
	],
	theme: {
		extend: {
			colors: {
				purple: {
					400: '#a855f7',
					500: '#9333ea',
					600: '#7e22ce',
					700: '#6b21a8',
					800: '#581c87',
					900: '#4c1d95'
				}
			}
		}
	},
	plugins: []
};