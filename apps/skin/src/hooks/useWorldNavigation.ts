import { useCallback, useEffect, useRef, useState } from "react";
import type { Creature } from "../lib/api";
import { parseLocation, type CameraView, type NavFocus } from "../lib/navigation";
import { clampZoom, loadViewerPrefs, persistViewerState, resolveInitialViewerState, type ViewerPrefs } from "../lib/viewerPrefs";

export type FocusTarget = NavFocus & { seq: number };

type ShellState = Pick<ViewerPrefs, "studioOpen" | "deployCell" | "studioWidthPct" | "studioCodeHeightPct">;

export type PopShellState = Pick<ViewerPrefs, "studioOpen" | "deployCell">;

export function useWorldNavigation(
  creatures: Creature[],
  shell: ShellState,
  onPopShell?: (next: PopShellState) => void,
) {
  const initial = useRef(resolveInitialViewerState());
  const seqRef = useRef(0);
  const [view, setView] = useState<CameraView>(initial.current.view);
  const [followId, setFollowId] = useState<string | null>(initial.current.followId);
  const [zoom, setZoom] = useState(initial.current.zoom);
  const [focus, setFocus] = useState<FocusTarget | null>(() => {
    if (!initial.current.focus) return null;
    seqRef.current += 1;
    return { ...initial.current.focus, seq: seqRef.current };
  });
  const [jumpOpen, setJumpOpen] = useState(false);

  useEffect(() => {
    persistViewerState({
      view,
      followId,
      focus: focus ? { x: focus.x, y: focus.y } : null,
      zoom,
      studioOpen: shell.studioOpen,
      deployCell: shell.deployCell,
      studioWidthPct: shell.studioWidthPct,
      studioCodeHeightPct: shell.studioCodeHeightPct,
    });
  }, [view, followId, focus, zoom, shell.studioOpen, shell.deployCell, shell.studioWidthPct, shell.studioCodeHeightPct]);

  useEffect(() => {
    const onPop = () => {
      const nav = parseLocation();
      const stored = loadViewerPrefs();
      setView(nav.view);
      setFollowId(nav.followId);
      setZoom(clampZoom(nav.zoom));
      const nextFocus = nav.studioOpen ? stored.focus : nav.focus;
      if (nextFocus) {
        seqRef.current += 1;
        setFocus({ ...nextFocus, seq: seqRef.current });
      } else {
        setFocus(null);
      }
      onPopShell?.({
        studioOpen: nav.studioOpen,
        deployCell: nav.studioOpen && nav.focus ? nav.focus : null,
      });
    };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, [onPopShell]);

  useEffect(() => {
    if (view !== "follow" || !followId || creatures.length === 0) return;
    if (creatures.some((c) => c.id === followId)) return;
    const prefix = followId.toLowerCase();
    const match = creatures.filter((c) => c.id.toLowerCase().startsWith(prefix));
    if (match.length === 1) {
      setFollowId(match[0].id);
      return;
    }
    setView("god");
    setFollowId(null);
  }, [creatures, view, followId]);

  const jumpTo = useCallback((x: number, y: number) => {
    setView("god");
    setFollowId(null);
    seqRef.current += 1;
    setFocus({ x, y, seq: seqRef.current });
    setJumpOpen(false);
  }, []);

  const followCreature = useCallback((id: string) => {
    setView("follow");
    setFollowId(id);
    setFocus(null);
    setJumpOpen(false);
  }, []);

  const exitFollow = useCallback(() => {
    setView("god");
    setFollowId(null);
  }, []);

  const enterFollow = useCallback(() => {
    if (followId) {
      setView("follow");
      return;
    }
    setJumpOpen(true);
  }, [followId]);

  const syncViewport = useCallback((center: NavFocus, nextZoom: number) => {
    setView("god");
    setFollowId(null);
    seqRef.current += 1;
    setFocus({ x: center.x, y: center.y, seq: seqRef.current });
    setZoom(clampZoom(nextZoom));
  }, []);

  return {
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
  };
}
