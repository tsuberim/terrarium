import type { Creature } from "../lib/api";

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
  error?: string | null;
  onViewChange: (view: "god" | "follow") => void;
  onJumpOpen: () => void;
  onFollowCreature: (id: string) => void;
  onSignIn: () => void;
  onSignOut: () => void;
  onFaucet: () => void;
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
  error,
  onViewChange,
  onJumpOpen,
  onFollowCreature,
  onSignIn,
  onSignOut,
  onFaucet,
}: Props) {
  const followLabel = followId ? `${followId.slice(0, 8)}…` : null;
  const statusText = error
    ? { kind: "error" as const, text: error }
    : cell
      ? { kind: "cell" as const, cell }
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
          <button type="button" className="hud-btn-sm shrink-0 px-1.5" onClick={onJumpOpen}>
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
                    <span className="shrink-0 font-mono text-[9px] text-white/30">{c.energy}</span>
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
            <span className="px-1.5 font-mono text-[10px] tabular-nums text-amber/70">
              {credits.toLocaleString()}
            </span>
          )}
          {signedIn ? (
            <>
              <button type="button" className="hud-action" onClick={onFaucet} disabled={busy}>
                +100
              </button>
              <button type="button" className="hud-action" onClick={onSignOut} disabled={busy}>
                Out
              </button>
            </>
          ) : (
            <button type="button" className="hud-action hud-action-accent" onClick={onSignIn} disabled={busy}>
              Sign in
            </button>
          )}
        </div>
      </div>

      {statusText && (
        <div className="status-bar">
          {statusText.kind === "error" ? (
            <span className="truncate text-red-400/80">{statusText.text}</span>
          ) : statusText.kind === "cell" ? (
            <>
              <span className="shrink-0 tabular-nums text-white/35">
                {statusText.cell.x}, {statusText.cell.y}
              </span>
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
