import handler from '@tanstack/react-start/server-entry';
import api from '../api/worker.ts';
import type { WorkerEnv } from '../api/feedback.ts';

export default {
  fetch(request: Request, env: WorkerEnv, ctx: ExecutionContext): Promise<Response> | Response {
    const path = new URL(request.url).pathname;
    if (path === '/api' || path.startsWith('/api/') || path.startsWith('/_api-data/')) {
      return api.fetch(request, env);
    }
    const startHandler = handler as { fetch(request: Request, env: WorkerEnv, ctx: ExecutionContext): Promise<Response> | Response };
    return startHandler.fetch(request, env, ctx);
  },
};
