import { useEffect, useMemo, useRef, useState } from "react";
import type { Creature } from "../lib/api";
import { creatureMatches, parseJumpQuery, resolveCreatureId } from "../lib/navigation";

type Props = {
  open: boolean;
  creatures: Creature[];
  userUid?: string;
  onClose: () => void;
  onJump: (x: number, y: number) => void;
  onFollow: (id: string) => void;
};

export function JumpDialog({ open, creatures, userUid, onClose, onJump, onFollow }: Props) {
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setError(null);
    setActive(0);
    requestAnimationFrame(() => inputRef.current?.focus());
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  const suggestions = useMemo(() => creatureMatches(query, creatures), [query, creatures]);
  const parsed = useMemo(() => parseJumpQuery(query), [query]);

  const submit = () => {
    const q = query.trim();
    if (!q) return;

    const coords = parseJumpQuery(q);
    if (coords?.type === "coords") {
      onJump(coords.x, coords.y);
      return;
    }

    const id = resolveCreatureId(q, creatures);
    if (id) {
      onFollow(id);
      return;
    }

    if (suggestions[active]) {
      onFollow(suggestions[active].id);
      return;
    }

    setError("Try coordinates like 45, 19 or a creature id");
  };

  if (!open) return null;

  return (
    <div className="pointer-events-auto absolute inset-0 z-30 flex items-start justify-center p-3 pt-[12vh] sm:p-4 sm:pt-[14vh]">
      <button
        type="button"
        className="absolute inset-0 bg-black/40 backdrop-blur-[2px]"
        aria-label="Close search"
        onClick={onClose}
      />

      <div className="deploy-panel relative w-full max-w-md">
        <div className="mb-2">
          <h2 className="text-sm font-medium text-white/85">Find location or creature</h2>
        </div>

        <input
          ref={inputRef}
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setActive(0);
            if (error) setError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setActive((i) => Math.min(i + 1, Math.max(0, suggestions.length - 1)));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setActive((i) => Math.max(i - 1, 0));
            } else if (e.key === "Enter") {
              e.preventDefault();
              submit();
            }
          }}
          placeholder="45, 19 or creature id"
          spellCheck={false}
          className="deploy-input !min-h-0 py-2"
        />

        {error && <p className="mt-1.5 text-[11px] text-red-400/80">{error}</p>}

        <div className="mt-2 space-y-0.5">
          {parsed?.type === "coords" && (
            <button
              type="button"
              className="jump-row jump-row-active w-full text-left"
              onClick={() => onJump(parsed.x, parsed.y)}
            >
              <span className="text-[11px] text-white/70">Go to location</span>
              <span className="font-mono text-[10px] text-white/40">
                {parsed.x}, {parsed.y}
              </span>
            </button>
          )}

          {suggestions.map((c, i) => {
            const mine = c.owner_uid === userUid;
            return (
              <button
                key={c.id}
                type="button"
                className={`jump-row w-full text-left ${i === active ? "jump-row-active" : ""}`}
                onMouseEnter={() => setActive(i)}
                onClick={() => onFollow(c.id)}
              >
                <span className={`truncate text-[11px] ${mine ? "text-biolume/80" : "text-white/55"}`}>
                  {mine ? "Your creature" : "Creature"} · {c.id.slice(0, 8)}…
                </span>
                <span className="shrink-0 text-[10px] text-white/30">
                  {c.x}, {c.y}
                </span>
              </button>
            );
          })}

          {!parsed && !suggestions.length && query.trim() && (
            <p className="px-2 py-1 text-[11px] text-white/30">No matches</p>
          )}
        </div>

        <p className="mt-2 text-[10px] leading-snug text-white/30">
          Coordinates pan the map. A creature id switches to follow mode.
        </p>
      </div>
    </div>
  );
}
