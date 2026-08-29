import { motion } from "framer-motion";

type Props = {
  status: string;
  credits: number | null;
  signedIn: boolean;
  onSignIn: () => void;
  onSignOut: () => void;
  onFaucet: () => void;
  busy: boolean;
};

export function TopBar({
  status,
  credits,
  signedIn,
  onSignIn,
  onSignOut,
  onFaucet,
  busy,
}: Props) {
  return (
    <header className="glass-panel sticky top-0 z-10 flex items-center gap-4 border-b border-line px-4 py-3">
      <div className="flex items-center gap-2">
        <span className="inline-block h-2 w-2 rounded-full bg-biolume shadow-[0_0_10px_rgba(74,232,194,0.8)]" />
        <span className="font-display text-xl tracking-wide text-white">Terrarium</span>
      </div>

      <motion.p
        key={status}
        className="flex-1 truncate font-mono text-xs text-fog"
        initial={{ opacity: 0.5 }}
        animate={{ opacity: 1 }}
      >
        {status}
      </motion.p>

      <div className="flex items-center gap-2">
        {signedIn && credits !== null && (
          <span className="rounded-full border border-amber/20 bg-amber/10 px-2.5 py-1 font-mono text-xs text-amber">
            {credits.toLocaleString()} cr
          </span>
        )}
        {signedIn && (
          <button type="button" className="btn btn-primary" onClick={onFaucet} disabled={busy}>
            +100
          </button>
        )}
        {signedIn ? (
          <button type="button" className="btn btn-ghost" onClick={onSignOut} disabled={busy}>
            Sign out
          </button>
        ) : (
          <button type="button" className="btn btn-primary" onClick={onSignIn} disabled={busy}>
            Sign in
          </button>
        )}
      </div>
    </header>
  );
}
