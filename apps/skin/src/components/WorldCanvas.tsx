import { useEffect, useRef } from "react";
import type { Creature, WorldTile } from "../lib/api";

const CELL = 16;
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 3;
const ZOOM_SENSITIVITY = 0.0012;

type Camera = { panX: number; panY: number; zoom: number };

type Props = {
  creatures: Creature[];
  tiles?: WorldTile[];
  canDeploy: boolean;
  userUid?: string;
  view: "god" | "follow";
  followId?: string | null;
  focus?: { x: number; y: number; seq: number } | null;
  initialZoom?: number;
  onCellSelect: (x: number, y: number) => void;
  onHover?: (hover: { x: number; y: number } | null) => void;
  onManualCamera?: () => void;
  onZoomChange?: (zoom: number) => void;
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

export function WorldCanvas({
  creatures,
  tiles = [],
  canDeploy,
  userUid,
  view,
  followId = null,
  focus = null,
  initialZoom = 1,
  onCellSelect,
  onHover,
  onManualCamera,
  onZoomChange,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const cameraRef = useRef<Camera>({ panX: 0, panY: 0, zoom: clampZoom(initialZoom) });
  const draggingRef = useRef({ active: false, lastX: 0, lastY: 0, moved: false, startX: 0, startY: 0 });
  const creaturesRef = useRef(creatures);
  const tilesRef = useRef(tiles);
  const canDeployRef = useRef(canDeploy);
  const userUidRef = useRef(userUid);
  const onCellSelectRef = useRef(onCellSelect);
  const onHoverRef = useRef(onHover);
  const onManualCameraRef = useRef(onManualCamera);
  const onZoomChangeRef = useRef(onZoomChange);
  const viewRef = useRef(view);
  const followIdRef = useRef(followId);
  const hoverRef = useRef<{ x: number; y: number } | null>(null);
  const displayPos = useRef(new Map<string, { x: number; y: number }>());

  creaturesRef.current = creatures;
  tilesRef.current = tiles;
  canDeployRef.current = canDeploy;
  userUidRef.current = userUid;
  onCellSelectRef.current = onCellSelect;
  onHoverRef.current = onHover;
  onManualCameraRef.current = onManualCamera;
  onZoomChangeRef.current = onZoomChange;
  viewRef.current = view;
  followIdRef.current = followId;

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

    const drawCreature = (c: Creature, dx: number, dy: number, followed: boolean) => {
      const cx = dx * CELL + CELL / 2;
      const cy = dy * CELL + CELL / 2;
      const mine = c.owner_uid === userUidRef.current;
      const color = mine ? "#4ae8c2" : "#7b6dff";

      ctx.fillStyle = mine ? "rgba(74, 232, 194, 0.2)" : "rgba(123, 109, 255, 0.2)";
      ctx.fillRect(dx * CELL + 1, dy * CELL + 1, CELL - 2, CELL - 2);

      if (followed) {
        ctx.strokeStyle = "rgba(232, 168, 74, 0.85)";
        ctx.lineWidth = 2 / cameraRef.current.zoom;
        ctx.beginPath();
        ctx.arc(cx, cy, CELL * 0.38, 0, Math.PI * 2);
        ctx.stroke();
      }

      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(cx, cy, CELL * 0.28, 0, Math.PI * 2);
      ctx.fill();
    };

    const draw = () => {
      const w = canvas.width / dpr;
      const h = canvas.height / dpr;
      const cam = cameraRef.current;
      const zoom = cam.zoom;
      frame += 1;

      const followTarget = followIdRef.current;
      if (viewRef.current === "follow" && followTarget) {
        const c = creaturesRef.current.find((cr) => cr.id === followTarget);
        if (c) {
          const pos = displayPos.current.get(c.id) ?? { x: c.x, y: c.y };
          const target = cellToPan(pos.x, pos.y, w, h, zoom);
          cam.panX += (target.panX - cam.panX) * 0.18;
          cam.panY += (target.panY - cam.panY) * 0.18;
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

      const liveIds = new Set<string>();
      for (const c of creaturesRef.current ?? []) {
        liveIds.add(c.id);
        const pos = displayPos.current.get(c.id) ?? { x: c.x, y: c.y };
        pos.x += (c.x - pos.x) * 0.35;
        pos.y += (c.y - pos.y) * 0.35;
        displayPos.current.set(c.id, pos);

        const px = pos.x * CELL;
        const py = pos.y * CELL;
        if (px + CELL < gx0 || px > gx1 || py + CELL < gy0 || py > gy1) continue;
        drawCreature(c, pos.x, pos.y, c.id === followIdRef.current);
      }
      for (const id of displayPos.current.keys()) {
        if (!liveIds.has(id)) displayPos.current.delete(id);
      }

      ctx.restore();

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
