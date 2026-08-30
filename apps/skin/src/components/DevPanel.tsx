import { useEffect, useState } from "react";
import type { SimConfig } from "../lib/api";
import { getSimConfig, patchSimConfig, postClearWorld } from "../lib/api";

type Props = {
  config: SimConfig | null;
  onConfigChange: (c: SimConfig) => void;
};

const DEFAULTS: SimConfig = {
  r_vis: 5,
  r_sig: 5,
  corpse_energy: 1_000_000,
  opcodes_per_tick: 25_000,
  energy_per_opcode: 1,
  move_extra: 25_000,
  dig_extra: 25_000,
  place_extra: 25_000,
  hit_extra: 25_000,
  rotate_extra: 25_000,
  vis_half_arc: 1,
  signal_inbox_cap: 8,
  max_health: 100,
  hit_damage: 34,
  health_regen: 5,
  health_regen_cost: 25_000,
};

export function DevPanel({ config, onConfigChange }: Props) {
  const [open, setOpen] = useState(false);
  const [local, setLocal] = useState<SimConfig>(config ?? DEFAULTS);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (config) setLocal(config);
  }, [config]);

  if (!import.meta.env.DEV) return null;

  const apply = async () => {
    setBusy(true);
    try {
      const next = await patchSimConfig(local);
      onConfigChange(next);
    } catch {
      /* dev endpoint may be unavailable */
    } finally {
      setBusy(false);
    }
  };

  const load = async () => {
    setBusy(true);
    try {
      const next = await getSimConfig();
      setLocal(next);
      onConfigChange(next);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="pointer-events-auto absolute bottom-3 right-3 z-20 sm:bottom-4 sm:right-4">
      <button
        type="button"
        className="hud-chip text-[10px] text-white/40 hover:text-white/65"
        onClick={() => setOpen((v) => !v)}
      >
        {open ? "Close dev" : "Dev"}
      </button>
      {open && (
        <div className="hud-panel mt-1.5 w-56 p-2 text-[10px]">
          <p className="mb-2 font-medium text-white/55">Sim constants</p>
          <div className="space-y-2">
            <label className="flex items-center justify-between gap-2 text-white/40">
              R_vis
              <input
                type="number"
                min={1}
                max={12}
                value={local.r_vis}
                onChange={(e) => setLocal({ ...local, r_vis: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
            <label className="flex items-center justify-between gap-2 text-white/40">
              Vis half-arc
              <input
                type="number"
                min={0}
                max={3}
                value={local.vis_half_arc}
                onChange={(e) => setLocal({ ...local, vis_half_arc: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
            <label className="flex items-center justify-between gap-2 text-white/40">
              R_sig
              <input
                type="number"
                min={1}
                max={12}
                value={local.r_sig}
                onChange={(e) => setLocal({ ...local, r_sig: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
            <label className="flex items-center justify-between gap-2 text-white/40">
              Corpse floor
              <input
                type="number"
                min={0}
                value={local.corpse_energy}
                onChange={(e) => setLocal({ ...local, corpse_energy: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
            <label className="flex items-center justify-between gap-2 text-white/40">
              Opcodes / tick
              <input
                type="number"
                min={1}
                value={local.opcodes_per_tick}
                onChange={(e) => setLocal({ ...local, opcodes_per_tick: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
            <label className="flex items-center justify-between gap-2 text-white/40">
              Energy / opcode
              <input
                type="number"
                min={1}
                value={local.energy_per_opcode}
                onChange={(e) => setLocal({ ...local, energy_per_opcode: Number(e.target.value) })}
                className="w-14 rounded border border-white/[0.08] bg-black/30 px-1 py-0.5 text-right font-mono text-white/70"
              />
            </label>
          </div>
          <div className="mt-2 flex gap-1">
            <button type="button" className="hud-btn-sm flex-1" onClick={() => void load()} disabled={busy}>
              Reload
            </button>
            <button type="button" className="hud-btn-sm flex-1" onClick={() => setLocal(DEFAULTS)} disabled={busy}>
              Defaults
            </button>
            <button type="button" className="hud-btn-sm hud-btn-accent flex-1" onClick={() => void apply()} disabled={busy}>
              Apply
            </button>
          </div>
          <button
            type="button"
            className="hud-btn-sm mt-2 w-full text-red-400/70 hover:text-red-400"
            disabled={busy}
            onClick={() => void postClearWorld()}
          >
            Clear world
          </button>
        </div>
      )}
    </div>
  );
}
