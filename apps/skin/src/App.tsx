import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EventFeed } from "./components/EventFeed";
import { ApiKeysDialog } from "./components/ApiKeysDialog";
import { CreatureStudio } from "./components/CreatureStudio";
import { DevPanel } from "./components/DevPanel";
import { isLocalDev } from "./lib/config";
import { HudOverlay } from "./components/HudOverlay";
import { JumpDialog } from "./components/JumpDialog";
import { WorldCanvas } from "./components/WorldCanvas";
import { useAuth } from "./hooks/useAuth";
import { useWorldNavigation } from "./hooks/useWorldNavigation";
import { useWorldStream } from "./hooks/useWorldStream";
import { getMe, getWorld, postDeploy, postFaucet } from "./lib/api";
import { describeCell } from "./lib/cell";
import { formatDeathNotice, type DeathEvent } from "./lib/death";
import { formatGlimString } from "./lib/glim";
import { auth, googleProvider, signInWithPopup, signOut } from "./lib/firebase";
import { signInWithEmulatorTestUser } from "./lib/emulatorAuth";
import { e2eHooksEnabled, authEmulatorEnabled } from "./lib/config";
import { useE2eBridge } from "./hooks/useE2eBridge";
import type { StudioE2eSlice } from "./lib/e2eBridge";
import { resolveInitialViewerState, clampStudioWidthPct, clampStudioCodeHeightPct, DEFAULT_STUDIO_WIDTH_PCT, DEFAULT_STUDIO_CODE_HEIGHT_PCT } from "./lib/viewerPrefs";

type Hover = { x: number; y: number };

