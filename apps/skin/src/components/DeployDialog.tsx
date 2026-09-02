import { useEffect, useRef, useState } from "react";
import { EXAMPLE_PROGRAMS } from "../lib/examples";
import { formatWasmLimit, MAX_WASM_BYTES } from "../lib/deployLimits";
import {
  DEFAULT_RUST_SOURCE,
  SANDBOX_SCENARIOS,
  type CompileDiagnostic,
  type SandboxResult,
  type SandboxScenario,
} from "../lib/creatureEditor";
import { formatGlimString, GLIM_SCALE } from "../lib/glim";
import { postCompile, postSandboxRun } from "../lib/api";
import { authorLinks } from "../lib/authoringLinks";
import { GlimAmount } from "./GlimAmount";
import { RustEditor } from "./RustEditor";
import { SandboxPreview } from "./SandboxPreview";

type Props = {
  cell: { x: number; y: number } | null;
  minExtra: number;
  corpseEnergy: number;
  credits: number | null;
  busy: boolean;
  onDeploy: (code: string, extraEnergy: number, wasmB64?: string) => void;
  onClose: () => void;
};

async function readWasmFile(file: File): Promise<{ b64: string; name: string }> {
  if (!file.name.toLowerCase().endsWith(".wasm")) {
    throw new Error("Only .wasm files");
  }
  const buf = await file.arrayBuffer();
  if (buf.byteLength === 0) throw new Error("Empty file");
  if (buf.byteLength > MAX_WASM_BYTES) throw new Error(`WASM must be ≤ ${formatWasmLimit()}`);
  const bytes = new Uint8Array(buf);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]!);
  return { b64: btoa(binary), name: file.name };
}

