import { useCallback, useEffect, useRef, useState } from "react";
import type { Creature } from "../lib/api";
import {
  parseLocation,
  type CameraView,
  type NavFocus,
  writeLocation,
} from "../lib/navigation";
import { loadViewerPrefs, resolveInitialViewerState, saveViewerPrefs } from "../lib/viewerPrefs";

export type FocusTarget = NavFocus & { seq: number };

export function useWorldNavigation(creatures: Creature[]) {
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

  const persist = useCallback(
    (nextView: CameraView, nextFollowId: string | null, nextFocus: NavFocus | null, nextZoom: number) => {
      writeLocation({ view: nextView, followId: nextFollowId, focus: nextFocus });
      saveViewerPrefs({
        view: nextView,
        followId: nextFollowId,
        focus: nextFocus,
        zoom: nextZoom,
        spriteMode: loadViewerPrefs().spriteMode,
      });
    },
    [],
  );

  useEffect(() => {
    persist(view, followId, focus ? { x: focus.x, y: focus.y } : null, zoom);
  }, [view, followId, focus, zoom, persist]);

  useEffect(() => {
    const onPop = () => {
      const nav = parseLocation();
      setView(nav.view);
      setFollowId(nav.followId);
      if (nav.focus) {
        seqRef.current += 1;
        setFocus({ ...nav.focus, seq: seqRef.current });
      } else {
        setFocus(null);
      }
    };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

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
  };
}
