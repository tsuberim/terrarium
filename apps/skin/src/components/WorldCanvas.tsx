import { useEffect, useRef } from "react";
import type { FxEvent } from "../hooks/useWorldStream";
import type { Creature, WorldTile } from "../lib/api";
import { creatureSprite, drawCreatureSprite, type SpriteMode } from "../lib/creatureSprite";

const CELL = 16;
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 3;
const ZOOM_SENSITIVITY = 0.0012;
const TICK_MS = 100;

type Camera = { panX: number; panY: number; zoom: number };
type AnimState = { fromX: number; fromY: number; toX: number; toY: number; start: number };

type Props = {
  creatures: Creature[];
  tiles?: WorldTile[];
  canDeploy: boolean;
  userUid?: string;
  senseRange?: number;
  corpseEnergy?: number;
  view: "god" | "follow";
  followId?: string | null;
  spriteMode?: SpriteMode;
  focus?: { x: number; y: number; seq: number } | null;
  initialZoom?: number;
  onCellSelect: (x: number, y: number) => void;
  onHover?: (hover: { x: number; y: number } | null) => void;
  onManualCamera?: () => void;
  onZoomChange?: (zoom: number) => void;
  fxEvents?: FxEvent[];
};

function clampZoom(z: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));
}

function screenToCell(screenX: number, screenY: number, camera: Camera) {
  const worldX = (screenX - camera.panX) / camera.zoom;
  const worldY = (screenY - camera.panY) / camera.zoom;
  return { x: Math.floor(worldX / CELL), y: Math.floor(worldY / CELL) };
}

function cellToPan(x: number, y: number, w: number, h: number, zoom: number) {
  return {
    panX: w / 2 - (x * CELL + CELL / 2) * zoom,
    panY: h / 2 - (y * CELL + CELL / 2) * zoom,
  };
}

function easeOutCubic(t: number) {
  return 1 - (1 - t) ** 3;
}

function stepCreatureAnim(
  id: string,
  serverX: number,
  serverY: number,
  now: number,
  displayPos: Map<string, { x: number; y: number }>,
  animState: Map<string, AnimState>,
) {
  let anim = animState.get(id);
  if (!anim || anim.toX !== serverX || anim.toY !== serverY) {
    const current = displayPos.get(id);
    anim = {
      fromX: current?.x ?? serverX,
      fromY: current?.y ?? serverY,
      toX: serverX,
      toY: serverY,
      start: now,
    };
    animState.set(id, anim);
  }

  const t = easeOutCubic(Math.min(1, (now - anim.start) / TICK_MS));
  const pos = {
    x: anim.fromX + (anim.toX - anim.fromX) * t,
    y: anim.fromY + (anim.toY - anim.fromY) * t,
  };
  displayPos.set(id, pos);
  return pos;
}

