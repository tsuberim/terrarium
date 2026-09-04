import type { CreatureAction, WorldEvent } from "./api";

export const DEFAULT_RUST_SOURCE = `let _ = move_forward();
`;

export const DEFAULT_TESTS_SOURCE = `#[terrarium::test]
fn open_field() {
    energy(15000000);
    run_ticks(100);
    assert!(alive());
}

#[terrarium::test]
fn wall_blocked() {
    tile_ahead(solid());
    run_ticks(10);
    assert_eq!(x(), 0);
}
`;

export type CompileDiagnostic = {
  level: string;
  message: string;
  line?: number;
  column?: number;
  area?: "source" | "tests";
};

export type TestTileKind =
  | { kind: "solid" }
  | { kind: "food"; energy?: number }
  | { kind: "corpse"; energy?: number };

export type TestTile =
  | { at: "coord"; x: number; y: number; tile: TestTileKind }
  | { at: "ahead"; facing: number; tile: TestTileKind };

export type TestAssertion =
  | { type: "alive"; expected: boolean; line: number }
  | {
      type: "compare";
      field: "x" | "y" | "facing" | "energy";
      op: "eq" | "ne" | "gt" | "gte" | "lt" | "lte";
      value: number;
      atTick?: number;
      line: number;
    };

export type TestSpec = {
  name: string;
  ticks: number;
  facing: number;
  start_energy: number;
  tiles: Array<
    | { At: { x: number; y: number; kind: unknown } }
    | { Ahead: { kind: unknown; facing: number } }
  >;
  assertions: Array<
    | { Alive: { expected: boolean; line: number } }
    | {
        Compare: {
          field: string;
          op: string;
          value: number;
          at_tick: number | null;
          line: number;
        };
      }
  >;
};

