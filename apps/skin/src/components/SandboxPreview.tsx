import { useEffect, useRef } from "react";
import { axialToPixel, hexPathAt, HEX_RADIUS } from "../lib/hex";
import type { SandboxResult } from "../lib/creatureEditor";

const TILE_COLORS: Record<number, string> = {
  1: "rgba(120,110,100,0.9)",
  3: "rgba(180,90,70,0.85)",
  4: "rgba(80,200,120,0.85)",
};

type Props = {
  result: SandboxResult | null;
  frameIndex: number;
};

export function SandboxPreview({ result, frameIndex }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !result) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const frame = result.frames[Math.min(frameIndex, result.frames.length - 1)];
    if (!frame) return;

    const scale = Math.min(w, h) / (HEX_RADIUS * 8);
    const cx = w / 2;
    const cy = h / 2;

    ctx.save();
    ctx.translate(cx, cy);
    ctx.scale(scale, scale);

    for (let dq = -3; dq <= 3; dq++) {
      for (let dr = -3; dr <= 3; dr++) {
        if (Math.abs(dq + dr) > 3) continue;
        hexPathAt(ctx, dq, dr);
        ctx.fillStyle = "rgba(255,255,255,0.04)";
        ctx.fill();
        ctx.strokeStyle = "rgba(255,255,255,0.08)";
        ctx.stroke();
      }
    }

    for (const tile of result.tiles) {
      hexPathAt(ctx, tile.x, tile.y);
      ctx.fillStyle = TILE_COLORS[tile.kind] ?? "rgba(255,255,255,0.2)";
      ctx.fill();
    }

    const { x: px, y: py } = axialToPixel(frame.x, frame.y);
    ctx.beginPath();
    ctx.arc(px, py, HEX_RADIUS * 0.45, 0, Math.PI * 2);
    ctx.fillStyle = "rgba(100,220,180,0.95)";
    ctx.fill();
    ctx.strokeStyle = "rgba(255,255,255,0.5)";
    ctx.lineWidth = 1 / scale;
    ctx.stroke();

    const angle = (frame.facing * Math.PI) / 3;
    ctx.beginPath();
    ctx.moveTo(px, py);
    ctx.lineTo(px + Math.cos(angle) * HEX_RADIUS * 0.55, py + Math.sin(angle) * HEX_RADIUS * 0.55);
    ctx.strokeStyle = "rgba(255,255,255,0.85)";
    ctx.stroke();

    ctx.restore();
  }, [result, frameIndex]);

  if (!result) {
    return (
      <div className="flex h-full min-h-[140px] items-center justify-center rounded-lg border border-white/[0.06] bg-black/25 text-[10px] text-white/30">
        Run Test to preview
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <canvas ref={canvasRef} className="h-[140px] w-full rounded-lg border border-white/[0.06] bg-black/25" />
      {result.death_reason && (
        <p className="font-mono text-[10px] text-amber-400/90">Death: {result.death_reason.replace(/_/g, " ")}</p>
      )}
    </div>
  );
}
