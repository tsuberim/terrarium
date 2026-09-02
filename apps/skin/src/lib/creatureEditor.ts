export const DEFAULT_RUST_SOURCE = `use terrarium_sdk::prelude::*;

pub fn tick() {
    let _ = move_forward();
    sleep();
}
`;

export type CompileDiagnostic = {
  level: string;
  message: string;
  line?: number;
  column?: number;
};

export type SandboxScenario = "open" | "food_ahead" | "wall_ahead" | "corpse_ahead";

export const SANDBOX_SCENARIOS: { id: SandboxScenario; label: string }[] = [
  { id: "open", label: "Open field" },
  { id: "food_ahead", label: "Food ahead" },
  { id: "wall_ahead", label: "Wall ahead" },
  { id: "corpse_ahead", label: "Corpse ahead" },
];

export type SandboxFrame = {
  tick: number;
  x: number;
  y: number;
  facing: number;
  energy: number;
  health: number;
  alive: boolean;
};

export type SandboxResult = {
  ok: boolean;
  alive: boolean;
  ticks_run: number;
  death_reason?: string;
  frames: SandboxFrame[];
  tiles: { x: number; y: number; kind: number; energy?: number }[];
  bench: {
    ticks_run: number;
    start_energy: number;
    end_energy: number;
    total_spent: number;
    per_tick_avg: number;
  };
  error?: string;
};