export type ParsedTest = {
  name: string;
  label: string;
  spec: TestSpec;
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

export type AssertionResult = {
  passed: boolean;
  message: string;
  line?: number;
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
  test_passed: boolean;
  assertions: AssertionResult[];
  error?: string;
};

const MAX_TICKS = 500;
const DEFAULT_START_ENERGY = 4_000_000;

export function normalizeCompileDiagnostics(
  diagnostics: { level: string; message: string; line?: number; column?: number; area?: string }[],
): CompileDiagnostic[] {
  return diagnostics.map((d) => ({
    level: d.level,
    message: d.message,
    line: d.line,
    column: d.column,
    area: d.area === "tests" ? "tests" : d.area === "source" ? "source" : undefined,
  }));
}

export function parseTests(source: string): { tests: ParsedTest[]; diagnostics: CompileDiagnostic[] } {
  const normalized = source.replace(/\r\n/g, "\n");
  const diagnostics: CompileDiagnostic[] = [];
  const tests: ParsedTest[] = [];
  const seen = new Set<string>();
  let i = 0;

  while (i < normalized.length) {
    if (normalized.startsWith("#[terrarium::test]", i)) {
      const attrLine = lineNumber(normalized, i);
      i += "#[terrarium::test]".length;
      const rest = normalized.slice(i);
      const fnMatch = /^\s*fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\)\s*\{/.exec(rest);
      if (!fnMatch) {
        diagnostics.push(testError(attrLine, "expected fn name() { after #[terrarium::test]"));
        i += 1;
        continue;
      }
      const name = fnMatch[1]!;
      const fnStart = i + fnMatch.index! + fnMatch[0].indexOf("{");
      const bodyStartLine = lineNumber(normalized, fnStart + 1);
      const extracted = extractBraceBody(normalized.slice(fnStart));
      if (!extracted) {
        diagnostics.push(testError(bodyStartLine, "unclosed test function body `{`"));
        i += 1;
        continue;
      }
      i = fnStart + extracted.end;

      if (seen.has(name)) {
        diagnostics.push(testError(attrLine, `duplicate test \`${name}\``));
        continue;
      }
      seen.add(name);

      const { spec, diags } = parseTestBody(name, extracted.body, bodyStartLine);
      diagnostics.push(...diags);
      if (spec.ticks === 0) {
        diagnostics.push(testError(bodyStartLine, `test \`${name}\` must call run_ticks(n)`));
      } else {
        tests.push({ name, label: titleCase(name), spec: toWireSpec(spec) });
      }
      continue;
    }
    i += 1;
  }

  if (tests.length === 0 && diagnostics.length === 0 && normalized.trim()) {
    diagnostics.push(testError(1, "expected at least one #[terrarium::test] fn name() { ... } block"));
  }

  return { tests, diagnostics };
}

type InternalSpec = {
  name: string;
  ticks: number;
  facing: number;
  startEnergy: number;
  tiles: TestTile[];
  assertions: TestAssertion[];
};

function parseTestBody(name: string, body: string, bodyStart: number): { spec: InternalSpec; diags: CompileDiagnostic[] } {
  const spec: InternalSpec = {
    name,
    ticks: 0,
    facing: 0,
    startEnergy: DEFAULT_START_ENERGY,
    tiles: [],
    assertions: [],
  };
  const diags: CompileDiagnostic[] = [];
  let aheadFacing = 0;

  for (const [idx, rawLine] of body.split("\n").entries()) {
    const line = bodyStart + idx;
    const stmt = stripComment(rawLine).trim().replace(/;$/, "").trim();
    if (!stmt) continue;

    const runTicks = parseNumArg(stmt, "run_ticks");
    if (runTicks !== null) {
      spec.ticks = Math.min(MAX_TICKS, Math.max(1, runTicks));
      continue;
    }
    const facing = parseNumArg(stmt, "facing");
    if (facing !== null) {
      aheadFacing = facing % 6;
      spec.facing = aheadFacing;
      continue;
    }
    const energy = parseSignedArg(stmt, "energy");
    if (energy !== null) {
      spec.startEnergy = energy;
      continue;
    }

    const tile = parseTileStmt(stmt, line, aheadFacing);
    if (tile) {
      if ("error" in tile) diags.push(tile.error);
      else spec.tiles.push(tile.tile);
      continue;
    }

    const assertion = parseAssertion(stmt, line);
    if (assertion) {
      if ("error" in assertion) diags.push(assertion.error);
      else spec.assertions.push(assertion.assertion);
      continue;
    }

    diags.push(testError(line, `unknown test statement \`${stmt}\``));
  }

  return { spec, diags };
}

function toWireSpec(spec: InternalSpec): TestSpec {
  return {
    name: spec.name,
    ticks: spec.ticks,
    facing: spec.facing,
    start_energy: spec.startEnergy,
    tiles: spec.tiles.map((t) =>
      t.at === "ahead"
        ? { Ahead: { kind: wireTileKind(t.tile), facing: t.facing } }
        : { At: { x: t.x, y: t.y, kind: wireTileKind(t.tile) } },
    ),
    assertions: spec.assertions.map((a) => {
      if (a.type === "alive") return { Alive: { expected: a.expected, line: a.line } };
      return {
        Compare: {
          field: a.field,
          op: opWire(a.op),
          value: a.value,
          at_tick: a.atTick ?? null,
          line: a.line,
        },
      };
    }),
  };
}

function opWire(op: "eq" | "ne" | "gt" | "gte" | "lt" | "lte"): string {
  const map = { eq: "Eq", ne: "Ne", gt: "Gt", gte: "Gte", lt: "Lt", lte: "Lte" } as const;
  return map[op];
}

function wireTileKind(tile: TestTileKind): unknown {
  if (tile.kind === "solid") return "Solid";
  if (tile.kind === "food") return { Food: { energy: tile.energy ?? null } };
  return { Corpse: { energy: tile.energy ?? null } };
}

function parseTileStmt(
  stmt: string,
  line: number,
  aheadFacing: number,
): { tile: TestTile } | { error: CompileDiagnostic } | null {
  const aheadMatch = /^tile_ahead\((.+)\)$/.exec(stmt);
  if (aheadMatch) {
    const kind = parseTileKind(aheadMatch[1]!.trim(), line);
    if ("error" in kind) return kind;
    return { tile: { at: "ahead", facing: aheadFacing, tile: kind.kind } };
  }
  const tileMatch = /^tile\((.+)\)$/.exec(stmt);
  if (tileMatch) {
    const parts = tileMatch[1]!.split(",").map((p) => p.trim());
    if (parts.length !== 3) {
      return { error: testError(line, "tile(x, y, kind) expects three arguments") };
    }
    const x = Number.parseInt(parts[0]!, 10);
    const y = Number.parseInt(parts[1]!, 10);
    if (Number.isNaN(x) || Number.isNaN(y)) {
      return { error: testError(line, "tile coordinates must be integers") };
    }
    const kind = parseTileKind(parts[2]!, line);
    if ("error" in kind) return kind;
    return { tile: { at: "coord", x, y, tile: kind.kind } };
  }
  return null;
}

function parseTileKind(text: string, line: number): { kind: TestTileKind } | { error: CompileDiagnostic } {
  if (text === "solid()") return { kind: { kind: "solid" } };
  if (text === "food()") return { kind: { kind: "food" } };
  if (text === "corpse()") return { kind: { kind: "corpse" } };
  const food = parseNumArg(text, "food");
  if (food !== null) return { kind: { kind: "food", energy: food } };
  const corpse = parseNumArg(text, "corpse");
  if (corpse !== null) return { kind: { kind: "corpse", energy: corpse } };
  return { error: testError(line, "expected solid(), food(), food(n), corpse(), or corpse(n)") };
}

function parseAssertion(
  stmt: string,
  line: number,
): { assertion: TestAssertion } | { error: CompileDiagnostic } | null {
  if (stmt === "assert!(alive())") return { assertion: { type: "alive", expected: true, line } };
  if (stmt === "assert!(!alive())") return { assertion: { type: "alive", expected: false, line } };

  const eqMatch = /^assert_eq!\((.+)\)$/.exec(stmt);
  if (eqMatch) {
    const parsed = parseFieldValue(eqMatch[1]!, line);
    if ("error" in parsed) return parsed;
    return { assertion: { type: "compare", field: parsed.field, op: "eq", value: parsed.value, line } };
  }

  const atMatch = /^assert_at!\((.+)\)$/.exec(stmt);
  if (atMatch) {
    const parts = atMatch[1]!.split(",").map((p) => p.trim());
    if (parts.length !== 3) {
      return { error: testError(line, "assert_at!(tick, field(), value) expects three arguments") };
    }
    const tick = Number.parseInt(parts[0]!, 10);
    const parsed = parseFieldValue(`${parts[1]}, ${parts[2]}`, line);
    if ("error" in parsed) return parsed;
    if (Number.isNaN(tick)) return { error: testError(line, "assert_at tick must be an integer") };
    return {
      assertion: { type: "compare", field: parsed.field, op: "eq", value: parsed.value, atTick: tick, line },
    };
  }

  const assertMatch = /^assert!\((.+)\)$/.exec(stmt);
  if (assertMatch) {
    const inner = assertMatch[1]!;
    for (const [opText, op] of [
      ["==", "eq"],
      ["!=", "ne"],
      [">=", "gte"],
      ["<=", "lte"],
      [">", "gt"],
      ["<", "lt"],
    ] as const) {
      const idx = inner.indexOf(opText);
      if (idx === -1) continue;
      const left = inner.slice(0, idx).trim();
      const right = inner.slice(idx + opText.length).trim();
      const field = parseFieldName(left, line);
      if ("error" in field) return field;
      const value = Number.parseInt(right, 10);
      if (Number.isNaN(value)) return { error: testError(line, "assertion value must be an integer") };
      return { assertion: { type: "compare", field: field.field, op, value, line } };
    }
  }

  return null;
}

function parseFieldValue(
  inner: string,
  line: number,
): { field: "x" | "y" | "facing" | "energy"; value: number } | { error: CompileDiagnostic } {
  const parts = inner.split(",").map((p) => p.trim());
  if (parts.length !== 2) {
    return { error: testError(line, "expected field(), value — e.g. assert_eq!(x(), 0)") };
  }
  const field = parseFieldName(parts[0]!, line);
  if ("error" in field) return field;
  const value = Number.parseInt(parts[1]!, 10);
  if (Number.isNaN(value)) return { error: testError(line, "assertion value must be an integer") };
  return { field: field.field, value };
}

function parseFieldName(
  text: string,
  line: number,
): { field: "x" | "y" | "facing" | "energy" } | { error: CompileDiagnostic } {
  switch (text) {
    case "x()":
      return { field: "x" };
    case "y()":
      return { field: "y" };
    case "facing()":
      return { field: "facing" };
    case "energy()":
      return { field: "energy" };
    default:
      return { error: testError(line, "expected x(), y(), facing(), or energy()") };
  }
}

function testError(line: number, message: string): CompileDiagnostic {
  return { level: "error", message, line, area: "tests" };
}

function stripComment(line: string): string {
  return line.split("//")[0] ?? line;
}

function lineNumber(src: string, idx: number): number {
  return src.slice(0, idx).split("\n").length;
}

function extractBraceBody(src: string): { body: string; end: number } | null {
  let depth = 0;
  let start: number | null = null;
  for (let i = 0; i < src.length; i++) {
    const ch = src[i];
    if (ch === "{") {
      depth += 1;
      if (depth === 1) start = i + 1;
    } else if (ch === "}") {
      depth -= 1;
      if (depth === 0 && start !== null) {
        return { body: src.slice(start, i), end: i + 1 };
      }
    }
  }
  return null;
}

function parseNumArg(stmt: string, name: string): number | null {
  const m = new RegExp(`^${name}\\((\\d+)\\)$`).exec(stmt);
  return m ? Number.parseInt(m[1]!, 10) : null;
}

function parseSignedArg(stmt: string, name: string): number | null {
  const m = new RegExp(`^${name}\\((-?\\d+)\\)$`).exec(stmt);
  return m ? Number.parseInt(m[1]!, 10) : null;
}

function titleCase(id: string): string {
  return id
    .split("_")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}
