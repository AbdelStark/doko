import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill'
import { NodeModulesPolyfillPlugin } from '@esbuild-plugins/node-modules-polyfill'
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'

export default ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')

  return defineConfig({
    plugins: [react()],
    resolve: {
      alias: {
        buffer: 'rollup-plugin-node-polyfills/polyfills/buffer-es6',
        stream: 'rollup-plugin-node-polyfills/polyfills/stream',
        events: 'rollup-plugin-node-polyfills/polyfills/events',
        util: 'rollup-plugin-node-polyfills/polyfills/util',
      },
      dedupe: ['react', 'react-dom']
    },
    define: {
      __RPC_URL__: JSON.stringify(env.VITE_RPC_URL || ''),
      __RPC_PORT__: JSON.stringify(env.VITE_RPC_PORT || ''),
      __RPC_USER__: JSON.stringify(env.VITE_RPC_USER || ''),
      __RPC_PASSWORD__: JSON.stringify(env.VITE_RPC_PASSWORD || ''),
      __RPC_WALLET__: JSON.stringify(env.VITE_RPC_WALLET || ''),
    },
    optimizeDeps: {
      esbuildOptions: {
        define: {
          global: 'globalThis',
        },
        plugins: [
          NodeGlobalsPolyfillPlugin({
            process: true,
            buffer: true,
          }),
          NodeModulesPolyfillPlugin(),
        ],
      },
    },
    build: {
      rollupOptions: {
        plugins: [rollupNodePolyFill()],
      },
    },
    server: {
      port: 3000,
      open: true,
      proxy: {
        '/wallet': {
          target: `http://${env.VITE_RPC_URL}:${env.VITE_RPC_PORT}`,
          changeOrigin: true,
          auth: `${env.VITE_RPC_USER}:${env.VITE_RPC_PASSWORD}`,
          secure: false,
        }
      }
    },
  })
} 