import { useEffect, useRef, type RefObject } from "react";
import type { Creature, WorldTile } from "../lib/api";
import { drawEatFx, EAT_LIFE_MS } from "../lib/eatFx";
import { drawHitFx, HIT_LIFE_MS } from "../lib/hitFx";
import { creatureAnim, drawCreatureSprite, lifeModifiers, SPAWN_LIFE_MS, DEATH_LIFE_MS, BODY_R, type LifeFx, type SpriteMode } from "../lib/creatureSprite";
import { drawTileSprite } from "../lib/tileSprite";
import { drawWorldBackground } from "../lib/worldBackground";
import type { WorldRuntime } from "../lib/worldRuntime";
import {
  cellCenter,
  facingAngle,
  hexIntersectsViewport,
  hexPath,
  hexPathAt,
  HEX_RADIUS,
  hexRangePixelRadius,
  pixelToAxial,
  visibleHexRange,
} from "../lib/hex";

const MIN_ZOOM = 0.35;
const GRID_MIN_ZOOM = 0.55;
const MAX_ZOOM = 8;
const ZOOM_SENSITIVITY = 0.0012;
const DEFAULT_TICK_HZ = 2;

type Camera = { panX: number; panY: number; zoom: number };

type Props = {
  creaturesLiveRef: RefObject<Creature[]>;
  tilesLiveRef: RefObject<WorldTile[]>;
  canDeploy: boolean;
  userUid?: string;
  senseRange?: number;
  signalRange?: number;
  visHalfArc?: number;
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
  runtimeRef: RefObject<WorldRuntime>;
  worldTick?: number;
  tickHz?: number;
};

function clampZoom(z: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));
}

function screenToCell(screenX: number, screenY: number, camera: Camera) {
  const worldX = (screenX - camera.panX) / camera.zoom;
  const worldY = (screenY - camera.panY) / camera.zoom;
  const { q, r } = pixelToAxial(worldX, worldY);
  return { x: q, y: r };
}

function cellToPan(q: number, r: number, w: number, h: number, zoom: number) {
  const { x, y } = cellCenter(q, r);
  return {
    panX: w / 2 - x * zoom,
    panY: h / 2 - y * zoom,
  };
}


