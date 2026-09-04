import { useCallback, useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { createPortal } from "react-dom";
import { formatWasmLimit, MAX_WASM_BYTES } from "../lib/deployLimits";
import {
  DEFAULT_RUST_SOURCE,
  DEFAULT_TESTS_SOURCE,
  parseTests,
  normalizeCompileDiagnostics,
  lintSourceActions,
  type CompileDiagnostic,
  type ParsedTest,
  type SandboxResult,
} from "../lib/creatureEditor";
import { formatGlimString, GLIM_SCALE } from "../lib/glim";
import { postCompile, postSandboxRun } from "../lib/api";
import { authorLinks } from "../lib/authoringLinks";
import { e2eHooksEnabled } from "../lib/config";
import { GlimAmount } from "./GlimAmount";
import { RustEditor, type RustEditorHandle } from "./RustEditor";
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

type TestRunResult = { passed: boolean; message?: string };

function testFailureMessage(result: SandboxResult): string {
  const failed = result.assertions.find((a) => !a.passed);
  if (failed) return failed.message;
  if (!result.alive && result.death_reason) {
    return `died at tick ${result.ticks_run} (${result.death_reason.replace(/_/g, " ")})`;
  }
  return "assertion failed";
}

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
  const [testsSource, setTestsSource] = useState(DEFAULT_TESTS_SOURCE);
  const [editorTab, setEditorTab] = useState<"source" | "tests">("source");
  const [selectedTestIndex, setSelectedTestIndex] = useState(0);
  const [activeTestName, setActiveTestName] = useState<string | null>(null);
  const [wasmB64, setWasmB64] = useState<string | undefined>();
  const [wasmLabel, setWasmLabel] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [diagnostics, setDiagnostics] = useState<CompileDiagnostic[]>([]);
  const [testResults, setTestResults] = useState<Record<string, TestRunResult>>({});
  const [previewGen, setPreviewGen] = useState(0);
  const sourceEditorRef = useRef<RustEditorHandle | null>(null);
  const testsEditorRef = useRef<RustEditorHandle | null>(null);
  const sourceRef = useRef(source);
  const testsSourceRef = useRef(testsSource);
  sourceRef.current = source;
  testsSourceRef.current = testsSource;
  const [testing, setTesting] = useState(false);
  const [deployOpen, setDeployOpen] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
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

  const parsedTests = useMemo(() => parseTests(testsSource), [testsSource]);
  const tests = parsedTests.tests;
  const selectedTest = tests[selectedTestIndex] ?? tests[0];
  const sourceShownDiagnostics = useMemo(
    () => diagnostics.filter((d) => (d.area ?? "source") === "source"),
    [diagnostics],
  );
  const testsShownDiagnostics = useMemo(
    () => [
      ...diagnostics.filter((d) => d.area === "tests"),
      ...parsedTests.diagnostics,
    ],
    [diagnostics, parsedTests.diagnostics],
  );
  const compileBlocked = diagnostics.some((d) => d.level === "error") || parsedTests.diagnostics.some((d) => d.level === "error");
  const allTestsPassed = useMemo(
    () => tests.length > 0 && !compileBlocked && tests.every((t) => testResults[t.name]?.passed === true),
    [tests, testResults, compileBlocked],
  );
  const selectedTestResult = selectedTest ? testResults[selectedTest.name] : undefined;
  const deployBlockedReason = !wasmB64
    ? null
    : compileBlocked
      ? "Fix compile errors first"
      : tests.length === 0
        ? "Add at least one test"
        : !allTestsPassed
          ? "All tests must pass"
          : null;

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
      allTestsPassed,
      playback: mapPlayback(playback),
      error,
    });
  }, [testing, wasmB64, allTestsPassed, playback, error, onE2eSliceChange]);

  const locked = busy || testing;

  const readEditorSources = useCallback((): { source: string; testsSource: string } => ({
    source: sourceEditorRef.current?.getValue() ?? sourceRef.current,
    testsSource: testsEditorRef.current?.getValue() ?? testsSourceRef.current,
  }), []);

  useEffect(() => {
    if (!open || !canCheck || busy || testing || e2eHooksEnabled()) return;
    const t = window.setTimeout(() => {
      const { source: liveSource, testsSource: liveTests } = readEditorSources();
      void postCompile("rust", liveSource, liveTests)
        .then((compiled) =>
          setDiagnostics([
            ...normalizeCompileDiagnostics(compiled.diagnostics),
            ...lintSourceActions(),
          ]),
        )
        .catch(() => {
          /* offline / auth */
        });
    }, 900);
    return () => window.clearTimeout(t);
  }, [source, testsSource, open, canCheck, busy, testing, readEditorSources]);

  useEffect(() => {
    if (selectedTestIndex >= tests.length) {
      setSelectedTestIndex(Math.max(0, tests.length - 1));
    }
  }, [tests.length, selectedTestIndex]);

  const applyWasm = async (file: File) => {
    try {
      const { b64, name } = await readWasmFile(file);
      setWasmB64(b64);
      setWasmLabel(name);
      loadResult(null);
      setTestResults({});
      setError(null);
    } catch (err) {
      setWasmB64(undefined);
      setWasmLabel(null);
      setError(err instanceof Error ? err.message : "Invalid WASM file");
    }
  };

  const invalidateWasm = () => {
    setWasmB64(undefined);
    setWasmLabel(null);
    loadResult(null);
    setActiveTestName(null);
    setTestResults({});
    if (error) setError(null);
  };

  const compileSources = useCallback(async (liveSource: string, liveTests: string): Promise<string | null> => {
    sourceRef.current = liveSource;
    testsSourceRef.current = liveTests;
    if (liveSource !== source) setSource(liveSource);
    if (liveTests !== testsSource) setTestsSource(liveTests);

    const compiled = await postCompile("rust", liveSource, liveTests);
    setDiagnostics([
      ...normalizeCompileDiagnostics(compiled.diagnostics),
      ...lintSourceActions(),
    ]);
    if (!compiled.wasm_b64) {
      const msg = compiled.diagnostics.find((d) => d.level === "error")?.message ?? "Compile failed";
      setError(msg);
      return null;
    }
    if (!compiled.ok) {
      const msg = compiled.diagnostics.find((d) => d.level === "error")?.message ?? "Fix compile errors first";
      setError(msg);
      if (compiled.diagnostics.some((d) => d.area === "tests")) setEditorTab("tests");
      return null;
    }
    setWasmB64(compiled.wasm_b64);
    setWasmLabel("creature.wasm");
    setError(null);
    return compiled.wasm_b64;
  }, [source, testsSource]);

  const ensureWasm = useCallback(async (): Promise<string | null> => {
    if (wasmB64) return wasmB64;
    const snap = readEditorSources();
    return compileSources(snap.source, snap.testsSource);
  }, [compileSources, readEditorSources, wasmB64]);

  const recordTestResult = useCallback((test: ParsedTest, result: SandboxResult) => {
    const passed = result.test_passed;
    const message = passed ? undefined : testFailureMessage(result);
    setTestResults((prev) => ({ ...prev, [test.name]: { passed, message } }));
    return passed;
  }, []);

  const runSandboxTest = useCallback(
    async (test: ParsedTest, b64: string, preview: boolean) => {
      const result = await postSandboxRun(b64, test.spec);
      recordTestResult(test, result);
      if (preview) {
        setActiveTestName(test.name);
        loadResult(result);
        setPreviewGen((g) => g + 1);
        if (result.frames.length > 0) {
          if (!e2eHooksEnabled()) window.requestAnimationFrame(() => play());
        } else if (result.error) setError(result.error);
      }
      return result;
    },
    [loadResult, play, recordTestResult],
  );

  const runSelectedTest = async () => {
    const snap = readEditorSources();
    const liveTests = parseTests(snap.testsSource).tests;
    const test = liveTests[selectedTestIndex] ?? liveTests[0];
    if (!test) {
      setError("Add at least one #[terrarium::test] in the Tests tab");
      setEditorTab("tests");
      return;
    }
    setTesting(true);
    setError(null);
    stop();
    try {
      const b64 = await compileSources(snap.source, snap.testsSource);
      if (!b64) {
        return;
      }
      await runSandboxTest(test, b64, true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Test failed");
    } finally {
      setTesting(false);
    }
  };

  const runAllTests = async () => {
    const snap = readEditorSources();
    const liveTests = parseTests(snap.testsSource).tests;
    if (liveTests.length === 0) {
      setError("Add at least one #[terrarium::test] in the Tests tab");
      setEditorTab("tests");
      return;
    }
    setTesting(true);
    setError(null);
    stop();
    try {
      const b64 = await compileSources(snap.source, snap.testsSource);
      if (!b64) {
        return;
      }
      let preview: { test: ParsedTest; result: SandboxResult } | null = null;
      let firstFail: { test: ParsedTest; result: SandboxResult } | null = null;
      for (const test of liveTests) {
        const result = await postSandboxRun(b64, test.spec);
        recordTestResult(test, result);
        if (!result.test_passed && !firstFail) firstFail = { test, result };
        preview = { test, result };
      }
      const show = firstFail ?? preview!;
      setSelectedTestIndex(liveTests.indexOf(show.test));
      setActiveTestName(show.test.name);
      loadResult(show.result);
      setPreviewGen((g) => g + 1);
      if (show.result.frames.length > 0) {
        window.requestAnimationFrame(() => play());
      } else if (show.result.error) setError(show.result.error);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Test failed");
    } finally {
      setTesting(false);
    }
  };

  const canDeploy = !!wasmB64 && allTestsPassed;
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
            title={deployBlockedReason ?? "Deploy creature"}
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
                setError(deployBlockedReason ?? "All tests must pass before deploy");
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
            <div className="studio-tabs studio-tabs-corner">
              <button
                type="button"
                className={`studio-tab${editorTab === "source" ? " studio-tab-active" : ""}`}
                onClick={() => setEditorTab("source")}
              >
                Source
              </button>
              <button
                type="button"
                className={`studio-tab${editorTab === "tests" ? " studio-tab-active" : ""}`}
                onClick={() => setEditorTab("tests")}
              >
                Tests
              </button>
            </div>
            <div className={editorTab === "source" ? "studio-editor-panel" : "studio-editor-panel studio-editor-panel-hidden"}>
              <RustEditor
                ref={sourceEditorRef}
                value={source}
                height="100%"
                onChange={(v) => {
                  setSource(v);
                  sourceRef.current = v;
                  invalidateWasm();
                }}
                diagnostics={sourceShownDiagnostics}
                readOnly={locked}
              />
            </div>
            <div className={editorTab === "tests" ? "studio-editor-panel" : "studio-editor-panel studio-editor-panel-hidden"}>
              <RustEditor
                ref={testsEditorRef}
                value={testsSource}
                height="100%"
                onChange={(v) => {
                  setTestsSource(v);
                  testsSourceRef.current = v;
                  invalidateWasm();
                }}
                diagnostics={testsShownDiagnostics}
                readOnly={locked}
              />
            </div>
          </div>

          <div className="studio-controls">
            <select
              className="studio-test-select"
              value={selectedTest?.name ?? ""}
              disabled={locked || tests.length === 0}
              onChange={(e) => {
                const idx = tests.findIndex((t) => t.name === e.target.value);
                if (idx >= 0) setSelectedTestIndex(idx);
              }}
              data-testid="e2e-studio-test-select"
            >
              {tests.length === 0 ? (
                <option value="">No tests</option>
              ) : (
                tests.map((t) => {
                  const run = testResults[t.name];
                  const mark = run?.passed ? "✓ " : run?.passed === false ? "✗ " : "";
                  return (
                    <option key={t.name} value={t.name}>
                      {mark}
                      {t.label}
                    </option>
                  );
                })
              )}
            </select>
            <button
              type="button"
              className="deploy-btn"
              disabled={locked || !selectedTest}
              onClick={() => void runSelectedTest()}
              title="Run selected test in preview"
              data-testid="e2e-studio-test"
            >
              {testing ? "Running…" : "Run test"}
            </button>
            <button
              type="button"
              className="deploy-btn"
              disabled={locked || tests.length === 0}
              onClick={() => void runAllTests()}
              title="Run all tests (required before deploy)"
              data-testid="e2e-studio-run-all"
            >
              Run all
            </button>
            {selectedTestResult && (
              <span
                className={
                  selectedTestResult.passed ? "studio-test-status studio-test-status-pass" : "studio-test-status studio-test-status-fail"
                }
                data-testid="e2e-studio-test-status"
              >
                {selectedTestResult.passed
                  ? "Passed"
                  : selectedTestResult.message ?? "Failed"}
              </span>
            )}
            {error && (
              <span className="studio-test-err" title={error}>
                {error}
              </span>
            )}
          </div>
        </div>

        <div
          className="studio-split-handle"
          onPointerDown={startSplitResize}
          title="Drag to resize"
          aria-hidden
        />

        <div className="studio-preview-pane">
          {activeTestName && selectedTest && (
            <div className="absolute left-2 top-2 z-10 flex max-w-[calc(100%-1rem)] flex-wrap items-center gap-2">
              <div className="rounded-md border border-white/[0.08] bg-black/60 px-2 py-1 text-[10px] text-white/70 backdrop-blur-sm">
                Test: <span className="text-biolume/90">{selectedTest.label}</span>
              </div>
              {selectedTestResult && (
                <div
                  className={
                    selectedTestResult.passed
                      ? "studio-preview-badge studio-preview-badge-pass"
                      : "studio-preview-badge studio-preview-badge-fail"
                  }
                >
                  {selectedTestResult.passed ? "Passed" : selectedTestResult.message ?? "Failed"}
                </div>
              )}
            </div>
          )}
          {!sandbox || !sandbox.frames.length ? (
            <div className="flex h-full items-center justify-center pb-16 px-4 text-center text-[11px] text-white/30">
              {sandbox?.error ?? (testing ? "Running test…" : "Pick a test and run it to preview")}
            </div>
          ) : (
            <WorldCanvas
              key={previewGen}
              creaturesLiveRef={creaturesLiveRef}
              tilesLiveRef={tilesLiveRef}
              canDeploy={false}
              userUid="sandbox"
              corpseEnergy={corpseEnergy}
              senseRange={senseRange}
              view="follow"
              followId="sandbox"
              initialZoom={1.2}
              onCellSelect={() => {}}
              runtimeRef={runtimeRef}
              worldTick={worldTick}
              tickHz={tickHz}
            />
          )}

          <div className="studio-preview-controls">
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
                onClick={stop}
              >
                <StopIcon />
              </button>
            </div>
            {sandbox && (
              <>
                <input
                  type="range"
                  min={0}
                  max={Math.max(0, sandbox.frames.length - 1)}
                  value={frameIndex}
                  disabled={playback === "playing"}
                  onChange={(e) => seek(Number(e.target.value))}
                  className="min-w-0 flex-1 accent-biolume"
                />
                <div className="hidden shrink-0 grid-cols-2 gap-x-3 font-mono text-[9px] text-white/45 sm:grid">
                  <span>Tick {sandbox.frames[frameIndex]?.tick ?? 0}</span>
                  <GlimAmount
                    label="Energy"
                    amount={sandbox.frames[frameIndex]?.energy ?? 0}
                    compact
                    className="justify-end text-[9px]"
                  />
                </div>
              </>
            )}
          </div>
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
