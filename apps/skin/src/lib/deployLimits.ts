/** Must match `MAX_WASM_BYTES` in crates/server/src/main.rs */
export const MAX_WASM_BYTES = 64 * 1024;

export function formatWasmLimit(): string {
  return `${MAX_WASM_BYTES / 1024} KB`;
}
