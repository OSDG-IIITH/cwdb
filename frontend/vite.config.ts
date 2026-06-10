import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';

const base = process.env.BASE_PATH ?? '';

export default defineConfig({
	server: {
		proxy: {
			'/api': 'http://localhost:3000'
		}
	},
	plugins: [
		tailwindcss(),
		sveltekit(),
		SvelteKitPWA({
			registerType: 'autoUpdate',
			devOptions: { enabled: false },
			kit: {
				adapterFallback: '200.html'
			},
			manifest: {
				name: 'cwdb',
				short_name: 'cwdb',
				start_url: base || '/',
				display: 'standalone',
				theme_color: '#000000',
				background_color: '#ffffff',
				icons: [{ src: `${base}/favicon.svg`, sizes: 'any', type: 'image/svg+xml' }]
			},
			workbox: {
				globPatterns: ['**/*.{js,css,html,svg,ico,woff2}'],
				navigateFallback: `${base}/200.html`,
				navigateFallbackDenylist: [/^\/api/, /^\/data/]
			}
		})
	]
});
