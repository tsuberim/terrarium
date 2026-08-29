import { useCallback, useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { TopBar } from "./components/TopBar";
import { WorldCanvas } from "./components/WorldCanvas";
import { useAuth } from "./hooks/useAuth";
import { getHealth, getMe, postFaucet } from "./lib/api";
import {
  auth,
  googleProvider,
  signInWithPopup,
  signOut,
} from "./lib/firebase";

export default function App() {
  const { user, ready } = useAuth();
  const [view, setView] = useState<"god" | "follow">("god");
  const [status, setStatus] = useState("connecting…");
  const [credits, setCredits] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);

  const refreshHealth = useCallback(async () => {
    try {
      const health = await getHealth();
      setStatus(`world ${health.status} · ${health.tick_hz} Hz`);
    } catch {
      setStatus("world unreachable");
    }
  }, []);

  const refreshAccount = useCallback(async () => {
    if (!user) {
      setCredits(null);
      return;
    }
    try {
      const me = await getMe();
      setCredits(me.credits);
    } catch {
      setCredits(null);
    }
  }, [user]);

  useEffect(() => {
    void refreshHealth();
    const id = window.setInterval(refreshHealth, 30_000);
    return () => window.clearInterval(id);
  }, [refreshHealth]);

  useEffect(() => {
    if (!ready) return;
    void refreshAccount();
  }, [ready, refreshAccount]);

  const signIn = async () => {
    setBusy(true);
    try {
      await signInWithPopup(auth, googleProvider);
    } finally {
      setBusy(false);
    }
  };

  const signOutUser = async () => {
    setBusy(true);
    try {
      await signOut(auth);
    } finally {
      setBusy(false);
    }
  };

  const faucet = async () => {
    setBusy(true);
    try {
      const res = await postFaucet(100);
      setCredits(res.credits);
    } finally {
      setBusy(false);
    }
  };

  const accountLabel = !ready
    ? "Loading…"
    : user
      ? (user.email ?? user.uid)
      : "Sign in to deploy creatures and manage credits";

  return (
    <div className="min-h-screen">
      <TopBar
        status={status}
        credits={credits}
        signedIn={!!user}
        onSignIn={() => void signIn()}
        onSignOut={() => void signOutUser()}
        onFaucet={() => void faucet()}
        busy={busy}
      />
      <div className="grid md:grid-cols-[240px_1fr]">
        <Sidebar
          view={view}
          onViewChange={setView}
          accountLabel={accountLabel}
          worldNote="Kernel not wired yet. Grid preview only — the world runs continuously once live."
        />
        <main className="flex items-center justify-center p-4 md:p-8">
          <WorldCanvas />
        </main>
      </div>
    </div>
  );
}
