import { fileURLToPath, URL } from 'node:url';

import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

import AutoImport from 'unplugin-auto-import/vite';
import Components from 'unplugin-vue-components/vite';
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers';

// https://vitejs.dev/config/
export default defineConfig({
  envDir: 'env',
  // mode: 'development',
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
  },
  plugins: [
    vue(),
    AutoImport({
      dirs: ['./src/stores/modules'],
      resolvers: [ElementPlusResolver()],
      imports: ['vue', 'vue-router', 'pinia'],
      dts: './auto-imports.d.ts', // path to save dts
    }),
    Components({
      dirs: ['src/components'], // folders to be loaded on demand
      resolvers: [ElementPlusResolver()],
      dts: './components.d.ts', // path to save dts
    }),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
});
