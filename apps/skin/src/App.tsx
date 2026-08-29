import { useCallback, useEffect, useMemo, useState } from "react";
import { DeployDialog } from "./components/DeployDialog";
import { HudOverlay } from "./components/HudOverlay";
import { JumpDialog } from "./components/JumpDialog";
import { WorldCanvas } from "./components/WorldCanvas";
import { useAuth } from "./hooks/useAuth";
import { useWorldNavigation } from "./hooks/useWorldNavigation";
import { useWorldStream } from "./hooks/useWorldStream";
import { getMe, postDeploy, postFaucet } from "./lib/api";
import { describeCell } from "./lib/cell";
import { auth, googleProvider, signInWithPopup, signOut } from "./lib/firebase";

type Hover = { x: number; y: number };

export default function App() {
  const { user, ready } = useAuth();
  const [credits, setCredits] = useState<number | null>(null);
  const { creatures, tiles, deployCost, connected } = useWorldStream();
  const {
    view,
    followId,
    focus,
    zoom,
    jumpOpen,
    setJumpOpen,
    setZoom,
    jumpTo,
    followCreature,
    exitFollow,
    enterFollow,
  } = useWorldNavigation(creatures);
  const [busy, setBusy] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [deployCell, setDeployCell] = useState<{ x: number; y: number } | null>(null);
  const [hover, setHover] = useState<Hover | null>(null);

  const cellInfo = useMemo(() => {
    if (!hover) return null;
    return {
      x: hover.x,
      y: hover.y,
      label: describeCell(hover.x, hover.y, tiles, creatures, user?.uid),
    };
  }, [hover, tiles, creatures, user?.uid]);

  const myCreatures = useMemo(
    () =>
      user
        ? creatures
            .filter((c) => c.owner_uid === user.uid)
            .sort((a, b) => b.energy - a.energy || a.id.localeCompare(b.id))
        : [],
    [creatures, user],
  );

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
    if (!ready) return;
    void refreshAccount();
  }, [ready, refreshAccount]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || e.key.toLowerCase() !== "k") return;
      if (deployCell) return;
      e.preventDefault();
      setJumpOpen(true);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [deployCell, setJumpOpen]);

  const signIn = async () => {
    setBusy(true);
    setAuthError(null);
    try {
      await signInWithPopup(auth, googleProvider);
    } catch (err) {
      const code = err instanceof Error && "code" in err ? String((err as { code: string }).code) : "";
      if (code === "auth/operation-not-allowed") {
        setAuthError("Sign-in is not available right now.");
      } else if (code === "auth/unauthorized-domain") {
        setAuthError("Sign-in failed for this site.");
      } else if (code !== "auth/popup-closed-by-user") {
        setAuthError("Could not sign in. Try again.");
      }
    } finally {
      setBusy(false);
    }
  };

  const signOutUser = async () => {
    setBusy(true);
    try {
      await signOut(auth);
      setDeployCell(null);
    } finally {
      setBusy(false);
    }
  };

  const faucet = async () => {
    setBusy(true);
    setActionError(null);
    try {
      const res = await postFaucet(100);
      setCredits(res.credits);
    } catch (err) {
      setActionError(err instanceof Error ? err.message : "Faucet failed");
    } finally {
      setBusy(false);
    }
  };

  const submitDeploy = async (code: string) => {
    if (!user || !deployCell) return;
    setBusy(true);
    setActionError(null);
    try {
      const res = await postDeploy(deployCell.x, deployCell.y, code);
      setCredits(res.credits);
      setDeployCell(null);
      await refreshAccount();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : "Deploy failed");
    } finally {
      setBusy(false);
    }
  };

  const canDeploy = !!user && credits !== null && credits >= deployCost && !busy && !deployCell && !jumpOpen;
  const error = authError ?? actionError;

  const message =
    deployCell
      ? "Write a program for your creature"
      : jumpOpen
        ? "Search by coordinates or creature id"
        : user && canDeploy
          ? "Click an empty cell to deploy"
          : user && credits !== null && credits < deployCost
            ? `Need ${deployCost} credits to deploy`
            : !user
              ? "Sign in to deploy creatures"
              : null;

  return (
    <div className="fixed inset-0 overflow-hidden bg-void">
      <WorldCanvas
        creatures={creatures}
        tiles={tiles}
        canDeploy={canDeploy}
        userUid={user?.uid}
        view={view}
        followId={followId}
        focus={focus}
        initialZoom={zoom}
        onCellSelect={(x, y) => setDeployCell({ x, y })}
        onHover={setHover}
        onManualCamera={exitFollow}
        onZoomChange={setZoom}
      />
      <HudOverlay
        online={connected}
        credits={credits}
        signedIn={!!user}
        busy={busy}
        view={view}
        followId={followId}
        myCreatures={myCreatures}
        cell={cellInfo}
        message={message}
        error={error}
        onViewChange={(next) => (next === "god" ? exitFollow() : enterFollow())}
        onJumpOpen={() => setJumpOpen(true)}
        onFollowCreature={followCreature}
        onSignIn={() => void signIn()}
        onSignOut={() => void signOutUser()}
        onFaucet={() => void faucet()}
      />
      <JumpDialog
        open={jumpOpen}
        creatures={creatures}
        userUid={user?.uid}
        onClose={() => setJumpOpen(false)}
        onJump={jumpTo}
        onFollow={followCreature}
      />
      <DeployDialog
        cell={deployCell}
        deployCost={deployCost}
        busy={busy}
        onDeploy={(code) => void submitDeploy(code)}
        onClose={() => setDeployCell(null)}
      />
    </div>
  );
}
