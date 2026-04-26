/**
 * Web Worker Pool for Parallel WASM Simulations
 *
 * Manages a pool of web workers that run WASM simulations in parallel.
 * This enables 4-8x speedup for batch portfolio evaluations.
 * Supports model loading for faster optimization sweeps.
 */

import type {
  BatteryMode,
  OptimizerConfig,
  SweepResult,
  SweepPoint,
} from '../types';

// WASM-compatible cost params (depreciation_method is a string)
export type WasmCostParams = Record<string, number | string | boolean>;

export interface Portfolio {
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
}

export interface EvalResult {
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  lcoe: number;
  clean_match: number;
}

export interface ZoneProfiles {
  solar: Float64Array | number[];
  wind: Float64Array | number[];
  load: Float64Array | number[];
}

type BatteryModeString = 'Default' | 'PeakShaver' | 'Hybrid';

type WorkerMessage = {
  type: 'ready';
} | {
  type: 'results';
  results: EvalResult[];
} | {
  type: 'model_loaded';
  zone: string;
  batteryMode: BatteryModeString;
} | {
  type: 'sweep_results';
  result: SweepResult;
} | {
  type: 'error';
  error: string;
};

type WorkerRequest = {
  type: 'init';
  wasmPath: string;
} | {
  type: 'load_model';
  zone: string;
  batteryMode: BatteryModeString;
  bytes: ArrayBuffer;
} | {
  type: 'sweep';
  zone: string;
  targets: number[];
  profiles: ZoneProfiles;
  costs: WasmCostParams;
  config: OptimizerConfig;
  mode: BatteryModeString;
} | {
  type: 'evaluate';
  portfolios: Portfolio[];
  profiles: ZoneProfiles;
  costs: WasmCostParams;
  config: OptimizerConfig;
  mode: BatteryModeString;
};

export class SimulatorWorkerPool {
  private workers: Worker[] = [];
  private ready: Promise<void>;
  private isReady = false;
  private numWorkers: number;

