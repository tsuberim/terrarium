import { useEffect, useRef, useState, type DragEvent } from "react";
import { EXAMPLE_PROGRAMS } from "../lib/examples";
import { formatWasmLimit, MAX_WASM_BYTES } from "../lib/deployLimits";
import { formatGlimString, GLIM_SCALE } from "../lib/glim";
import { apiRoot } from "../lib/config";
import { authorLinks, openReplitWithEnv, resolveApiBase } from "../lib/authoringLinks";
import { GlimAmount } from "./GlimAmount";

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
  if (buf.byteLength === 0) {
    throw new Error("Empty file");
  }
  if (buf.byteLength > MAX_WASM_BYTES) {
    throw new Error(`WASM must be ≤ ${formatWasmLimit()}`);
  }
  const bytes = new Uint8Array(buf);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]!);
  }
  return { b64: btoa(binary), name: file.name };
}

export function DeployDialog({ cell, minExtra, corpseEnergy, credits, busy, onDeploy, onClose }: Props) {
  const [code, setCode] = useState("");
  const [wasmB64, setWasmB64] = useState<string | undefined>();
  const [wasmLabel, setWasmLabel] = useState<string | null>(null);
  const [extra, setExtra] = useState(minExtra);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [replitCopied, setReplitCopied] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!cell) return;
    setCode("");
    setWasmB64(undefined);
    setWasmLabel(null);
    setExtra(minExtra);
    setError(null);
    setAdvancedOpen(false);
    setReplitCopied(false);
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
  const apiBase = resolveApiBase(apiRoot());

  const applyWasm = async (file: File) => {
    try {
      const { b64, name } = await readWasmFile(file);
      setWasmB64(b64);
      setWasmLabel(name);
      setCode(`(module ; ${name})`);
      setError(null);
    } catch (err) {
      setWasmB64(undefined);
      setWasmLabel(null);
      setError(err instanceof Error ? err.message : "Invalid WASM file");
    }
  };

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    if (busy) return;
    const file = e.dataTransfer.files[0];
    if (file) void applyWasm(file);
  };

  const launchReplit = async () => {
    await openReplitWithEnv({ apiBase, x: cell.x, y: cell.y, energy: totalEnergy });
    setReplitCopied(true);
  };

  const submit = () => {
    if (!code.trim()) {
      setError("Drop a .wasm file or paste WAT under Advanced");
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
    onDeploy(code, extra, wasmB64);
  };

  const canSubmit = !!wasmB64 || (advancedOpen && code.trim());

  return (
    <div className="pointer-events-auto absolute inset-0 z-20 flex items-end justify-center p-3 sm:items-center sm:p-4">
      <button
        type="button"
        className="absolute inset-0 bg-black/40 backdrop-blur-[2px]"
        aria-label="Close deploy dialog"
        onClick={onClose}
      />

      <div className="deploy-panel relative w-full max-w-lg">
        <div className="mb-2 flex items-baseline justify-between gap-3">
          <h2 className="text-sm font-medium text-white/85">Deploy creature</h2>
          <span className="font-mono text-[10px] text-white/35">
            ({cell.x}, {cell.y})
          </span>
        </div>

        <div className="mb-3 rounded-lg border border-biolume/15 bg-biolume/[0.04] p-3">
          <p className="mb-2 text-[11px] leading-relaxed text-white/50">
            Code in Replit — edit Zig, press Run to build and deploy here.
          </p>
          <button
            type="button"
            className="deploy-btn deploy-btn-primary w-full"
            disabled={busy}
            onClick={() => void launchReplit()}
          >
            Open in Replit
          </button>
          {replitCopied && (
            <p className="mt-1.5 font-mono text-[9px] text-biolume/70">
              .env copied — paste at sdk/zig/.env, add API key from Keys
            </p>
          )}
          <a
            href={authorLinks.docs}
            target="_blank"
            rel="noreferrer"
            className="mt-2 block text-center text-[10px] text-white/30 hover:text-white/50"
          >
            Docs → hello world
          </a>
        </div>

        <div className="mb-2 flex items-center justify-between gap-3 rounded-lg border border-white/[0.06] bg-black/20 px-2.5 py-2">
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
              disabled={busy}
              onChange={(e) => {
                setExtra(Number.parseInt(e.target.value, 10) || minExtra);
                if (error) setError(null);
              }}
              className="w-20 rounded-md border border-white/[0.08] bg-black/30 px-2 py-1 text-right font-mono text-[11px] text-white/75 outline-none focus:border-biolume/25"
            />
            <span className="text-[10px] text-white/30">=</span>
            <GlimAmount amount={totalEnergy} className="text-[10px] text-white/45" />
          </div>
        </div>

        <input
          ref={fileRef}
          type="file"
          accept=".wasm,application/wasm"
          className="hidden"
          disabled={busy}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) void applyWasm(file);
            e.target.value = "";
          }}
        />

        <button
          type="button"
          disabled={busy}
          onClick={() => fileRef.current?.click()}
          onDragOver={(e) => {
            e.preventDefault();
            if (!busy) setDragOver(true);
          }}
          onDragLeave={() => setDragOver(false)}
          onDrop={onDrop}
          className={`mb-2 w-full rounded-lg border border-dashed px-3 py-2.5 text-left transition-colors ${
            dragOver
              ? "border-biolume/40 bg-biolume/5"
              : "border-white/[0.1] bg-black/15 hover:border-white/20"
          }`}
        >
          <p className="text-[11px] text-white/55">
            {wasmLabel ? (
              <>
                <span className="font-mono text-biolume/80">{wasmLabel}</span>
                <span className="text-white/30"> · drop another to replace</span>
              </>
            ) : (
              <>Or drop a prebuilt .wasm file</>
            )}
          </p>
          <p className="mt-0.5 font-mono text-[9px] text-white/25">Max {formatWasmLimit()}</p>
        </button>

        <button
          type="button"
          className="mb-2 w-full text-left text-[10px] text-white/35 hover:text-white/55"
          onClick={() => setAdvancedOpen((v) => !v)}
        >
          {advancedOpen ? "▾" : "▸"} Advanced — paste WAT
        </button>

        {advancedOpen && (
          <div className="mb-2 space-y-2">
            <div className="flex flex-wrap gap-1">
              {EXAMPLE_PROGRAMS.map((example) => (
                <button
                  key={example.id}
                  type="button"
                  className="deploy-btn"
                  title={example.description}
                  disabled={busy}
                  onClick={() => {
                    setCode(example.code);
                    setWasmB64(example.wasmB64);
                    setWasmLabel(example.wasmB64 ? `${example.name}.wasm` : null);
                    if (error) setError(null);
                  }}
                >
                  {example.name}
                </button>
              ))}
            </div>
            <textarea
              value={code}
              onChange={(e) => {
                setCode(e.target.value);
                setWasmB64(undefined);
                setWasmLabel(null);
                if (error) setError(null);
              }}
              placeholder={'(module\n  (import "terrarium" "sleep" (func $sleep))\n  (func (export "tick") (call $sleep))\n)'}
              spellCheck={false}
              rows={8}
              className="deploy-input"
            />
          </div>
        )}

        {error && <p className="mt-1.5 font-mono text-[10px] text-red-400/80">{error}</p>}

        <div className="mt-3 flex justify-end gap-1.5">
          <button type="button" className="deploy-btn" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          {canSubmit && (
            <button type="button" className="deploy-btn deploy-btn-primary" onClick={submit} disabled={busy}>
              Deploy · <GlimAmount amount={totalCost} className="text-[10px]" compact />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
