import { useCallback, useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { createPortal } from "react-dom";
import { formatWasmLimit, MAX_WASM_BYTES } from "../lib/deployLimits";
import {
  DEFAULT_RUST_SOURCE,
  parseScenarios,
  type CompileDiagnostic,
} from "../lib/creatureEditor";
import { formatGlimString, GLIM_SCALE } from "../lib/glim";
import { postCompile, postSandboxRun } from "../lib/api";
import { authorLinks } from "../lib/authoringLinks";
import { GlimAmount } from "./GlimAmount";
import { RustEditor } from "./RustEditor";
import { WorldCanvas } from "./WorldCanvas";
import { useSandboxPlayback } from "../hooks/useSandboxPlayback";
import { clampStudioWidthPct, clampStudioCodeHeightPct } from "../lib/viewerPrefs";
import { mapPlayback, type StudioE2eSlice } from "../lib/e2eBridge";

type Props = {
  open: boolean;
  widthPct: number;
  onWidthChange: (pct: number) => void;
  codeHeightPct: number;
  onCodeHeightChange: (pct: number) => void;
  cell: { x: number; y: number } | null;
  minExtra: number;
  corpseEnergy: number;
  credits: number | null;
  busy: boolean;
  tickHz: number;
  senseRange?: number;
  canCheck?: boolean;
  onDeploy: (x: number, y: number, code: string, extraEnergy: number, wasmB64?: string) => void;
  onDeployDialogChange?: (open: boolean) => void;
  onE2eSliceChange?: (slice: StudioE2eSlice) => void;
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

export function CreatureStudio({
  open,
  widthPct,
  onWidthChange,
  codeHeightPct,
  onCodeHeightChange,
  cell,
  minExtra,
  corpseEnergy,
  credits,
  busy,
  tickHz,
  senseRange = 5,
  canCheck = false,
  onDeploy,
  onDeployDialogChange,
  onE2eSliceChange,
  onClose,
}: Props) {
  const [source, setSource] = useState(DEFAULT_RUST_SOURCE);
  const [wasmB64, setWasmB64] = useState<string | undefined>();
  const [wasmLabel, setWasmLabel] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [diagnostics, setDiagnostics] = useState<CompileDiagnostic[]>([]);
  const [testing, setTesting] = useState(false);
  const [testLoopActive, setTestLoopActive] = useState(false);
  const [testLoopIndex, setTestLoopIndex] = useState(0);
  const [deployOpen, setDeployOpen] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const wasmForLoopRef = useRef<string | undefined>(undefined);
  const resizingRef = useRef(false);
  const splitResizingRef = useRef(false);
  const deployDialogPrevRef = useRef<boolean | null>(null);

  const startResize = (e: ReactPointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    resizingRef.current = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);

    const onMove = (ev: PointerEvent) => {
      if (!resizingRef.current) return;
      const pct = clampStudioWidthPct((ev.clientX / window.innerWidth) * 100);
      onWidthChange(pct);
    };
    const onUp = () => {
      resizingRef.current = false;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const startSplitResize = (e: ReactPointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    splitResizingRef.current = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);

    const onMove = (ev: PointerEvent) => {
      if (!splitResizingRef.current || !bodyRef.current) return;
      const rect = bodyRef.current.getBoundingClientRect();
      const pct = clampStudioCodeHeightPct(((ev.clientY - rect.top) / rect.height) * 100);
      onCodeHeightChange(pct);
    };
    const onUp = () => {
      splitResizingRef.current = false;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const scenarios = useMemo(() => parseScenarios(source), [source]);
  const activeScenario = scenarios[testLoopIndex] ?? scenarios[0];

  const {
    result: sandbox,
    frameIndex,
    playback,
    creaturesLiveRef,
    tilesLiveRef,
    runtimeRef,
    loadResult,
    seek,
    play,
    pause,
    stop,
  } = useSandboxPlayback(tickHz);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !deployOpen) onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose, deployOpen]);

  useEffect(() => {
    if (deployDialogPrevRef.current === deployOpen) return;
    deployDialogPrevRef.current = deployOpen;
    onDeployDialogChange?.(deployOpen);
  }, [deployOpen, onDeployDialogChange]);

  useEffect(() => {
    onE2eSliceChange?.({
      testing,
      wasmReady: !!wasmB64,
      playback: mapPlayback(playback),
      error,
    });
  }, [testing, wasmB64, playback, error, onE2eSliceChange]);

  useEffect(() => {
    if (!open || !canCheck || busy || testing) return;
    const t = window.setTimeout(() => {
      void postCompile("rust", source)
        .then((compiled) => setDiagnostics(compiled.diagnostics))
        .catch(() => {
          /* offline / auth */
        });
    }, 900);
    return () => window.clearTimeout(t);
  }, [source, open, canCheck, busy, testing]);

  const locked = busy || testing;

  const applyWasm = async (file: File) => {
    try {
      const { b64, name } = await readWasmFile(file);
      setWasmB64(b64);
      setWasmLabel(name);
      loadResult(null);
      setError(null);
    } catch (err) {
      setWasmB64(undefined);
      setWasmLabel(null);
      setError(err instanceof Error ? err.message : "Invalid WASM file");
    }
  };

  const ensureWasm = useCallback(async (): Promise<string | null> => {
    if (wasmB64) return wasmB64;
    const compiled = await postCompile("rust", source);
    setDiagnostics(compiled.diagnostics);
    if (!compiled.ok || !compiled.wasm_b64) {
      return null;
    }
    setWasmB64(compiled.wasm_b64);
    setWasmLabel("creature.wasm");
    return compiled.wasm_b64;
  }, [source, wasmB64]);

  const runScenario = useCallback(
    async (b64: string, index: number) => {
      const sc = scenarios[index];
      if (!sc) return;
      setTestLoopIndex(index);
      const result = await postSandboxRun(b64, sc.id, 100);
      loadResult(result);
      if (!result.alive && result.death_reason) {
        setError(`${sc.label}: died at tick ${result.ticks_run} (${result.death_reason.replace(/_/g, " ")})`);
      } else {
        setError(null);
      }
      play();
    },
    [loadResult, play, scenarios],
  );

  const startTestLoop = async () => {
    setTesting(true);
    setError(null);
    setDiagnostics([]);
    stop();
    setTestLoopActive(true);
    setTestLoopIndex(0);
    try {
      const b64 = await ensureWasm();
      if (!b64) {
        setTestLoopActive(false);
        return;
      }
      wasmForLoopRef.current = b64;
      await runScenario(b64, 0);
    } catch (err) {
      setTestLoopActive(false);
      setError(err instanceof Error ? err.message : "Test failed");
    } finally {
      setTesting(false);
    }
  };

  const stopTestLoop = () => {
    setTestLoopActive(false);
    stop();
  };

  useEffect(() => {
    if (!testLoopActive || !sandbox?.frames.length || playback !== "paused") return;
    if (frameIndex < sandbox.frames.length - 1) return;

    const b64 = wasmForLoopRef.current;
    if (!b64 || !scenarios.length) return;

    const next = (testLoopIndex + 1) % scenarios.length;
    if (next === testLoopIndex) {
      setTestLoopActive(false);
      return;
    }
    void runScenario(b64, next);
  }, [testLoopActive, sandbox, playback, frameIndex, testLoopIndex, scenarios, runScenario]);

  const canDeploy = !!wasmB64 || source.trim().length > 0;
  const worldTick = sandbox?.frames[frameIndex]?.tick ?? 0;

  return (
    <div
      className={`studio-shell ${open ? "studio-shell-open" : "studio-shell-closed"}${deployOpen ? " studio-shell-passthrough" : ""}`}
      style={{ width: `${widthPct}%` }}
      aria-hidden={!open}
    >
      <div className="studio-toolbar">
        <div className="flex min-w-0 items-center gap-2">
          <h2 className="shrink-0 text-sm font-medium text-white/85">Creature studio</h2>
          <a
            href={authorLinks.docsHome}
            target="_blank"
            rel="noreferrer"
            className="flex shrink-0 items-center gap-1 px-1 text-[10px] text-white/40 hover:text-white/70"
            title="Documentation"
          >
            <LinkIcon />
            <span>docs</span>
          </a>
        </div>
        <div className="flex shrink-0 items-center gap-1">
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
            className="studio-icon-btn"
            disabled={locked}
            title={wasmLabel ? `Uploaded: ${wasmLabel}` : "Upload .wasm"}
            onClick={() => fileRef.current?.click()}
          >
            <UploadIcon />
          </button>
          <button
            type="button"
            className="deploy-btn deploy-btn-primary flex items-center gap-1 px-2.5"
            disabled={locked || !canDeploy}
            onClick={() => setDeployOpen(true)}
            title="Deploy creature"
            data-testid="e2e-studio-deploy"
          >
            <DeployIcon />
            <span>Deploy</span>
          </button>
          <button
            type="button"
            className="studio-icon-btn"
            onClick={onClose}
            disabled={locked}
            title="Close"
            data-testid="e2e-studio-close"
          >
            <CloseIcon />
          </button>
        </div>
      </div>

      {deployOpen &&
        createPortal(
          <DeployDialog
            cell={cell}
            minExtra={minExtra}
            corpseEnergy={corpseEnergy}
            credits={credits}
            busy={busy}
            onClose={() => setDeployOpen(false)}
            onConfirm={async (x, y, extra) => {
              if (!canDeploy) {
                setError("Write code and Test, or upload a .wasm file");
                return;
              }
              setError(null);
              const b64 = await ensureWasm();
              if (!b64) return;
              onDeploy(x, y, "// creature.rs", extra, b64);
              setDeployOpen(false);
            }}
          />,
          document.body,
        )}

      <div className="studio-body" ref={bodyRef}>
        <div className="studio-code-pane" style={{ flex: `0 0 ${codeHeightPct}%` }}>
          <div className="studio-editor-wrap">
            <RustEditor
              value={source}
              height="100%"
              onChange={(v) => {
                setSource(v);
                setWasmB64(undefined);
                setWasmLabel(null);
                loadResult(null);
                setTestLoopActive(false);
                if (error) setError(null);
              }}
              diagnostics={diagnostics}
              readOnly={locked}
            />
          </div>

          <div className="studio-controls">
            <button
              type="button"
              className="deploy-btn"
              disabled={locked || testLoopActive}
              onClick={() => void startTestLoop()}
              title="Run all scenarios"
              data-testid="e2e-studio-test"
            >
              {testing ? "Testing…" : "Test"}
            </button>
            <div className="flex items-center gap-0.5">
              <button
                type="button"
                className="studio-icon-btn"
                disabled={!sandbox?.frames.length || locked}
                title={playback === "playing" ? "Pause" : "Play"}
                aria-label={playback === "playing" ? "Pause" : "Play"}
                data-testid="e2e-studio-play"
                onClick={() => (playback === "playing" ? pause() : play())}
              >
                {playback === "playing" ? <PauseIcon /> : <PlayIcon />}
              </button>
              <button
                type="button"
                className="studio-icon-btn"
                disabled={!sandbox?.frames.length || locked}
                title="Stop"
                aria-label="Stop"
                data-testid="e2e-studio-stop"
                onClick={testLoopActive ? stopTestLoop : stop}
              >
                <StopIcon />
              </button>
            </div>
            {wasmLabel && (
              <span className="truncate font-mono text-[9px] text-biolume/70" title={wasmLabel}>
                {wasmLabel}
              </span>
            )}
          </div>

          {sandbox && (
            <div className="studio-scrub">
              <input
                type="range"
                min={0}
                max={Math.max(0, sandbox.frames.length - 1)}
                value={frameIndex}
                disabled={playback === "playing"}
                onChange={(e) => seek(Number(e.target.value))}
                className="w-full accent-biolume"
              />
              <div className="grid grid-cols-2 gap-2 font-mono text-[9px] text-white/45">
                <span>Tick {sandbox.frames[frameIndex]?.tick ?? 0}</span>
                <span>
                  Energy <GlimAmount amount={sandbox.frames[frameIndex]?.energy ?? 0} compact className="inline text-[9px]" />
                </span>
                <span>
                  Spent <GlimAmount amount={sandbox.bench.total_spent} compact className="inline text-[9px]" />
                </span>
                <span>
                  / tick <GlimAmount amount={sandbox.bench.per_tick_avg} compact className="inline text-[9px]" />
                </span>
              </div>
            </div>
          )}

          {error && !diagnostics.some((d) => d.level === "error") && (
            <p className="font-mono text-[10px] text-red-400/80">{error}</p>
          )}
        </div>

        <div
          className="studio-split-handle"
          onPointerDown={startSplitResize}
          title="Drag to resize"
          aria-hidden
        />

        <div className="studio-preview-pane">
          {testLoopActive && activeScenario && (
            <div className="absolute left-2 top-2 z-10 rounded-md border border-white/[0.08] bg-black/60 px-2 py-1 text-[10px] text-white/70 backdrop-blur-sm">
              Running: <span className="text-biolume/90">{activeScenario.label}</span>
            </div>
          )}
          {!sandbox ? (
            <div className="flex h-full items-center justify-center text-[11px] text-white/30">
              Run Test to preview scenarios
            </div>
          ) : (
            <WorldCanvas
              creaturesLiveRef={creaturesLiveRef}
              tilesLiveRef={tilesLiveRef}
              canDeploy={false}
              userUid="sandbox"
              senseRange={senseRange}
              view="god"
              followId={null}
              initialZoom={1.2}
              onCellSelect={() => {}}
              runtimeRef={runtimeRef}
              worldTick={worldTick}
              tickHz={tickHz}
            />
          )}
        </div>
      </div>
      <div
        className="studio-resize-handle"
        onPointerDown={startResize}
        title="Drag to resize"
        aria-hidden
      />
    </div>
  );
}

function DeployDialog({
  cell,
  minExtra,
  corpseEnergy,
  credits,
  busy,
  onClose,
  onConfirm,
}: {
  cell: { x: number; y: number } | null;
  minExtra: number;
  corpseEnergy: number;
  credits: number | null;
  busy: boolean;
  onClose: () => void;
  onConfirm: (x: number, y: number, extra: number) => void | Promise<void>;
}) {
  const [extra, setExtra] = useState(minExtra);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setExtra(minExtra);
  }, [minExtra]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const x = cell?.x ?? null;
  const y = cell?.y ?? null;
  const maxExtra = Math.max(minExtra, (credits ?? minExtra) - corpseEnergy);
  const totalCost = corpseEnergy + extra;
  const canAfford = credits === null || totalCost <= credits;
  const hasCell = x !== null && y !== null;
  const valid = hasCell && extra >= minExtra && canAfford;

  const submit = async () => {
    if (!valid || x === null || y === null) return;
    setSubmitting(true);
    try {
      await onConfirm(x, y, extra);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="pointer-events-none fixed inset-0 z-[70] flex items-center justify-center p-4">
      <div className="pointer-events-none absolute inset-0 bg-black/45 backdrop-blur-[2px]" aria-hidden />
      <div className="deploy-panel pointer-events-auto relative z-10 w-full max-w-sm">
          <div className="mb-2 flex items-start justify-between gap-3">
            <div>
              <h3 className="text-sm font-medium text-white/85">Deploy creature</h3>
              <p className="mt-1 text-[10px] text-white/40">Click a cell on the map to choose coordinates.</p>
            </div>
            <button type="button" className="studio-icon-btn shrink-0" onClick={onClose} title="Close">
              <CloseIcon />
            </button>
          </div>

          <div
            data-testid="e2e-deploy-location"
            className={`mt-3 rounded-lg border px-3 py-2.5 ${
              hasCell ? "border-biolume/25 bg-biolume/5" : "border-white/[0.08] bg-black/20"
            }`}
          >
            <p className="text-[10px] text-white/45">Location</p>
            {hasCell ? (
              <p className="mt-0.5 font-mono text-[13px] text-biolume/90">
                ({x}, {y})
              </p>
            ) : (
              <p className="mt-0.5 text-[11px] text-white/35">No cell selected — pick on the map →</p>
            )}
          </div>

          <div className="mt-3 space-y-2 rounded-lg border border-white/[0.06] bg-black/20 px-2.5 py-2">
            <div className="flex items-center justify-between text-[10px]">
              <span className="text-white/45">Base (corpse)</span>
              <GlimAmount amount={corpseEnergy} compact className="text-[10px] text-white/55" />
            </div>
            <label className="flex items-center justify-between gap-2 text-[10px]">
              <span className="text-white/45">Extra glims</span>
              <input
                type="number"
                min={minExtra}
                max={maxExtra}
                step={GLIM_SCALE}
                value={extra}
                data-testid="e2e-deploy-extra"
                onChange={(e) => setExtra(Number.parseInt(e.target.value, 10) || minExtra)}
                className="w-24 rounded-md border border-white/[0.08] bg-black/30 px-2 py-1 text-right font-mono text-[11px] text-white/75 outline-none focus:border-biolume/25"
              />
            </label>
            <div className="flex items-center justify-between border-t border-white/[0.06] pt-2 text-[11px]">
              <span className="text-white/55">Total cost</span>
              <GlimAmount amount={totalCost} className="text-[11px] text-biolume/90" />
            </div>
            {!canAfford && (
              <p className="text-[10px] text-red-400/80">Not enough glims (have {formatGlimString(credits ?? 0)})</p>
            )}
            {extra < minExtra && (
              <p className="text-[10px] text-red-400/80">Minimum extra is {formatGlimString(minExtra)}</p>
            )}
          </div>

          <div className="mt-4 flex justify-end gap-2">
            <button
              type="button"
              className="deploy-btn"
              onClick={onClose}
              disabled={submitting || busy}
              data-testid="e2e-deploy-cancel"
            >
              Cancel
            </button>
            <button
              type="button"
              className="deploy-btn deploy-btn-primary flex items-center gap-1"
              disabled={!valid || submitting || busy}
              onClick={() => void submit()}
              data-testid="e2e-deploy-confirm"
            >
              {!submitting && !busy && <DeployIcon />}
              <span>{submitting || busy ? "Deploying…" : "Deploy"}</span>
            </button>
          </div>
        </div>
    </div>
  );
}

function DeployIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden className="shrink-0">
      <path
        d="M12 3c-1.5 3.5-3 5.5-3 9a3 3 0 006 0c0-3.5-1.5-5.5-3-9z"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinejoin="round"
      />
      <path d="M8 17l-2 4 4-1M16 17l2 4-4-1" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
      <circle cx="12" cy="10" r="1.25" fill="currentColor" />
    </svg>
  );
}

function LinkIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M10 13a5 5 0 007.07 0l2.83-2.83a5 5 0 00-7.07-7.07L11 4" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
      <path d="M14 11a5 5 0 00-7.07 0L4.1 13.83a5 5 0 007.07 7.07L13 20" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
    </svg>
  );
}

function UploadIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M12 16V4M12 4l4 4M12 4L8 8" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M8 5v14l11-7L8 5z" fill="currentColor" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M7 5h3v14H7V5zm7 0h3v14h-3V5z" fill="currentColor" />
    </svg>
  );
}

function StopIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M6 6l12 12M18 6L6 18" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
    </svg>
  );
}
