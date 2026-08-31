import { useState } from "react";
import { apiRoot } from "../lib/config";
import { authorLinks, openReplitWithEnv, resolveApiBase } from "../lib/authoringLinks";

type Props = {
  open: boolean;
  cell: { x: number; y: number } | null;
  onClose: () => void;
  onOpenKeys: () => void;
};

export function AuthorDialog({ open, cell, onClose, onOpenKeys }: Props) {
  const [copied, setCopied] = useState(false);

  if (!open) return null;

  const target = cell ?? { x: 0, y: 0 };
  const apiBase = resolveApiBase(apiRoot());

  const launch = async () => {
    await openReplitWithEnv({ apiBase, x: target.x, y: target.y });
    setCopied(true);
    window.setTimeout(() => setCopied(false), 4000);
  };

  return (
    <div className="pointer-events-auto absolute inset-0 z-30 flex items-end justify-center p-3 sm:items-center sm:p-4">
      <button
        type="button"
        className="absolute inset-0 bg-black/50 backdrop-blur-[3px]"
        aria-label="Close"
        onClick={onClose}
      />

      <div className="deploy-panel relative w-full max-w-md">
        <h2 className="mb-1 text-sm font-medium text-white/85">Code in Replit</h2>
        <p className="mb-3 text-[11px] leading-relaxed text-white/40">
          Browser IDE with Zig and AI (Replit Agent, etc.). No install. Press{" "}
          <span className="font-mono text-white/55">Run</span> to build and deploy to{" "}
          <span className="font-mono text-white/55">
            ({target.x}, {target.y})
          </span>
          .
        </p>

        <div className="flex flex-col gap-2">
          <button type="button" className="deploy-btn deploy-btn-primary" onClick={() => void launch()}>
            Open in Replit
          </button>
          <button
            type="button"
            className="deploy-btn"
            onClick={() => {
              onClose();
              onOpenKeys();
            }}
          >
            Get API key
          </button>
          <a
            href={authorLinks.docs}
            target="_blank"
            rel="noreferrer"
            className="deploy-btn text-center text-white/50"
          >
            Docs
          </a>
        </div>

        <ol className="mt-3 list-decimal space-y-1 pl-4 text-[10px] leading-relaxed text-white/32">
          <li>Open Replit — .env copied to clipboard</li>
          <li>Paste into sdk/zig/.env, add API key from Keys</li>
          <li>Edit main.zig → Run</li>
        </ol>

        {copied && (
          <p className="mt-2 font-mono text-[9px] text-biolume/70">.env copied — paste in Replit at sdk/zig/.env</p>
        )}

        {!cell && (
          <p className="mt-2 text-[10px] text-white/28">Tip: click an empty cell first to set deploy coordinates.</p>
        )}

        <div className="mt-3 flex justify-end">
          <button type="button" className="deploy-btn" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
