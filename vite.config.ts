import { defineConfig, type Plugin } from 'vite';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { mkdirSync, writeFileSync } from 'node:fs';
import { cp, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { handle } from './scripts/dev-api.ts';

const root = import.meta.dirname;
const web = resolve(root, 'apps/wfhub-web');
const generated = resolve(root, '.tanstack');
mkdirSync(generated, { recursive: true });
writeFileSync(resolve(generated, 'wfhub-tailwind.css'), '@import "tailwindcss";\n@source "../apps/wfhub-web/src";\n');

function localApi(): Plugin {
  return {
    name: 'wfhub-local-api',
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const path = req.url ?? '/';
        if (!path.startsWith('/api') && !path.startsWith('/_api-data/')) return next();
        try {
          const method = req.method ?? 'GET';
          const chunks: Buffer[] = [];
          for await (const chunk of req) chunks.push(Buffer.from(chunk));
          const body = method === 'GET' || method === 'HEAD' ? undefined : Buffer.concat(chunks);
          const headers = new Headers();
          for (const [key, value] of Object.entries(req.headers)) {
            if (Array.isArray(value)) value.forEach(item => headers.append(key, item));
            else if (value !== undefined) headers.set(key, value);
          }
          const response = await handle(new Request(new URL(path, 'http://localhost'), { method, headers, body }));
          res.writeHead(response.status, Object.fromEntries(response.headers));
          res.end(method === 'HEAD' ? undefined : Buffer.from(await response.arrayBuffer()));
        } catch (error) { next(error); }
      });
      server.middlewares.use((req, _res, next) => {
        if (req.url?.startsWith('/assets/') || req.url?.startsWith('/public/') || req.url === '/app-ads.txt') {
          req.url = `/@fs${web}${req.url}`;
        }
        next();
      });
    },
  };
}

function staticAssets(): Plugin {
  return {
    name: 'wfhub-static-assets',
    apply: 'build',
    async closeBundle() {
      const output = resolve(root, 'dist/client');
      await mkdir(output, { recursive: true });
      await Promise.all([
        cp(resolve(web, 'assets'), resolve(output, 'assets'), { recursive: true }),
        cp(resolve(web, 'public'), resolve(output, 'public'), { recursive: true }),
        cp(resolve(web, 'app-ads.txt'), resolve(output, 'app-ads.txt')),
      ]);
    },
  };
}

export default defineConfig(async ({ command, mode }) => {
  const useCloudflare = command === 'build' || mode === 'cloudflare';
  const cloudflare = useCloudflare
    ? (await import('@cloudflare/vite-plugin')).cloudflare({ viteEnvironment: { name: 'ssr' } }) : [];
  return {
    plugins: [localApi(), cloudflare, tanstackStart({ srcDirectory: 'apps/wfhub-web/src' }), tailwindcss(), react(), staticAssets()],
    server: { host: '127.0.0.1', port: 8000, strictPort: true },
  };
});
