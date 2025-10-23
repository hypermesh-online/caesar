import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import routify from '@roxi/routify/vite-plugin';

export default defineConfig({
  plugins: [
    routify({
      routesDir: 'src/routes',
      dynamicImports: true
    }),
    svelte({
      compilerOptions: {
        dev: true
      }
    })
  ],
  server: {
    port: 3003,
    host: true
  },
  build: {
    target: 'esnext'
  }
});