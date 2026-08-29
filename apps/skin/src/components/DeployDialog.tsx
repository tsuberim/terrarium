import { useEffect, useRef, useState } from "react";
import { EXAMPLE_PROGRAMS } from "../lib/examples";

type Props = {
  cell: { x: number; y: number } | null;
  minExtra: number;
  corpseEnergy: number;
  credits: number | null;
  busy: boolean;
  onDeploy: (code: string, extraEnergy: number) => void;
  onClose: () => void;
};

export function DeployDialog({
  cell,
  minExtra,
  corpseEnergy,
  credits,
  busy,
  onDeploy,
  onClose,
}: Props) {
  const [code, setCode] = useState("");
  const [extra, setExtra] = useState(minExtra);
  const [error, setError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!cell) return;
    setCode("");
    setExtra(minExtra);
    setError(null);
    requestAnimationFrame(() => textareaRef.current?.focus());
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

  const maxExtra = credits ?? minExtra;
  const totalEnergy = corpseEnergy + extra;

  const submit = () => {
    if (!code.trim()) {
      setError("Enter a program");
      return;
    }
    if (extra < minExtra) {
      setError(`Minimum extra energy is ${minExtra}`);
      return;
    }
    if (credits !== null && extra > credits) {
      setError("Not enough credits");
      return;
    }
    setError(null);
    onDeploy(code, extra);
  };

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

        <div className="mb-2 flex items-center justify-between gap-3 rounded-lg border border-white/[0.06] bg-black/20 px-2.5 py-2">
          <label className="text-[11px] text-white/45" htmlFor="deploy-extra">
            Extra energy
          </label>
          <div className="flex items-center gap-2">
            <input
              id="deploy-extra"
              type="number"
              min={minExtra}
              max={maxExtra}
              step={1}
              value={extra}
              disabled={busy}
              onChange={(e) => {
                setExtra(Number.parseInt(e.target.value, 10) || minExtra);
                if (error) setError(null);
              }}
              className="w-20 rounded-md border border-white/[0.08] bg-black/30 px-2 py-1 text-right font-mono text-[11px] text-white/75 outline-none focus:border-biolume/25"
            />
            <span className="font-mono text-[10px] text-white/30">
              = {totalEnergy} total
            </span>
          </div>
        </div>

        <p className="mb-2 text-[10px] text-white/28">
          Costs {extra} credits · {corpseEnergy} base + {extra} spendable
        </p>

        <div className="mb-2 flex flex-wrap gap-1">
          {EXAMPLE_PROGRAMS.map((example) => (
            <button
              key={example.id}
              type="button"
              className="deploy-btn"
              title={example.description}
              disabled={busy}
              onClick={() => {
                setCode(example.code);
                if (error) setError(null);
                textareaRef.current?.focus();
              }}
            >
              {example.name}
            </button>
          ))}
        </div>

        <textarea
          ref={textareaRef}
          value={code}
          onChange={(e) => {
            setCode(e.target.value);
            if (error) setError(null);
          }}
          placeholder={"loop:\n  sleep\n  jmp loop"}
          spellCheck={false}
          rows={10}
          className="deploy-input"
        />

        {error && <p className="mt-1.5 font-mono text-[10px] text-red-400/80">{error}</p>}

        <p className="mt-2 font-mono text-[9px] leading-relaxed text-white/25">
          Code is immutable after deploy. Only you can read it.
        </p>

        <div className="mt-3 flex justify-end gap-1.5">
          <button type="button" className="deploy-btn" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button type="button" className="deploy-btn deploy-btn-primary" onClick={submit} disabled={busy}>
            Deploy · {extra} cr
          </button>
        </div>
      </div>
    </div>
  );
}