export default function App() {
  const initialSession = useRef(resolveInitialViewerState());
  const { user, ready } = useAuth();
  const [credits, setCredits] = useState<number | null>(null);
  const { creatures, tiles, deployCost, corpseEnergy, simConfig, tick, tickHz, connected, fxEvents, runtimeRef, creaturesLiveRef, tilesLiveRef, setSimConfig, mergeCreatureMeta } =
    useWorldStream();
  const [busy, setBusy] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [deployCell, setDeployCell] = useState<{ x: number; y: number } | null>(initialSession.current.deployCell);
  const [deployDialogOpen, setDeployDialogOpen] = useState(false);
  const [apiKeysOpen, setApiKeysOpen] = useState(false);
  const [studioOpen, setStudioOpen] = useState(initialSession.current.studioOpen);
  const [studioWidthPct, setStudioWidthPct] = useState(() =>
    clampStudioWidthPct(initialSession.current.studioWidthPct ?? DEFAULT_STUDIO_WIDTH_PCT),
  );
  const [studioCodeHeightPct, setStudioCodeHeightPct] = useState(() =>
    clampStudioCodeHeightPct(initialSession.current.studioCodeHeightPct ?? DEFAULT_STUDIO_CODE_HEIGHT_PCT),
  );
  const [hover, setHover] = useState<Hover | null>(null);
  const [deathNotice, setDeathNotice] = useState<string | null>(null);
  const [studioE2e, setStudioE2e] = useState<StudioE2eSlice>({
    testing: false,
    wasmReady: false,
    allTestsPassed: false,
    playback: "idle",
    error: null,
  });

  const studioVisible = !!user && (studioOpen || !!deployCell);

  const onPopShell = useCallback((next: { studioOpen: boolean; deployCell: { x: number; y: number } | null }) => {
    setStudioOpen(next.studioOpen);
    setDeployCell(next.deployCell);
  }, []);

  useEffect(() => {
    if (!ready || user) return;
    setStudioOpen(false);
    setDeployCell(null);
    setDeployDialogOpen(false);
  }, [ready, user]);

  const handleDeployDialogChange = useCallback((open: boolean) => {
    setDeployDialogOpen((prev) => (prev === open ? prev : open));
  }, []);

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
    syncViewport,
  } = useWorldNavigation(
    creatures,
    { studioOpen: studioVisible, deployCell, studioWidthPct, studioCodeHeightPct },
    onPopShell,
  );

  useEffect(() => {
    if (!ready || user || !authEmulatorEnabled()) return;
    void signInWithEmulatorTestUser().catch(() => {});
  }, [ready, user]);

  useEffect(() => {
    if (!ready || !user || !e2eHooksEnabled()) return;
    setStudioOpen(true);
  }, [ready, user]);

  const error = authError ?? actionError;
  useE2eBridge({
    ready,
    signedIn: !!user,
    studioOpen: studioVisible,
    deployCell,
    deployDialogOpen,
    credits,
    testing: studioE2e.testing,
    wasmReady: studioE2e.wasmReady,
    allTestsPassed: studioE2e.allTestsPassed,
    playback: studioE2e.playback,
    error: studioE2e.error ?? error,
    busy,
  });

  useEffect(() => {
    const deaths = fxEvents.filter((e): e is DeathEvent & { at: number } => e.type === "death");
    if (!deaths.length) return;
    const latest = deaths[deaths.length - 1];
    const mine = !!user && latest.owner_uid === user.uid;
    const followed = !!followId && latest.creature_id === followId;
    if (!mine && !followed) return;
    setDeathNotice(formatDeathNotice(latest));
    const timer = window.setTimeout(() => setDeathNotice(null), 6000);
    return () => window.clearTimeout(timer);
  }, [fxEvents, followId, user]);

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
    if (!user) return;
    getWorld()
      .then((world) => mergeCreatureMeta(world.creatures))
      .catch(() => {});
  }, [user, mergeCreatureMeta]);

  useEffect(() => {
    if (!ready) return;
    void refreshAccount();
  }, [ready, refreshAccount]);

  useEffect(() => {
    if (!user) return;
    const cashed = fxEvents.some(
      (e) => e.type === "death" && e.reason === "suicide" && e.owner_uid === user.uid,
    );
    if (cashed) void refreshAccount();
  }, [fxEvents, user, refreshAccount]);

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
      if (authEmulatorEnabled()) {
        await signInWithEmulatorTestUser();
        return;
      }
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
      const res = await postFaucet(10_000_000);
      setCredits(res.credits);
    } catch (err) {
      setActionError(err instanceof Error ? err.message : "Faucet failed");
    } finally {
      setBusy(false);
    }
  };

  const submitDeploy = async (x: number, y: number, code: string, extraEnergy: number, wasmB64?: string) => {
    if (!user) return;
    setBusy(true);
    setActionError(null);
    try {
      const res = await postDeploy(x, y, code, extraEnergy, wasmB64);
      setCredits(res.credits);
      setDeployCell(null);
      await refreshAccount();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : "Deploy failed");
    } finally {
      setBusy(false);
    }
  };

  const canDeploy = !!user && credits !== null && credits >= deployCost && !busy && !studioOpen && !deployCell && !jumpOpen && !deployDialogOpen;
  const canPickDeployCell = !!user && !busy && (deployDialogOpen || studioVisible);

  const message =
    deployDialogOpen || (studioVisible && !deployCell)
      ? "Click a cell on the map to set deploy location"
      : deployCell
      ? "Edit Rust, test, then deploy"
      : jumpOpen
        ? "Search by coordinates or creature id"
        : user && canDeploy
          ? "Click an empty cell to deploy"
          : user && credits !== null && credits < deployCost
            ? `Need ${formatGlimString(deployCost)} to deploy`
            : !user
              ? "Sign in to deploy creatures"
              : null;

  return (
    <div className="fixed inset-0 overflow-hidden bg-void">
      <div className="game-shell">
        <WorldCanvas
          creaturesLiveRef={creaturesLiveRef}
          tilesLiveRef={tilesLiveRef}
          canDeploy={canDeploy || canPickDeployCell}
          userUid={user?.uid}
          senseRange={simConfig?.r_vis ?? 5}
          signalRange={simConfig?.r_sig ?? 5}
          visHalfArc={simConfig?.vis_half_arc ?? 1}
          corpseEnergy={corpseEnergy}
          view={view}
          followId={followId}
          focus={focus}
          initialZoom={zoom}
          onCellSelect={(x, y) => {
            if (!user) return;
            if (deployDialogOpen || studioVisible) {
              setDeployCell({ x, y });
              return;
            }
            setStudioOpen(true);
            setDeployCell({ x, y });
          }}
          onHover={setHover}
          onManualCamera={exitFollow}
          onZoomChange={setZoom}
          onViewportChange={syncViewport}
          runtimeRef={runtimeRef}
          worldTick={tick}
          tickHz={tickHz}
          mapTestId="e2e-world-map"
        />
        <div
          className="hud-shell"
          style={{ left: studioVisible ? `${studioWidthPct}%` : 0 }}
        >
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
            deathNotice={deathNotice}
            error={error}
            onViewChange={(next) => (next === "god" ? exitFollow() : enterFollow())}
            onJumpOpen={() => setJumpOpen(true)}
            onFollowCreature={followCreature}
            onSignIn={() => void signIn()}
            onSignOut={() => void signOutUser()}
            onFaucet={() => void faucet()}
            onApiKeysOpen={() => setApiKeysOpen(true)}
            onCodeOpen={() => {
              if (!user) return;
              setStudioOpen(true);
            }}
          />
          <JumpDialog
            open={jumpOpen}
            creatures={creatures}
            userUid={user?.uid}
            onClose={() => setJumpOpen(false)}
            onJump={jumpTo}
            onFollow={followCreature}
          />
          <EventFeed events={fxEvents} />
          {import.meta.env.DEV && isLocalDev() && (
            <DevPanel config={simConfig} onConfigChange={setSimConfig} />
          )}
        </div>
        <ApiKeysDialog open={apiKeysOpen} busy={busy} onClose={() => setApiKeysOpen(false)} />
      </div>
      {user && (
        <CreatureStudio
          open={studioVisible}
          widthPct={studioWidthPct}
          onWidthChange={setStudioWidthPct}
          codeHeightPct={studioCodeHeightPct}
          onCodeHeightChange={setStudioCodeHeightPct}
          cell={deployCell}
          minExtra={deployCost}
          corpseEnergy={corpseEnergy}
          credits={credits}
          busy={busy}
          tickHz={tickHz}
          senseRange={simConfig?.r_vis ?? 5}
          canCheck
          onDeploy={(x, y, code, extra, wasmB64) => void submitDeploy(x, y, code, extra, wasmB64)}
          onDeployDialogChange={handleDeployDialogChange}
          onE2eSliceChange={setStudioE2e}
          onClose={() => {
            setDeployCell(null);
            setStudioOpen(false);
            setDeployDialogOpen(false);
          }}
        />
      )}
    </div>
  );
}