export function WorldCanvas({
  creaturesLiveRef,
  tilesLiveRef,
  canDeploy,
  userUid,
  senseRange = 5,
  signalRange = 5,
  visHalfArc = 1,
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
  runtimeRef,
  worldTick = 0,
  tickHz = DEFAULT_TICK_HZ,
}: Props) {
  const tickMs = 1000 / tickHz;
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const cameraRef = useRef<Camera>({ panX: 0, panY: 0, zoom: clampZoom(initialZoom) });
  const draggingRef = useRef({ active: false, lastX: 0, lastY: 0, moved: false, startX: 0, startY: 0 });
  const canDeployRef = useRef(canDeploy);
  const userUidRef = useRef(userUid);
  const senseRangeRef = useRef(senseRange);
  const signalRangeRef = useRef(signalRange);
  const visHalfArcRef = useRef(visHalfArc);
  const corpseEnergyRef = useRef(corpseEnergy);
  const onCellSelectRef = useRef(onCellSelect);
  const onHoverRef = useRef(onHover);
  const onManualCameraRef = useRef(onManualCamera);
  const onZoomChangeRef = useRef(onZoomChange);
  const viewRef = useRef(view);
  const followIdRef = useRef(followId);
  const spriteModeRef = useRef(spriteMode);
  const hoverRef = useRef<{ x: number; y: number } | null>(null);
  const worldTickRef = useRef(worldTick);
  const tickMsRef = useRef(tickMs);

  worldTickRef.current = worldTick;
  tickMsRef.current = tickMs;
  if (runtimeRef.current) runtimeRef.current.setTickHz(tickHz);

  canDeployRef.current = canDeploy;
  userUidRef.current = userUid;
  senseRangeRef.current = senseRange;
  signalRangeRef.current = signalRange;
  visHalfArcRef.current = visHalfArc;
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

    const drawVisionOverlay = (
      q: number,
      r: number,
      lookAngle: number,
      range: number,
      halfArc: number,
      emphasized: boolean,
    ) => {
      const zoom = cameraRef.current.zoom;
      const { x: cx, y: cy } = cellCenter(q, r);
      const angle = lookAngle;
      const wedgeSpan = (Math.PI / 3) * halfArc;
      const wedgeRadius = HEX_RADIUS * (range + 0.55);

      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.arc(cx, cy, wedgeRadius, angle - wedgeSpan, angle + wedgeSpan);
      ctx.closePath();
      ctx.fillStyle = emphasized ? "rgba(74, 232, 194, 0.07)" : "rgba(74, 232, 194, 0.045)";
      ctx.fill();
      ctx.strokeStyle = emphasized ? "rgba(74, 232, 194, 0.42)" : "rgba(74, 232, 194, 0.22)";
      ctx.lineWidth = 1.25 / zoom;
      ctx.stroke();

      const tipX = cx + Math.cos(angle) * HEX_RADIUS * 0.72;
      const tipY = cy + Math.sin(angle) * HEX_RADIUS * 0.72;
      const wing = Math.PI / 2 - 0.35;
      const wingLen = HEX_RADIUS * 0.28;
      ctx.beginPath();
      ctx.moveTo(tipX, tipY);
      ctx.lineTo(
        cx + Math.cos(angle - wing) * wingLen,
        cy + Math.sin(angle - wing) * wingLen,
      );
      ctx.moveTo(tipX, tipY);
      ctx.lineTo(
        cx + Math.cos(angle + wing) * wingLen,
        cy + Math.sin(angle + wing) * wingLen,
      );
      ctx.strokeStyle = emphasized ? "rgba(232, 168, 74, 0.9)" : "rgba(74, 232, 194, 0.65)";
      ctx.lineWidth = (emphasized ? 2 : 1.5) / zoom;
      ctx.stroke();
      ctx.fillStyle = emphasized ? "rgba(232, 168, 74, 0.95)" : "rgba(74, 232, 194, 0.85)";
      ctx.beginPath();
      ctx.arc(tipX, tipY, (emphasized ? 2.2 : 1.6) / zoom, 0, Math.PI * 2);
      ctx.fill();
    };

    const drawCreature = (
      c: Creature,
      q: number,
      r: number,
      followed: boolean,
      energyRefMax: number,
      now: number,
      moving: boolean,
      lookAngle: number,
      eatOpen: number,
      hitFire: number,
      life?: LifeFx,
      drawCx?: number,
      drawCy?: number,
    ) => {
      const mine = c.owner_uid === userUidRef.current;
      const cell = cellCenter(q, r);
      const cx = drawCx ?? cell.x;
      const cy = drawCy ?? cell.y;
      const anim = creatureAnim(c.id, now, moving, lookAngle);

      drawCreatureSprite(
        ctx,
        cx,
        cy,
        mine,
        lookAngle,
        anim,
        c.health,
        c.max_health,
        mine && life?.kind !== "death"
          ? { value: c.energy, floor: corpseEnergyRef.current, refMax: energyRefMax }
          : undefined,
        life,
        hitFire,
        eatOpen,
      );

      if (followed) {
        ctx.strokeStyle = "rgba(232, 168, 74, 0.85)";
        ctx.lineWidth = 2 / cameraRef.current.zoom;
        hexPathAt(ctx, cx, cy, HEX_RADIUS * 0.88);
        ctx.stroke();
      }
    };

    const draw = () => {
      const w = canvas.width / dpr;
      const h = canvas.height / dpr;
      const cam = cameraRef.current;
      const zoom = cam.zoom;
      const now = performance.now();
      const fxNow = Date.now();
      const runtime = runtimeRef.current;
      const frame = runtime?.sample(now, fxNow);

      const followTarget = followIdRef.current;
      if (viewRef.current === "follow" && followTarget && frame) {
        const pose = frame.poses.get(followTarget);
        if (pose) {
          const target = cellToPan(pose.q, pose.r, w, h, zoom);
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

      drawWorldBackground(ctx, left, top, right, bottom, now);

      const { qMin, qMax, rMin, rMax } = visibleHexRange(left, top, right, bottom);

      if (zoom >= GRID_MIN_ZOOM) {
        ctx.strokeStyle = "rgba(74, 232, 194, 0.014)";
        ctx.lineWidth = 1 / zoom;
        for (let q = qMin; q <= qMax; q++) {
          for (let r = rMin; r <= rMax; r++) {
            if (!hexIntersectsViewport(q, r, left, top, right, bottom)) continue;
            hexPath(ctx, q, r);
            ctx.stroke();
          }
        }
      }

      const displayTiles =
        runtime?.displayTiles(tilesLiveRef.current ?? [], now, fxNow) ?? tilesLiveRef.current ?? [];

      for (const t of displayTiles) {
        const { x: cx, y: cy } = cellCenter(t.x, t.y);
        if (cx + HEX_RADIUS < left || cx - HEX_RADIUS > right || cy + HEX_RADIUS < top || cy - HEX_RADIUS > bottom) {
          continue;
        }
        drawTileSprite(ctx, t, now);
      }

      const senseR = senseRangeRef.current;
      const halfArc = visHalfArcRef.current;
      const creatures = creaturesLiveRef.current ?? [];
      for (const c of creatures) {
        if (c.owner_uid !== userUidRef.current) continue;
        const pose = frame?.poses.get(c.id) ?? runtime?.pose(c.id, c);
        if (!pose) continue;
        drawVisionOverlay(pose.q, pose.r, pose.angle, senseR, halfArc, c.id === followTarget);
      }

      const uid = userUidRef.current;
      const floor = corpseEnergyRef.current;
      let energyRefMax = floor + 10_000_000;
      if (uid) {
        for (const c of creatures) {
          if (c.owner_uid === uid) {
            energyRefMax = Math.max(energyRefMax, c.energy);
          }
        }
      }

      for (const c of creatures) {
        const pose = frame?.poses.get(c.id) ?? runtime?.pose(c.id, c);
        if (!pose) continue;

        let cx = pose.px;
        let cy = pose.py;
        let life: LifeFx | undefined;
        const spawnEntry = frame?.spawnLife.get(c.id);
        if (spawnEntry) {
          const t = Math.min(1, (fxNow - spawnEntry.at) / SPAWN_LIFE_MS);
          life = { kind: "spawn", t };
          const mod = lifeModifiers(life);
          const from = cellCenter(spawnEntry.fromQ, spawnEntry.fromR);
          cx = from.x + (pose.px - from.x) * mod.posT;
          cy = from.y + (pose.py - from.y) * mod.posT;
        }

        if (cx + HEX_RADIUS < left || cx - HEX_RADIUS > right || cy + HEX_RADIUS < top || cy - HEX_RADIUS > bottom) {
          continue;
        }
        const eatOpen = runtime?.eatOpen(c.id, now) ?? 0;
        const hitFire = runtime?.hitFire(c.id, fxNow) ?? 0;
        drawCreature(
          c,
          pose.q,
          pose.r,
          c.id === followIdRef.current,
          energyRefMax,
          now,
          pose.moving,
          pose.angle,
          eatOpen,
          hitFire,
          life,
          cx,
          cy,
        );
      }

      for (const [, ghost] of frame?.deathGhosts ?? []) {
        const t = Math.min(1, (fxNow - ghost.at) / DEATH_LIFE_MS);
        const life: LifeFx = { kind: "death", t };
        const { x: cx, y: cy } = cellCenter(ghost.creature.x, ghost.creature.y);
        if (cx + HEX_RADIUS < left || cx - HEX_RADIUS > right || cy + HEX_RADIUS < top || cy - HEX_RADIUS > bottom) {
          continue;
        }
        drawCreature(ghost.creature, ghost.creature.x, ghost.creature.y, false, energyRefMax, now, false, facingAngle(((ghost.creature.facing ?? 0) % 6 + 6) % 6), 0, 0, life, cx, cy);
      }

      const hover = hoverRef.current;
      if (hover) {
        ctx.strokeStyle = "rgba(74, 232, 194, 0.55)";
        ctx.lineWidth = 1.5 / zoom;
        hexPath(ctx, hover.x, hover.y);
        ctx.stroke();
      }

      ctx.restore();

      for (const fx of runtime?.fxForRender() ?? []) {
        const fxMs =
          fx.type === "hit"
            ? HIT_LIFE_MS
            : fx.type === "death"
              ? DEATH_LIFE_MS
              : fx.type === "spawn"
                ? SPAWN_LIFE_MS
                : fx.type === "eat"
                  ? EAT_LIFE_MS
                  : 600;
        const fxStart = fx.type === "eat" ? (runtime?.eatStartAt(fx) ?? fx.at) : fx.at;
        const age =
          fx.type === "eat"
            ? (now - fxStart) / fxMs
            : (fxNow - fxStart) / fxMs;
        if (age < 0 || age >= 1) continue;
        const alpha = 1 - age ** 1.4;
        ctx.save();
        ctx.translate(panX, panY);
        ctx.scale(zoom, zoom);

        if (fx.type === "signal") {
          const from = cellCenter(fx.from_x, fx.from_y);
          const fxFrom = from.x;
          const fyFrom = from.y;
          const signalAlpha = alpha * 0.35;
          if (fx.broadcast) {
            const maxR = hexRangePixelRadius(signalRangeRef.current);
            ctx.strokeStyle = `rgba(232, 168, 74, ${signalAlpha * 0.45})`;
            ctx.lineWidth = 0.85 / zoom;
            ctx.beginPath();
            ctx.arc(fxFrom, fyFrom, maxR * age, 0, Math.PI * 2);
            ctx.stroke();
          } else if (fx.to_id) {
            const target = creatures.find((c) => c.id === fx.to_id);
            const mine = target?.owner_uid === userUidRef.current;
            if (mine && target) {
              const pose = frame?.poses.get(target.id) ?? runtime?.pose(target.id, target);
              const to = pose ? { x: pose.px, y: pose.py } : cellCenter(target.x, target.y);
              ctx.strokeStyle = `rgba(123, 109, 255, ${signalAlpha * 0.55})`;
              ctx.lineWidth = 1 / zoom;
              ctx.beginPath();
              ctx.moveTo(fxFrom, fyFrom);
              ctx.lineTo(to.x, to.y);
              ctx.stroke();
              ctx.fillStyle = `rgba(123, 109, 255, ${signalAlpha * 0.5})`;
              ctx.beginPath();
              ctx.arc(to.x, to.y, HEX_RADIUS * 0.12, 0, Math.PI * 2);
              ctx.fill();
            }
          }
        } else if (fx.type === "spawn") {
          const fromQ = fx.parent_x ?? fx.x;
          const fromR = fx.parent_y ?? fx.y;
          const from = cellCenter(fromQ, fromR);
          const burst = 1 - age;
          ctx.strokeStyle = `rgba(74, 232, 194, ${alpha * 0.4 * burst})`;
          ctx.lineWidth = 2 / zoom;
          ctx.beginPath();
          ctx.arc(from.x, from.y, HEX_RADIUS * (0.15 + age * 0.55), 0, Math.PI * 2);
          ctx.stroke();
          ctx.fillStyle = `rgba(74, 232, 194, ${alpha * 0.12 * burst})`;
          ctx.beginPath();
          ctx.arc(from.x, from.y, HEX_RADIUS * (0.1 + age * 0.35), 0, Math.PI * 2);
          ctx.fill();
        } else if (fx.type === "hit") {
          const target = cellCenter(fx.x, fx.y);
          const actor = creatures.find((c) => c.id === fx.actor_id);
          if (actor && runtime) {
            const pose = frame?.poses.get(actor.id) ?? runtime.pose(actor.id, actor);
            drawHitFx(ctx, {
              age,
              alpha,
              actorCx: pose.px,
              actorCy: pose.py,
              look: pose.angle,
              targetX: target.x,
              targetY: target.y,
              lineWidth: 2 / zoom,
            });
          }
        } else if (fx.type === "eat") {
          const actor = creatures.find((c) => c.id === fx.actor_id);
          if (actor && runtime) {
            const pose = frame?.poses.get(actor.id) ?? runtime.pose(actor.id, actor);
            if (pose.moving || pose.rotating) continue;
            const foodCell = cellCenter(fx.x, fx.y);
            drawEatFx(ctx, {
              age,
              alpha,
              actorCx: pose.px,
              actorCy: pose.py,
              look: pose.angle,
              foodCx: foodCell.x,
              foodCy: foodCell.y,
              tileKind: fx.tile_kind,
              bodyR: BODY_R,
            });
          }
        }

        ctx.restore();
      }

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
  // Mount-only: canvas loop reads live refs each frame; stable ref identities by design.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !focus) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    const { panX, panY } = cellToPan(focus.x, focus.y, w, h, cameraRef.current.zoom);
    cameraRef.current.panX = panX;
    cameraRef.current.panY = panY;
  }, [focus]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    container.style.cursor = canDeploy ? "crosshair" : "grab";
  }, [canDeploy]);

  return (
    <div ref={containerRef} className="absolute inset-0 touch-none">
      <canvas
        ref={canvasRef}
        aria-label="World simulation"
        className="block h-full w-full"
      />
    </div>
  );
}
