import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
    plugins: [svelte({ hot: !process.env.VITEST })],
    resolve: {
        conditions: ['mode=test', 'browser']
    },
    test: {
        environment: 'jsdom',
        setupFiles: ['./src/setupTest.ts'],
        globals: true,
    },
});