export function WorldCanvas({
  creatures,
  tiles = [],
  canDeploy,
  userUid,
  senseRange = 5,
  corpseEnergy = 1_000_000,
  view,
  followId = null,
  spriteMode = "id",
  focus = null,
  initialZoom = 1,
  onCellSelect,
  onHover,
  onManualCamera,
  onZoomChange,
  fxEvents = [],
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const cameraRef = useRef<Camera>({ panX: 0, panY: 0, zoom: clampZoom(initialZoom) });
  const draggingRef = useRef({ active: false, lastX: 0, lastY: 0, moved: false, startX: 0, startY: 0 });
  const creaturesRef = useRef(creatures);
  const tilesRef = useRef(tiles);
  const canDeployRef = useRef(canDeploy);
  const userUidRef = useRef(userUid);
  const senseRangeRef = useRef(senseRange);
  const corpseEnergyRef = useRef(corpseEnergy);
  const onCellSelectRef = useRef(onCellSelect);
  const onHoverRef = useRef(onHover);
  const onManualCameraRef = useRef(onManualCamera);
  const onZoomChangeRef = useRef(onZoomChange);
  const viewRef = useRef(view);
  const followIdRef = useRef(followId);
  const spriteModeRef = useRef(spriteMode);
  const hoverRef = useRef<{ x: number; y: number } | null>(null);
  const displayPos = useRef(new Map<string, { x: number; y: number }>());
  const animState = useRef(new Map<string, AnimState>());

  const fxRef = useRef(fxEvents);
  const creaturesFxRef = useRef(creatures);
  const userUidFxRef = useRef(userUid);

  fxRef.current = fxEvents;
  creaturesFxRef.current = creatures;
  userUidFxRef.current = userUid;
  creaturesRef.current = creatures;
  tilesRef.current = tiles;
  canDeployRef.current = canDeploy;
  userUidRef.current = userUid;
  senseRangeRef.current = senseRange;
  corpseEnergyRef.current = corpseEnergy;
  onCellSelectRef.current = onCellSelect;
  onHoverRef.current = onHover;
  onManualCameraRef.current = onManualCamera;
  onZoomChangeRef.current = onZoomChange;
  viewRef.current = view;
  followIdRef.current = followId;
  spriteModeRef.current = spriteMode;

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let frame = 0;
    let raf = 0;
    let dpr = 1;

    const resize = () => {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      const { clientWidth, clientHeight } = container;
      canvas.width = Math.floor(clientWidth * dpr);
      canvas.height = Math.floor(clientHeight * dpr);
      canvas.style.width = `${clientWidth}px`;
      canvas.style.height = `${clientHeight}px`;
    };

    const drawSenseRange = (dx: number, dy: number, r: number) => {
      const x0 = (dx - r) * CELL;
      const y0 = (dy - r) * CELL;
      const size = (2 * r + 1) * CELL;
      ctx.fillStyle = "rgba(74, 232, 194, 0.05)";
      ctx.fillRect(x0, y0, size, size);
      ctx.strokeStyle = "rgba(74, 232, 194, 0.22)";
      ctx.lineWidth = 1 / cameraRef.current.zoom;
      ctx.setLineDash([4 / cameraRef.current.zoom, 3 / cameraRef.current.zoom]);
      ctx.strokeRect(x0 + 0.5, y0 + 0.5, size - 1, size - 1);
      ctx.setLineDash([]);
    };

    const drawEnergyBar = (dx: number, dy: number, energy: number, floor: number, refMax: number) => {
      const span = Math.max(refMax - floor, 1);
      const ratio = Math.max(0, Math.min(1, (energy - floor) / span));
      const x = dx * CELL + 2;
      const y = dy * CELL + CELL - 4;
      const w = CELL - 4;
      const h = 2;
      const lw = 1 / cameraRef.current.zoom;
      ctx.fillStyle = "rgba(0, 0, 0, 0.45)";
      ctx.fillRect(x, y, w, h);
      if (ratio <= 0.12) ctx.fillStyle = "rgba(232, 100, 90, 0.9)";
      else if (ratio <= 0.35) ctx.fillStyle = "rgba(232, 168, 74, 0.9)";
      else ctx.fillStyle = "rgba(74, 232, 194, 0.95)";
      ctx.fillRect(x, y, Math.max(lw, w * ratio), h);
    };

    const drawCreature = (
      c: Creature,
      dx: number,
      dy: number,
      followed: boolean,
      energyRefMax: number,
    ) => {
      const mine = c.owner_uid === userUidRef.current;
      const sprite = creatureSprite(c, spriteModeRef.current);

      drawCreatureSprite(ctx, dx, dy, CELL, sprite, mine);

      if (followed) {
        const cx = dx * CELL + CELL / 2;
        const cy = dy * CELL + CELL / 2;
        ctx.strokeStyle = "rgba(232, 168, 74, 0.85)";
        ctx.lineWidth = 2 / cameraRef.current.zoom;
        ctx.beginPath();
        ctx.arc(cx, cy, CELL * 0.42, 0, Math.PI * 2);
        ctx.stroke();
      }

      if (mine) {
        drawEnergyBar(dx, dy, c.energy, corpseEnergyRef.current, energyRefMax);
      }
    };

    const draw = () => {
      const w = canvas.width / dpr;
      const h = canvas.height / dpr;
      const cam = cameraRef.current;
      const zoom = cam.zoom;
      const now = performance.now();
      frame += 1;

      const liveIds = new Set<string>();
      for (const c of creaturesRef.current ?? []) {
        liveIds.add(c.id);
        stepCreatureAnim(c.id, c.x, c.y, now, displayPos.current, animState.current);
      }
      for (const id of displayPos.current.keys()) {
        if (!liveIds.has(id)) {
          displayPos.current.delete(id);
          animState.current.delete(id);
        }
      }

      const followTarget = followIdRef.current;
      if (viewRef.current === "follow" && followTarget) {
        const pos = displayPos.current.get(followTarget);
        if (pos) {
          const target = cellToPan(pos.x, pos.y, w, h, zoom);
          cam.panX = target.panX;
          cam.panY = target.panY;
        }
      }

      const panX = cam.panX;
      const panY = cam.panY;

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = "#030508";
      ctx.fillRect(0, 0, w, h);

      ctx.save();
      ctx.translate(panX, panY);
      ctx.scale(zoom, zoom);

      const left = -panX / zoom;
      const top = -panY / zoom;
      const right = (w - panX) / zoom;
      const bottom = (h - panY) / zoom;

      const gx0 = Math.floor(left / CELL) * CELL;
      const gy0 = Math.floor(top / CELL) * CELL;
      const gx1 = Math.ceil(right / CELL) * CELL;
      const gy1 = Math.ceil(bottom / CELL) * CELL;

      ctx.strokeStyle = "rgba(74, 232, 194, 0.045)";
      ctx.lineWidth = 1 / zoom;
      for (let x = gx0; x <= gx1; x += CELL) {
        ctx.beginPath();
        ctx.moveTo(x, gy0);
        ctx.lineTo(x, gy1);
        ctx.stroke();
      }
      for (let y = gy0; y <= gy1; y += CELL) {
        ctx.beginPath();
        ctx.moveTo(gx0, y);
        ctx.lineTo(gx1, y);
        ctx.stroke();
      }

      for (const t of tilesRef.current ?? []) {
        const px = t.x * CELL;
        const py = t.y * CELL;
        if (px + CELL < gx0 || px > gx1 || py + CELL < gy0 || py > gy1) continue;
        if (t.kind === 1) {
          ctx.fillStyle = "rgba(90, 100, 110, 0.55)";
          ctx.fillRect(px + 1, py + 1, CELL - 2, CELL - 2);
        } else if (t.kind === 3) {
          ctx.fillStyle = "rgba(232, 168, 74, 0.35)";
          ctx.fillRect(px + 2, py + 2, CELL - 4, CELL - 4);
          ctx.fillStyle = "rgba(232, 168, 74, 0.7)";
          ctx.beginPath();
          ctx.arc(px + CELL / 2, py + CELL / 2, CELL * 0.15, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      const senseR = senseRangeRef.current;
      for (const c of creaturesRef.current ?? []) {
        if (c.owner_uid !== userUidRef.current) continue;
        const pos = displayPos.current.get(c.id);
        if (!pos) continue;
        const x0 = (pos.x - senseR) * CELL;
        const y0 = (pos.y - senseR) * CELL;
        const x1 = (pos.x + senseR + 1) * CELL;
        const y1 = (pos.y + senseR + 1) * CELL;
        if (x1 < gx0 || x0 > gx1 || y1 < gy0 || y0 > gy1) continue;
        drawSenseRange(pos.x, pos.y, senseR);
      }

      const uid = userUidRef.current;
      const floor = corpseEnergyRef.current;
      let energyRefMax = floor + 10_000_000;
      if (uid) {
        for (const c of creaturesRef.current ?? []) {
          if (c.owner_uid === uid) {
            energyRefMax = Math.max(energyRefMax, c.energy);
          }
        }
      }

      for (const c of creaturesRef.current ?? []) {
        const pos = displayPos.current.get(c.id);
        if (!pos) continue;

        const px = pos.x * CELL;
        const py = pos.y * CELL;
        if (px + CELL < gx0 || px > gx1 || py + CELL < gy0 || py > gy1) continue;
        drawCreature(c, pos.x, pos.y, c.id === followIdRef.current, energyRefMax);
      }

      const hover = hoverRef.current;
      if (hover) {
        ctx.strokeStyle = "rgba(74, 232, 194, 0.55)";
        ctx.lineWidth = 1.5 / zoom;
        ctx.strokeRect(hover.x * CELL + 0.5, hover.y * CELL + 0.5, CELL - 1, CELL - 1);
      }

      ctx.restore();

      const fxNow = Date.now();
      for (const fx of fxRef.current) {
        const age = (fxNow - fx.at) / 500;
        if (age >= 1) continue;
        const alpha = 1 - age;
        ctx.save();
        ctx.translate(panX, panY);
        ctx.scale(zoom, zoom);

        if (fx.type === "signal") {
          const fxFrom = fx.from_x * CELL + CELL / 2;
          const fyFrom = fx.from_y * CELL + CELL / 2;
          if (fx.broadcast) {
            ctx.strokeStyle = `rgba(232, 168, 74, ${alpha * 0.55})`;
            ctx.lineWidth = 1.5 / zoom;
            ctx.beginPath();
            ctx.arc(fxFrom, fyFrom, CELL * (0.4 + age * 1.2), 0, Math.PI * 2);
            ctx.stroke();
          } else if (fx.to_id) {
            const target = creaturesFxRef.current.find((c) => c.id === fx.to_id);
            const mine = target?.owner_uid === userUidFxRef.current;
            if (mine && target) {
              const pos = displayPos.current.get(target.id) ?? { x: target.x, y: target.y };
              const tx = pos.x * CELL + CELL / 2;
              const ty = pos.y * CELL + CELL / 2;
              ctx.strokeStyle = `rgba(123, 109, 255, ${alpha * 0.85})`;
              ctx.lineWidth = 2 / zoom;
              ctx.beginPath();
              ctx.moveTo(fxFrom, fyFrom);
              ctx.lineTo(tx, ty);
              ctx.stroke();
              ctx.fillStyle = `rgba(123, 109, 255, ${alpha * 0.9})`;
              ctx.beginPath();
              ctx.arc(tx, ty, CELL * 0.12, 0, Math.PI * 2);
              ctx.fill();
            }
          }
        } else if (fx.type === "spawn") {
          const px = fx.x * CELL + CELL / 2;
          const py = fx.y * CELL + CELL / 2;
          ctx.strokeStyle = `rgba(74, 232, 194, ${alpha * 0.5})`;
          ctx.lineWidth = 1.5 / zoom;
          ctx.beginPath();
          ctx.arc(px, py, CELL * 0.35 * (1 + age * 0.5), 0, Math.PI * 2);
          ctx.stroke();
        }

        ctx.restore();
      }

      const pulse = 0.35 + Math.sin(frame * 0.03) * 0.15;
      const grad = ctx.createRadialGradient(w / 2, h / 2, 40, w / 2, h / 2, Math.max(w, h) * 0.55);
      grad.addColorStop(0, `rgba(74, 232, 194, ${pulse * 0.035})`);
      grad.addColorStop(1, "rgba(2, 4, 6, 0)");
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, w, h);

      raf = requestAnimationFrame(draw);
    };

    const zoomAt = (clientX: number, clientY: number, factor: number) => {
      const rect = canvas.getBoundingClientRect();
      const sx = clientX - rect.left;
      const sy = clientY - rect.top;
      const cam = cameraRef.current;
      const worldX = (sx - cam.panX) / cam.zoom;
      const worldY = (sy - cam.panY) / cam.zoom;
      cam.zoom = clampZoom(cam.zoom * factor);
      cam.panX = sx - worldX * cam.zoom;
      cam.panY = sy - worldY * cam.zoom;
      onZoomChangeRef.current?.(cam.zoom);
    };

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      zoomAt(e.clientX, e.clientY, Math.exp(-e.deltaY * ZOOM_SENSITIVITY));
    };

    const onPointerDown = (e: PointerEvent) => {
      if (e.button !== 0) return;
      const rect = canvas.getBoundingClientRect();
      draggingRef.current = {
        active: true,
        lastX: e.clientX,
        lastY: e.clientY,
        startX: e.clientX - rect.left,
        startY: e.clientY - rect.top,
        moved: false,
      };
      container.setPointerCapture(e.pointerId);
      container.style.cursor = "grabbing";
    };

    const setHover = (clientX: number, clientY: number) => {
      const rect = canvas.getBoundingClientRect();
      const sx = clientX - rect.left;
      const sy = clientY - rect.top;
      const cell = screenToCell(sx, sy, cameraRef.current);
      const prev = hoverRef.current;
      if (prev?.x === cell.x && prev?.y === cell.y) return;
      hoverRef.current = cell;
      onHoverRef.current?.(cell);
    };

    const clearHover = () => {
      hoverRef.current = null;
      onHoverRef.current?.(null);
    };

    const onPointerMove = (e: PointerEvent) => {
      const drag = draggingRef.current;
      if (drag.active) {
        const dx = e.clientX - drag.lastX;
        const dy = e.clientY - drag.lastY;
        if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
          drag.moved = true;
          if (viewRef.current === "follow") {
            viewRef.current = "god";
            followIdRef.current = null;
            onManualCameraRef.current?.();
          }
        }
        drag.lastX = e.clientX;
        drag.lastY = e.clientY;
        cameraRef.current.panX += dx;
        cameraRef.current.panY += dy;
        return;
      }
      setHover(e.clientX, e.clientY);
    };

    const onPointerUp = (e: PointerEvent) => {
      const drag = draggingRef.current;
      if (!drag.active) return;
      drag.active = false;
      container.releasePointerCapture(e.pointerId);
      container.style.cursor = canDeployRef.current ? "crosshair" : "grab";

      if (!drag.moved && canDeployRef.current) {
        const cell = screenToCell(drag.startX, drag.startY, cameraRef.current);
        onCellSelectRef.current(cell.x, cell.y);
      }
    };

    resize();
    raf = requestAnimationFrame(draw);
    container.style.cursor = canDeployRef.current ? "crosshair" : "grab";
    const observer = new ResizeObserver(resize);
    observer.observe(container);
    window.addEventListener("resize", resize);
    container.addEventListener("wheel", onWheel, { passive: false });
    container.addEventListener("pointerdown", onPointerDown);
    container.addEventListener("pointermove", onPointerMove);
    container.addEventListener("pointerup", onPointerUp);
    container.addEventListener("pointercancel", onPointerUp);
    container.addEventListener("pointerleave", clearHover);

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      window.removeEventListener("resize", resize);
      container.removeEventListener("wheel", onWheel);
      container.removeEventListener("pointerdown", onPointerDown);
      container.removeEventListener("pointermove", onPointerMove);
      container.removeEventListener("pointerup", onPointerUp);
      container.removeEventListener("pointercancel", onPointerUp);
      container.removeEventListener("pointerleave", clearHover);
    };
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !focus) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    const { panX, panY } = cellToPan(focus.x, focus.y, w, h, cameraRef.current.zoom);
    cameraRef.current.panX = panX;
    cameraRef.current.panY = panY;
  }, [focus?.seq]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    container.style.cursor = canDeploy ? "crosshair" : "grab";
  }, [canDeploy]);

  return (
    <div ref={containerRef} className="absolute inset-0 touch-none bg-[#030508]">
      <canvas
        ref={canvasRef}
        aria-label="World simulation"
        className="block h-full w-full [image-rendering:pixelated]"
      />
    </div>
  );
}