  constructor(numWorkers = 8) {
    this.numWorkers = Math.min(numWorkers, navigator.hardwareConcurrency || 4);
    const workerPromises: Promise<void>[] = [];

    for (let i = 0; i < this.numWorkers; i++) {
      try {
        const worker = new Worker(
          new URL('./simulator-worker.ts', import.meta.url),
          { type: 'module' }
        );
        this.workers.push(worker);

        workerPromises.push(
          new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
              reject(new Error(`Worker ${i} failed to initialize`));
            }, 10000);

            worker.onmessage = (e: MessageEvent<WorkerMessage>) => {
              if (e.data.type === 'ready') {
                clearTimeout(timeout);
                resolve();
              }
            };

            worker.onerror = (e) => {
              clearTimeout(timeout);
              reject(new Error(`Worker ${i} error: ${e.message}`));
            };
          })
        );
      } catch (e) {
        console.warn(`Failed to create worker ${i}:`, e);
      }
    }

    this.ready = Promise.all(workerPromises)
      .then(() => {
        this.isReady = true;
        console.log(`Worker pool ready: ${this.workers.length} workers`);
      })
      .catch((e) => {
        console.error('Worker pool initialization failed:', e);
        throw e;
      });
  }

  /**
   * Evaluate a batch of portfolios in parallel across workers
   */
  async evaluateBatch(
    portfolios: Portfolio[],
    profiles: ZoneProfiles,
    costs: WasmCostParams,
    config: OptimizerConfig,
    mode: BatteryMode
  ): Promise<EvalResult[]> {
    await this.ready;

    if (this.workers.length === 0) {
      throw new Error('No workers available');
    }

    // Split portfolios across workers
    const chunkSize = Math.ceil(portfolios.length / this.workers.length);
    const chunks: Portfolio[][] = [];

    for (let i = 0; i < portfolios.length; i += chunkSize) {
      chunks.push(portfolios.slice(i, i + chunkSize));
    }

    // Dispatch to workers in parallel
    const promises = chunks.map((chunk, i) =>
      new Promise<EvalResult[]>((resolve, reject) => {
        const worker = this.workers[i % this.workers.length];

        const timeout = setTimeout(() => {
          reject(new Error(`Worker ${i} timed out`));
        }, 30000);

        const handler = (e: MessageEvent<WorkerMessage>) => {
          if (e.data.type === 'results') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            resolve(e.data.results);
          } else if (e.data.type === 'error') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            reject(new Error(e.data.error));
          }
        };

        worker.addEventListener('message', handler);

        // Convert BatteryMode enum to string
        const modeStr: BatteryModeString = mode === 0
          ? 'Default'
          : mode === 1
            ? 'PeakShaver'
            : 'Hybrid';

        const request: WorkerRequest = {
          type: 'evaluate',
          portfolios: chunk,
          profiles: {
            solar: Array.from(profiles.solar),
            wind: Array.from(profiles.wind),
            load: Array.from(profiles.load),
          },
          costs,
          config,
          mode: modeStr,
        };

        worker.postMessage(request);
      })
    );

    const results = await Promise.all(promises);
    return results.flat();
  }

  /**
   * Load a model into all workers
   */
  async loadModelInWorkers(
    zone: string,
    batteryMode: BatteryMode,
    bytes: ArrayBuffer
  ): Promise<void> {
    await this.ready;

    if (this.workers.length === 0) {
      throw new Error('No workers available');
    }

    // Convert BatteryMode enum to string
    const batteryModeStr: BatteryModeString = batteryMode === 0
      ? 'Default'
      : batteryMode === 1
        ? 'PeakShaver'
        : 'Hybrid';

    // Load model in all workers
    const promises = this.workers.map((worker, i) =>
      new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error(`Worker ${i} model load timed out`));
        }, 15000);

        const handler = (e: MessageEvent<WorkerMessage>) => {
          if (e.data.type === 'model_loaded') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            resolve();
          } else if (e.data.type === 'error') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            reject(new Error(e.data.error));
          }
        };

        worker.addEventListener('message', handler);

        const request: WorkerRequest = {
          type: 'load_model',
          zone,
          batteryMode: batteryModeStr,
          bytes,
        };

        worker.postMessage(request, [bytes.slice(0)]); // Transfer a copy
      })
    );

    await Promise.all(promises);
    console.log(`[WorkerPool] Model loaded in all ${this.workers.length} workers`);
  }

  /**
   * Run optimizer sweep across workers in parallel
   *
   * Splits targets across workers for parallel execution.
   */
  async runParallelSweep(
    zone: string,
    targets: number[],
    profiles: ZoneProfiles,
    costs: WasmCostParams,
    config: OptimizerConfig,
    batteryMode: BatteryMode
  ): Promise<SweepResult> {
    await this.ready;

    if (this.workers.length === 0) {
      throw new Error('No workers available');
    }

    // Convert BatteryMode enum to string
    const batteryModeStr: BatteryModeString = batteryMode === 0
      ? 'Default'
      : batteryMode === 1
        ? 'PeakShaver'
        : 'Hybrid';

    // Split targets across workers
    const chunkSize = Math.ceil(targets.length / this.workers.length);
    const chunks: number[][] = [];

    for (let i = 0; i < targets.length; i += chunkSize) {
      chunks.push(targets.slice(i, i + chunkSize));
    }

    const startTime = performance.now();

    // Dispatch to workers in parallel
    const promises = chunks.map((chunk, i) =>
      new Promise<SweepResult>((resolve, reject) => {
        const worker = this.workers[i % this.workers.length];

        const timeout = setTimeout(() => {
          reject(new Error(`Worker ${i} sweep timed out`));
        }, 60000);

        const handler = (e: MessageEvent<WorkerMessage>) => {
          if (e.data.type === 'sweep_results') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            resolve(e.data.result);
          } else if (e.data.type === 'error') {
            clearTimeout(timeout);
            worker.removeEventListener('message', handler);
            reject(new Error(e.data.error));
          }
        };

        worker.addEventListener('message', handler);

        const request: WorkerRequest = {
          type: 'sweep',
          zone,
          targets: chunk,
          profiles: {
            solar: Array.from(profiles.solar),
            wind: Array.from(profiles.wind),
            load: Array.from(profiles.load),
          },
          costs,
          config,
          mode: batteryModeStr,
        };

        worker.postMessage(request);
      })
    );

    const results = await Promise.all(promises);

    // Merge results from all workers
    const allPoints: SweepPoint[] = results.flatMap(r => r.points);

    // Sort by target to ensure correct order
    allPoints.sort((a, b) => a.target - b.target);

    const elapsed_ms = performance.now() - startTime;

    return {
      points: allPoints,
      elapsed_ms,
    };
  }

  /**
   * Get the number of available workers
   */
  getWorkerCount(): number {
    return this.workers.length;
  }

  /**
   * Check if the pool is ready
   */
  isPoolReady(): boolean {
    return this.isReady;
  }

  /**
   * Wait for the pool to be ready
   */
  async waitForReady(timeoutMs = 10000): Promise<boolean> {
    if (this.isReady) return true;

    try {
      await Promise.race([
        this.ready,
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error('Worker pool initialization timed out')), timeoutMs)
        ),
      ]);
      return true;
    } catch (e) {
      console.warn('[WorkerPool] Failed to initialize:', e);
      return false;
    }
  }

  /**
   * Terminate all workers
   */
  terminate(): void {
    for (const worker of this.workers) {
      worker.terminate();
    }
    this.workers = [];
    this.isReady = false;
  }
}

// Singleton instance
let workerPool: SimulatorWorkerPool | null = null;

/**
 * Get or create the global worker pool
 */
export function getWorkerPool(numWorkers = 8): SimulatorWorkerPool {
  if (!workerPool) {
    workerPool = new SimulatorWorkerPool(numWorkers);
  }
  return workerPool;
}

/**
 * Terminate the global worker pool
 */
export function terminateWorkerPool(): void {
  if (workerPool) {
    workerPool.terminate();
    workerPool = null;
  }
}
