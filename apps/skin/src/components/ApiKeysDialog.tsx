import { useCallback, useEffect, useState } from "react";
import { deleteApiKey, getApiKeys, mintApiKey, type ApiKey, type MintApiKeyResponse } from "../lib/api";

type Props = {
  open: boolean;
  busy: boolean;
  onClose: () => void;
};

export function ApiKeysDialog({ open, busy, onClose }: Props) {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(false);
  const [name, setName] = useState("");
  const [fresh, setFresh] = useState<MintApiKeyResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setKeys((await getApiKeys()).keys);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load keys");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    setFresh(null);
    setName("");
    setError(null);
    void refresh();
  }, [open, refresh]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  const create = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await mintApiKey(name.trim() || undefined);
      setFresh(res);
      setName("");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Mint failed");
    } finally {
      setLoading(false);
    }
  };

  const revoke = async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      await deleteApiKey(id);
      if (fresh?.id === id) setFresh(null);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Revoke failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="pointer-events-auto absolute inset-0 z-30 flex items-end justify-center p-3 sm:items-center sm:p-4">
      <button type="button" className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" aria-label="Close" onClick={onClose} />
      <div className="deploy-panel relative w-full max-w-md">
        <div className="mb-2 flex items-baseline justify-between gap-3">
          <h2 className="text-sm font-medium text-white/85">API keys</h2>
          <a href="/api/docs" target="_blank" rel="noreferrer" className="font-mono text-[10px] text-biolume/60">
            Docs ↗
          </a>
        </div>
        <p className="mb-2 text-[10px] text-white/35">
          Bearer <span className="font-mono text-white/50">tr_…</span> for deploy and /me.
        </p>
        {fresh && (
          <div className="mb-2 rounded-lg border border-amber/20 bg-amber/5 px-2.5 py-2">
            <p className="text-[10px] text-amber/80">Copy now — shown once:</p>
            <code className="mt-1 block break-all font-mono text-[10px] text-white/75">{fresh.secret}</code>
          </div>
        )}
        <div className="mb-2 flex gap-1.5">
          <input
            type="text"
            value={name}
            placeholder="Label"
            disabled={busy || loading}
            onChange={(e) => setName(e.target.value)}
            className="min-w-0 flex-1 rounded-md border border-white/[0.08] bg-black/30 px-2 py-1.5 font-mono text-[11px] text-white/75"
          />
          <button type="button" className="deploy-btn deploy-btn-primary" disabled={busy || loading} onClick={() => void create()}>
            Mint
          </button>
        </div>
        <ul className="max-h-40 space-y-1 overflow-y-auto">
          {keys.map((k) => (
            <li key={k.id} className="flex items-center justify-between gap-2 rounded-md border border-white/[0.06] bg-black/20 px-2 py-1.5">
              <span className="truncate font-mono text-[10px] text-white/70">
                {k.name ? `${k.name} · ` : ""}
                {k.prefix}…
              </span>
              <button type="button" className="deploy-btn text-red-400/70" disabled={busy || loading} onClick={() => void revoke(k.id)}>
                Revoke
              </button>
            </li>
          ))}
        </ul>
        {error && <p className="mt-1.5 font-mono text-[10px] text-red-400/80">{error}</p>}
        <div className="mt-3 flex justify-end">
          <button type="button" className="deploy-btn" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
}
