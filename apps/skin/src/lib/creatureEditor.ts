import type { CreatureAction, WorldEvent } from "./api";

export const SCENARIO_DELIMITER = "---";

export const DEFAULT_RUST_SOURCE = `let _ = move_forward();
${SCENARIO_DELIMITER}
#[terrarium::scenario]
fn open_field() {}

#[terrarium::scenario(wall_ahead)]
fn wall_blocked() {}
`;

export type CompileDiagnostic = {
  level: string;
  message: string;
  line?: number;
  column?: number;
};

export type ParsedScenario = {
  id: string;
  label: string;
};

export type SandboxFrame = {
  tick: number;
  x: number;
  y: number;
  facing: number;
  energy: number;
  health: number;
  alive: boolean;
  actions?: CreatureAction[];
  events?: WorldEvent[];
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

const SCENARIO_ATTR =
  /#\[terrarium::scenario(?:\(([^)]*)\))?\]\s*(?:#\[test\]\s*)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)/g;

/** Creature code only — above `---`; scenario attrs below are not compiled. */
export function creatureBody(source: string): string {
  const idx = source.indexOf(SCENARIO_DELIMITER);
  const body = idx >= 0 ? source.slice(0, idx) : source;
  return body.trim();
}

export function scenarioSection(source: string): string {
  const idx = source.indexOf(SCENARIO_DELIMITER);
  return idx >= 0 ? source.slice(idx + SCENARIO_DELIMITER.length) : "";
}

export function parseScenarios(source: string): ParsedScenario[] {
  const scan = `${creatureBody(source)}\n${scenarioSection(source)}`;
  const found: ParsedScenario[] = [];
  const seen = new Set<string>();
  for (const match of scan.matchAll(SCENARIO_ATTR)) {
    const arg = match[1]?.trim().replace(/^"|"$/g, "");
    const fnName = match[2]!;
    const id = (arg || fnName).replace(/::.*/, "").trim();
    if (!id || seen.has(id)) continue;
    seen.add(id);
    found.push({ id, label: titleCase(id) });
  }
  if (!found.length) {
    return [{ id: "open", label: "Open field" }];
  }
  return found;
}

function titleCase(id: string): string {
  return id
    .split("_")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}
