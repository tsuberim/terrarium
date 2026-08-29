import { motion } from "framer-motion";
import { useEffect, useRef } from "react";

const CELL = 16;

export function WorldCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let frame = 0;
    let raf = 0;

    const draw = () => {
      const { width, height } = canvas;
      frame += 1;

      ctx.fillStyle = "#030508";
      ctx.fillRect(0, 0, width, height);

      ctx.strokeStyle = "rgba(74, 232, 194, 0.04)";
      ctx.lineWidth = 1;
      for (let x = 0; x <= width; x += CELL) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }
      for (let y = 0; y <= height; y += CELL) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }

      for (let i = 0; i < 18; i++) {
        const t = frame * 0.008 + i * 1.7;
        const x = ((Math.sin(t * 0.9 + i) + 1) / 2) * width;
        const y = ((Math.cos(t * 0.7 + i * 0.5) + 1) / 2) * height;
        const alpha = 0.08 + Math.sin(t) * 0.04;
        ctx.fillStyle = `rgba(123, 109, 255, ${alpha})`;
        ctx.beginPath();
        ctx.arc(x, y, 1.2, 0, Math.PI * 2);
        ctx.fill();
      }

      const pulse = 0.35 + Math.sin(frame * 0.03) * 0.15;
      const grad = ctx.createRadialGradient(
        width / 2,
        height / 2,
        40,
        width / 2,
        height / 2,
        Math.max(width, height) * 0.55,
      );
      grad.addColorStop(0, `rgba(74, 232, 194, ${pulse * 0.04})`);
      grad.addColorStop(1, "rgba(2, 4, 6, 0)");
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, width, height);

      raf = requestAnimationFrame(draw);
    };

    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, []);

  return (
    <motion.div
      className="relative w-full max-w-5xl"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="pointer-events-none absolute -inset-px rounded-2xl bg-gradient-to-b from-biolume/20 via-transparent to-phosphor/10 opacity-40" />
      <div className="glass-panel relative overflow-hidden rounded-2xl p-1 shadow-[0_0_80px_rgba(74,232,194,0.06)]">
        <canvas
          ref={canvasRef}
          width={960}
          height={640}
          aria-label="World simulation viewport"
          className="block h-auto w-full rounded-xl bg-[#030508] [image-rendering:pixelated]"
        />
        <div className="pointer-events-none absolute inset-0 rounded-xl shadow-[inset_0_0_120px_rgba(0,0,0,0.55)]" />
      </div>
      <p className="label mt-3 text-center">Specimen chamber · awaiting life</p>
    </motion.div>
  );
}
