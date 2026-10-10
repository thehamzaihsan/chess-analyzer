const ENGINE_SCRIPT_URL = '/stockfish/stockfish-nnue-16-single.js';

export interface EngineHandle {
  send(cmd: string): void;
  onLine(cb: (line: string) => void): void;
  offLine(cb: (line: string) => void): void;
  terminate(): void;
}

class WorkerEngineHandle implements EngineHandle {
  private worker: Worker;
  private handlers: ((line: string) => void)[] = [];

  constructor(worker: Worker) {
    this.worker = worker;
    this.worker.onmessage = (e: MessageEvent) => {
      const line = typeof e.data === 'string' ? e.data : '';
      for (const h of this.handlers) h(line);
    };
  }

  send(cmd: string): void {
    this.worker.postMessage(cmd);
  }

  onLine(cb: (line: string) => void): void {
    this.handlers.push(cb);
  }

  offLine(cb: (line: string) => void): void {
    const i = this.handlers.indexOf(cb);
    if (i >= 0) this.handlers.splice(i, 1);
  }

  terminate(): void {
    this.worker.terminate();
  }
}

/**
 * Best-effort main-thread fallback: loads the engine script via a classic
 * <script> tag (rather than a Worker) and talks to the exported factory
 * directly. Used only when Worker construction fails entirely (e.g. a
 * restrictive CSP), per the "fall back to the main thread" requirement.
 */
interface StockfishMainThreadEngine {
  postMessage: (cmd: string) => void;
  addMessageListener: (cb: (line: string) => void) => void;
  terminate?: () => void;
}

class MainThreadEngineHandle implements EngineHandle {
  private handlers: ((line: string) => void)[] = [];
  private engine: StockfishMainThreadEngine | null = null;
  private pending: string[] = [];
  private ready: Promise<void>;

  constructor() {
    this.ready = new Promise<void>((resolve, reject) => {
      const script = document.createElement('script');
      script.src = ENGINE_SCRIPT_URL;
      script.onload = () => {
        const exports = (
          script as unknown as {
            _exports?: (opts: unknown) => Promise<StockfishMainThreadEngine | undefined>;
          }
        )._exports;
        if (!exports) {
          reject(new Error('stockfish script did not export a factory'));
          return;
        }
        exports({}).then((engine) => {
          if (!engine) {
            reject(new Error('stockfish factory returned nothing'));
            return;
          }
          this.engine = engine;
          engine.addMessageListener((line: string) => {
            for (const h of this.handlers) h(line);
          });
          for (const cmd of this.pending) engine.postMessage(cmd);
          this.pending = [];
          resolve();
        }, reject);
      };
      script.onerror = () => reject(new Error('failed to load stockfish script'));
      document.head.appendChild(script);
    });
  }

  send(cmd: string): void {
    if (this.engine) this.engine.postMessage(cmd);
    else this.pending.push(cmd);
  }

  onLine(cb: (line: string) => void): void {
    this.handlers.push(cb);
  }

  offLine(cb: (line: string) => void): void {
    const i = this.handlers.indexOf(cb);
    if (i >= 0) this.handlers.splice(i, 1);
  }

  terminate(): void {
    this.engine?.terminate?.();
  }
}

async function handshake(handle: EngineHandle): Promise<void> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('engine handshake timed out')), 10000);
    const onLine = (line: string) => {
      if (line.trim() === 'uciok') {
        clearTimeout(timeout);
        handle.send('setoption name Hash value 16');
        resolve();
      }
    };
    handle.onLine(onLine);
    handle.send('uci');
  });
}

export interface EnginePool {
  size: number;
  mode: 'worker' | 'main-thread';
  run<T>(job: (handle: EngineHandle) => Promise<T>): Promise<T>;
  terminate(): void;
}

async function createWorkerHandle(): Promise<EngineHandle> {
  const worker = new Worker(ENGINE_SCRIPT_URL);
  const handle = new WorkerEngineHandle(worker);
  await handshake(handle);
  return handle;
}

export async function createEnginePool(): Promise<EnginePool | null> {
  const hc = typeof navigator !== 'undefined' ? navigator.hardwareConcurrency || 2 : 2;
  const desired = Math.max(1, Math.min(4, hc - 1));

  const handles: EngineHandle[] = [];
  for (let i = 0; i < desired; i++) {
    try {
      handles.push(await createWorkerHandle());
    } catch {
      break;
    }
  }

  let mode: 'worker' | 'main-thread' = 'worker';
  if (handles.length === 0) {
    try {
      const handle = new MainThreadEngineHandle();
      await handshake(handle);
      handles.push(handle);
      mode = 'main-thread';
    } catch {
      return null;
    }
  }

  const busy = new Array(handles.length).fill(false);
  const waiters: ((idx: number) => void)[] = [];

  function acquire(): Promise<number> {
    return new Promise((resolve) => {
      const idx = busy.findIndex((b) => !b);
      if (idx !== -1) {
        busy[idx] = true;
        resolve(idx);
      } else {
        waiters.push(resolve);
      }
    });
  }

  function release(idx: number) {
    const waiter = waiters.shift();
    if (waiter) {
      waiter(idx); // idx stays busy, handed directly to the next waiter
    } else {
      busy[idx] = false;
    }
  }

  return {
    size: handles.length,
    mode,
    async run<T>(job: (handle: EngineHandle) => Promise<T>): Promise<T> {
      const idx = await acquire();
      try {
        return await job(handles[idx]);
      } finally {
        release(idx);
      }
    },
    terminate() {
      for (const h of handles) h.terminate();
    },
  };
}
