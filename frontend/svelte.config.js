import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({ fallback: '200.html' }),
		paths: { base: process.env.BASE_PATH ?? '' },
		serviceWorker: {
			register: false
		}
	}
};

export default config;
