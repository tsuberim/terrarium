import { useEffect, useRef, useState } from "react";
import { EXAMPLE_PROGRAMS } from "../lib/examples";

type Props = {
  cell: { x: number; y: number } | null;
  deployCost: number;
  busy: boolean;
  onDeploy: (code: string) => void;
  onClose: () => void;
};

export function DeployDialog({ cell, deployCost, busy, onDeploy, onClose }: Props) {
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!cell) return;
    setCode("");
    setError(null);
    requestAnimationFrame(() => textareaRef.current?.focus());
  }, [cell]);

  useEffect(() => {
    if (!cell) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cell, onClose]);

  if (!cell) return null;

  const submit = () => {
    if (!code.trim()) {
      setError("Enter a program");
      return;
    }
    setError(null);
    onDeploy(code);
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
            ({cell.x}, {cell.y}) · {deployCost} cr
          </span>
        </div>

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
            Deploy
          </button>
        </div>
      </div>
    </div>
  );
}
