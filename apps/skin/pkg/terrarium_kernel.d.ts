/* tslint:disable */
/* eslint-disable */
/**
 * JS-facing world handle.
 */
export class JsWorld {
  free(): void;
  constructor();
  worldWidth(): number;
  worldHeight(): number;
  totalMass(): number;
  houseBurned(): number;
  spawnedMass(): number;
  tickCount(): number;
  /**
   * Spawn a cell. Returns cell id, or throws on error.
   */
  spawnCell(mass: number, x: number, y: number): number;
  /**
   * Install a text program on a cell.
   */
  setProgramText(cell_id: number, src: string): void;
  /**
   * Install raw bytecode on a cell.
   */
  setProgramBytes(cell_id: number, bytes: Uint8Array): void;
  /**
   * Compile text → bytecode (for skin tooling / demos).
   */
  static compileProgram(src: string): Uint8Array;
  /**
   * Dump inert mass from a cell (JSON-friendly). Conserves total_mass.
   */
  dumpMatter(cell_id: number, amount: number): number;
  tick(): void;
  /**
   * JSON snapshot for the canvas.
   */
  snapshot(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_jsworld_free: (a: number, b: number) => void;
  readonly jsworld_new: () => number;
  readonly jsworld_worldWidth: (a: number) => number;
  readonly jsworld_worldHeight: (a: number) => number;
  readonly jsworld_totalMass: (a: number) => number;
  readonly jsworld_houseBurned: (a: number) => number;
  readonly jsworld_spawnedMass: (a: number) => number;
  readonly jsworld_tickCount: (a: number) => number;
  readonly jsworld_spawnCell: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly jsworld_setProgramText: (a: number, b: number, c: number, d: number) => [number, number];
  readonly jsworld_setProgramBytes: (a: number, b: number, c: number, d: number) => [number, number];
  readonly jsworld_compileProgram: (a: number, b: number) => [number, number, number, number];
  readonly jsworld_dumpMatter: (a: number, b: number, c: number) => [number, number, number];
  readonly jsworld_tick: (a: number) => void;
  readonly jsworld_snapshot: (a: number) => [number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
