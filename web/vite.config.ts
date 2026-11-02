import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import { initializeTheme } from './src/lib/theme'

export default defineConfig(async () => ({
  plugins: [
    {
      name: 'theme-boot',
      transformIndexHtml: () => [
        {
          tag: 'script',
          children: `(${initializeTheme.toString()})()`,
          injectTo: 'head-prepend'
        }
      ]
    },
    tanstackRouter({ autoCodeSplitting: true }),
    react(),
    tailwindcss()
  ],
  resolve: {
    tsconfigPaths: true
  },
  server: {
    host: true,
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true
      }
    }
  }
}))
