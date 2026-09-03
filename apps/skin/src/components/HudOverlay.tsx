import { useState } from "react";
import type { Creature } from "../lib/api";
import { GlimAmount } from "./GlimAmount";
import { formatGlimString } from "../lib/glim";

type Props = {
  online: boolean;
  credits: number | null;
  signedIn: boolean;
  busy: boolean;
  view: "god" | "follow";
  followId: string | null;
  myCreatures: Creature[];
  cell: { x: number; y: number; label: string } | null;
  message: string | null;
  deathNotice?: string | null;
  error?: string | null;
  onViewChange: (view: "god" | "follow") => void;
  onJumpOpen: () => void;
  onFollowCreature: (id: string) => void;
  onSignIn: () => void;
  onSignOut: () => void;
  onFaucet: () => void;
  onApiKeysOpen: () => void;
  onCodeOpen: () => void;
};

export function HudOverlay({
  online,
  credits,
  signedIn,
  busy,
  view,
  followId,
  myCreatures,
  cell,
  message,
  deathNotice,
  error,
  onViewChange,
  onJumpOpen,
  onFollowCreature,
  onSignIn,
  onSignOut,
  onFaucet,
  onApiKeysOpen,
  onCodeOpen,
}: Props) {
  const followLabel = followId ? `${followId.slice(0, 8)}…` : null;
  const statusText = error
    ? { kind: "error" as const, text: error }
    : cell
      ? { kind: "cell" as const, cell }
      : deathNotice
        ? { kind: "death" as const, text: deathNotice }
    : view === "follow" && followLabel
        ? { kind: "text" as const, text: `Following ${followLabel}` }
        : message
          ? { kind: "text" as const, text: message }
          : null;

  return (
    <div className="pointer-events-none absolute inset-0 z-10">
      <div className="hud-panel pointer-events-auto absolute left-3 top-3 p-1.5 sm:left-4 sm:top-4">
        <div className="flex items-center gap-1.5">
          <span className={`hud-dot shrink-0 ${online ? "hud-dot-live" : ""}`} />
          <span className="shrink-0 text-[12px] font-medium text-white/75">Terrarium</span>
          {signedIn && (
            <button
              type="button"
              className="hud-btn-sm flex shrink-0 items-center gap-1 px-1.5 text-white/55 hover:text-white/85"
              onClick={onCodeOpen}
              disabled={busy}
              title="Creature studio"
              data-testid="e2e-hud-studio"
            >
              <StudioIcon />
              <span className="hidden text-[10px] sm:inline">Studio</span>
            </button>
          )}
          <div className="hud-segment min-w-0 flex-1">
            <button
              type="button"
              className={`hud-segment-btn flex-1 ${view === "god" ? "hud-segment-btn-active" : ""}`}
              onClick={() => onViewChange("god")}
            >
              Map
            </button>
            <button
              type="button"
              className={`hud-segment-btn flex-1 ${view === "follow" ? "hud-segment-btn-active" : ""}`}
              onClick={() => onViewChange("follow")}
            >
              Follow
            </button>
          </div>
          <button type="button" className="hud-btn-sm shrink-0 px-1.5" onClick={onJumpOpen} data-testid="e2e-hud-jump">
            ⌕
          </button>
        </div>

        {signedIn && myCreatures.length > 0 && (
          <ul className="mt-1.5 max-h-28 space-y-px overflow-y-auto border-t border-white/[0.05] pt-1.5">
            {myCreatures.map((c) => {
              const active = view === "follow" && followId === c.id;
              return (
                <li key={c.id}>
                  <button
                    type="button"
                    className={`creature-row w-full ${active ? "creature-row-active" : ""}`}
                    onClick={() => onFollowCreature(c.id)}
                  >
                    <span className="truncate font-mono text-[10px] text-biolume/75">
                      {c.id.slice(0, 6)}
                    </span>
                    <span className="shrink-0 font-mono text-[9px] text-white/30">
                      {c.health}/{c.max_health} ·{" "}
                      <GlimAmount amount={c.energy} className="text-[9px] text-white/45" compact />
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}

      </div>

      <div className="pointer-events-auto absolute right-3 top-3 sm:right-4 sm:top-4">
        <div className="hud-chip flex items-center gap-0.5">
          {signedIn && credits !== null && (
            <GlimAmount
              amount={credits}
              className="px-1.5 text-[10px] text-amber/70"
              iconClassName="h-2.5 w-2.5 shrink-0 text-amber/75"
            />
          )}
          {signedIn ? (
            <>
              <button type="button" className="hud-action" onClick={onApiKeysOpen} disabled={busy}>
                Keys
              </button>
              <button
                type="button"
                className="hud-action"
                onClick={onFaucet}
                disabled={busy}
                data-testid="e2e-hud-faucet"
              >
                +{formatGlimString(10_000_000)}
              </button>
              <button
                type="button"
                className="hud-action"
                onClick={onSignOut}
                disabled={busy}
                data-testid="e2e-hud-sign-out"
              >
                Out
              </button>
            </>
          ) : (
            <button
              type="button"
              className="hud-action hud-action-accent"
              onClick={onSignIn}
              disabled={busy}
              data-testid="e2e-hud-sign-in"
            >
              Sign in
            </button>
          )}
        </div>
      </div>

      {statusText && (
        <div className="status-bar">
          {statusText.kind === "error" ? (
            <span className="truncate text-red-400/80">{statusText.text}</span>
          ) : statusText.kind === "death" ? (
            <span className="truncate text-amber-300/75">{statusText.text}</span>
          ) : statusText.kind === "cell" ? (
            <>
              <span className="shrink-0 tabular-nums text-white/35">
                {statusText.cell.x}, {statusText.cell.y}
              </span>
              <CopyCoordsButton x={statusText.cell.x} y={statusText.cell.y} />
              <span className="text-white/15">·</span>
              <span className="truncate text-white/55">{statusText.cell.label}</span>
            </>
          ) : (
            <span className="truncate text-white/40">{statusText.text}</span>
          )}
        </div>
      )}
    </div>
  );
}

function CopyCoordsButton({ x, y }: { x: number; y: number }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    const text = `${x}, ${y}`;
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      /* clipboard blocked */
    }
  };

  return (
    <button
      type="button"
      className="pointer-events-auto -my-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-white/30 transition-colors hover:bg-white/[0.06] hover:text-white/60"
      onClick={() => void copy()}
      title={copied ? "Copied" : "Copy coordinates"}
      aria-label={copied ? "Copied coordinates" : "Copy coordinates"}
    >
      {copied ? <CheckIcon /> : <CopyIcon />}
    </button>
  );
}

function CopyIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden>
      <rect x="9" y="9" width="11" height="11" rx="1.5" stroke="currentColor" strokeWidth="1.75" />
      <path d="M6 15H5a2 2 0 01-2-2V5a2 2 0 012-2h8a2 2 0 012 2v1" stroke="currentColor" strokeWidth="1.75" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M5 12l5 5L19 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function StudioIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden className="shrink-0">
      <path
        d="M8 6l-4 6 4 6M16 6l4 6-4 6M14 4l-4 16"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