export function DeployDialog({ cell, minExtra, corpseEnergy, credits, busy, onDeploy, onClose }: Props) {
  const [source, setSource] = useState(DEFAULT_RUST_SOURCE);
  const [wasmB64, setWasmB64] = useState<string | undefined>();
  const [wasmLabel, setWasmLabel] = useState<string | null>(null);
  const [extra, setExtra] = useState(minExtra);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [diagnostics, setDiagnostics] = useState<CompileDiagnostic[]>([]);
  const [testing, setTesting] = useState(false);
  const [sandbox, setSandbox] = useState<SandboxResult | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [scenario, setScenario] = useState<SandboxScenario>("open");
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!cell) return;
    setSource(DEFAULT_RUST_SOURCE);
    setWasmB64(undefined);
    setWasmLabel(null);
    setExtra(minExtra);
    setError(null);
    setAdvancedOpen(false);
    setDiagnostics([]);
    setSandbox(null);
    setFrameIndex(0);
    setScenario("open");
  }, [cell, minExtra]);

  useEffect(() => {
    if (!cell) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cell, onClose]);

  if (!cell) return null;

  const maxExtra = Math.max(minExtra, (credits ?? minExtra) - corpseEnergy);
  const totalEnergy = corpseEnergy + extra;
  const totalCost = totalEnergy;

  const applyWasm = async (file: File) => {
    try {
      const { b64, name } = await readWasmFile(file);
      setWasmB64(b64);
      setWasmLabel(name);
      setSandbox(null);
      setError(null);
    } catch (err) {
      setWasmB64(undefined);
      setWasmLabel(null);
      setError(err instanceof Error ? err.message : "Invalid WASM file");
    }
  };

  const runTest = async () => {
    setTesting(true);
    setError(null);
    setSandbox(null);
    setDiagnostics([]);
    try {
      let b64 = wasmB64;
      if (!b64) {
        const compiled = await postCompile("rust", source);
        setDiagnostics(compiled.diagnostics);
        if (!compiled.ok || !compiled.wasm_b64) {
          setError(compiled.diagnostics[0]?.message ?? "Compile failed");
          return;
        }
        b64 = compiled.wasm_b64;
        setWasmB64(b64);
        setWasmLabel("creature.wasm");
      }
      const result = await postSandboxRun(b64, scenario, 100);
      setSandbox(result);
      setFrameIndex(Math.max(0, result.frames.length - 1));
      if (!result.alive && result.death_reason) {
        setError(`Died at tick ${result.ticks_run}: ${result.death_reason.replace(/_/g, " ")}`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Test failed");
    } finally {
      setTesting(false);
    }
  };

  const submit = () => {
    if (!wasmB64 && !source.trim()) {
      setError("Write code and Test, or drop a .wasm file");
      return;
    }
    if (extra < minExtra) {
      setError(`Minimum extra is ${formatGlimString(minExtra)}`);
      return;
    }
    if (credits !== null && totalCost > credits) {
      setError("Not enough glims");
      return;
    }
    setError(null);
    const deploy = async () => {
      let b64 = wasmB64;
      if (!b64) {
        const compiled = await postCompile("rust", source);
        if (!compiled.ok || !compiled.wasm_b64) {
          setDiagnostics(compiled.diagnostics);
          setError(compiled.diagnostics[0]?.message ?? "Compile failed");
          return;
        }
        b64 = compiled.wasm_b64;
      }
      onDeploy("// creature.rs", extra, b64);
    };
    void deploy();
  };

  const canDeploy = !!wasmB64 || source.trim().length > 0;
  const locked = busy || testing;

  return (
    <div className="pointer-events-auto absolute inset-0 z-20 flex items-end justify-center p-2 sm:items-center sm:p-4">
      <button
        type="button"
        className="absolute inset-0 bg-black/40 backdrop-blur-[2px]"
        aria-label="Close deploy dialog"
        onClick={onClose}
      />

      <div className="deploy-panel relative flex max-h-[92vh] w-full max-w-4xl flex-col overflow-hidden">
        <div className="mb-2 flex shrink-0 items-baseline justify-between gap-3">
          <h2 className="text-sm font-medium text-white/85">Deploy creature</h2>
          <span className="font-mono text-[10px] text-white/35">
            ({cell.x}, {cell.y})
          </span>
        </div>

        <div className="grid min-h-0 flex-1 gap-3 overflow-y-auto sm:grid-cols-2">
          <div className="min-w-0 space-y-2">
            <div className="overflow-hidden rounded-lg border border-white/[0.08]">
              <RustEditor
                value={source}
                onChange={(v) => {
                  setSource(v);
                  setWasmB64(undefined);
                  setWasmLabel(null);
                  setSandbox(null);
                  if (error) setError(null);
                }}
                diagnostics={diagnostics}
                readOnly={locked}
              />
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <select
                value={scenario}
                disabled={locked}
                onChange={(e) => setScenario(e.target.value as SandboxScenario)}
                className="rounded-md border border-white/[0.08] bg-black/30 px-2 py-1 text-[10px] text-white/70"
              >
                {SANDBOX_SCENARIOS.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
              <button type="button" className="deploy-btn" disabled={locked} onClick={() => void runTest()}>
                {testing ? "Testing…" : "Test"}
              </button>
              <a
                href={authorLinks.docs}
                target="_blank"
                rel="noreferrer"
                className="text-[10px] text-white/30 hover:text-white/50"
              >
                SDK docs
              </a>
            </div>
          </div>

          <div className="min-w-0 space-y-2">
            <SandboxPreview result={sandbox} frameIndex={frameIndex} />
            {sandbox && (
              <>
                <input
                  type="range"
                  min={0}
                  max={Math.max(0, sandbox.frames.length - 1)}
                  value={frameIndex}
                  onChange={(e) => setFrameIndex(Number(e.target.value))}
                  className="w-full accent-biolume"
                />
                <div className="grid grid-cols-2 gap-2 font-mono text-[9px] text-white/45">
                  <span>Tick {sandbox.frames[frameIndex]?.tick ?? 0}</span>
                  <span>
                    Energy <GlimAmount amount={sandbox.frames[frameIndex]?.energy ?? 0} compact className="inline text-[9px]" />
                  </span>
                  <span>Spent (run) <GlimAmount amount={sandbox.bench.total_spent} compact className="inline text-[9px]" /></span>
                  <span>Per tick <GlimAmount amount={sandbox.bench.per_tick_avg} compact className="inline text-[9px]" /></span>
                </div>
              </>
            )}
          </div>
        </div>

        <div className="mt-2 flex shrink-0 items-center justify-between gap-2 rounded-lg border border-white/[0.06] bg-black/20 px-2.5 py-2">
          <label className="text-[11px] text-white/45" htmlFor="deploy-extra">
            Extra glims
          </label>
          <div className="flex items-center gap-2">
            <input
              id="deploy-extra"
              type="number"
              min={minExtra}
              max={maxExtra}
              step={GLIM_SCALE}
              value={extra}
              disabled={locked}
              onChange={(e) => setExtra(Number.parseInt(e.target.value, 10) || minExtra)}
              className="w-20 rounded-md border border-white/[0.08] bg-black/30 px-2 py-1 text-right font-mono text-[11px] text-white/75 outline-none focus:border-biolume/25"
            />
            <GlimAmount amount={totalEnergy} className="text-[10px] text-white/45" />
          </div>
        </div>

        <input
          ref={fileRef}
          type="file"
          accept=".wasm,application/wasm"
          className="hidden"
          disabled={locked}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) void applyWasm(file);
            e.target.value = "";
          }}
        />

        <button
          type="button"
          disabled={locked}
          onClick={() => fileRef.current?.click()}
          onDragOver={(e) => {
            e.preventDefault();
            if (!locked) setDragOver(true);
          }}
          onDragLeave={() => setDragOver(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragOver(false);
            if (locked) return;
            const file = e.dataTransfer.files[0];
            if (file) void applyWasm(file);
          }}
          className={`mt-2 shrink-0 rounded-lg border border-dashed px-3 py-2 text-left text-[10px] ${
            dragOver ? "border-biolume/40 bg-biolume/5" : "border-white/[0.1] bg-black/15"
          }`}
        >
          {wasmLabel ? (
            <span className="font-mono text-biolume/80">{wasmLabel}</span>
          ) : (
            <span className="text-white/45">Or drop prebuilt .wasm</span>
          )}
        </button>

        <button
          type="button"
          className="mt-1 text-left text-[10px] text-white/35 hover:text-white/55"
          onClick={() => setAdvancedOpen((v) => !v)}
        >
          {advancedOpen ? "▾" : "▸"} Advanced — WAT examples
        </button>

        {advancedOpen && (
          <div className="flex flex-wrap gap-1">
            {EXAMPLE_PROGRAMS.map((example) => (
              <button
                key={example.id}
                type="button"
                className="deploy-btn"
                title={example.description}
                disabled={locked}
                onClick={() => {
                  setWasmB64(example.wasmB64);
                  setWasmLabel(example.wasmB64 ? `${example.name}.wasm` : null);
                  setSandbox(null);
                  if (error) setError(null);
                }}
              >
                {example.name}
              </button>
            ))}
          </div>
        )}

        {error && <p className="mt-1 font-mono text-[10px] text-red-400/80">{error}</p>}

        <div className="mt-2 flex shrink-0 justify-end gap-1.5 pb-1">
          <button type="button" className="deploy-btn" onClick={onClose} disabled={locked}>
            Cancel
          </button>
          {canDeploy && (
            <button type="button" className="deploy-btn deploy-btn-primary" onClick={submit} disabled={locked}>
              Deploy · <GlimAmount amount={totalCost} className="text-[10px]" compact />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
