/* tslint:disable */
/* eslint-disable */

export class MidiPlayer {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  set_volume(volume: number): void;
  current_bar(): number;
  set_display_range(range_sec: number): void;
  static new(): MidiPlayer;
  play(): void;
  skip(delta: number, clear_sounds: boolean): void;
  stop(): void;
  tick(delta_time: number): void;
  ready(): boolean;
  render(context: CanvasRenderingContext2D, left: number, top: number, width: number, height: number): void;
  volume(): number;
  num_bars(): number;
  seek_bar(bar: number, clear_sounds: boolean): void;
  load_midi(file: File): Promise<void>;
  seek_time(time: number, clear_sounds: boolean): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_midiplayer_free: (a: number, b: number) => void;
  readonly midiplayer_current_bar: (a: number) => number;
  readonly midiplayer_load_midi: (a: number, b: any) => any;
  readonly midiplayer_new: () => [number, number, number];
  readonly midiplayer_num_bars: (a: number) => number;
  readonly midiplayer_play: (a: number) => void;
  readonly midiplayer_ready: (a: number) => number;
  readonly midiplayer_render: (a: number, b: any, c: number, d: number, e: number, f: number) => [number, number];
  readonly midiplayer_seek_bar: (a: number, b: number, c: number) => void;
  readonly midiplayer_seek_time: (a: number, b: number, c: number) => void;
  readonly midiplayer_set_display_range: (a: number, b: number) => void;
  readonly midiplayer_set_volume: (a: number, b: number) => void;
  readonly midiplayer_skip: (a: number, b: number, c: number) => void;
  readonly midiplayer_stop: (a: number) => void;
  readonly midiplayer_tick: (a: number, b: number) => [number, number];
  readonly midiplayer_volume: (a: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__hce19deebc5ffd07f: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hda381aeee11983f2: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h868dfb7b2863349d: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
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
