use serde::{Deserialize, Serialize};

const MAX_TICKS: u64 = 500;
const DEFAULT_START_ENERGY: i64 = 4_000_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum TileKind {
    Solid,
    Food {
        #[serde(skip_serializing_if = "Option::is_none")]
        energy: Option<i64>,
    },
    Corpse {
        #[serde(skip_serializing_if = "Option::is_none")]
        energy: Option<i64>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum TilePlacement {
    At { x: i32, y: i32, kind: TileKind },
    Ahead { kind: TileKind, facing: u8 },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum AssertionOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Assertion {
    Alive {
        expected: bool,
        line: u32,
    },
    Compare {
        field: String,
        op: AssertionOp,
        value: i64,
        at_tick: Option<u64>,
        line: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestSpec {
    pub name: String,
    pub ticks: u64,
    pub facing: u8,
    pub start_energy: i64,
    pub tiles: Vec<TilePlacement>,
    pub assertions: Vec<Assertion>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedTests {
    pub tests: Vec<TestSpec>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssertionResult {
    pub passed: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameSnapshot {
    pub tick: u64,
    pub x: i32,
    pub y: i32,
    pub facing: u8,
    pub energy: i64,
    pub alive: bool,
}

pub fn parse_tests(source: &str) -> ParsedTests {
    let normalized = source.replace("\r\n", "\n");
    let mut out = ParsedTests::default();
    let mut seen = std::collections::HashSet::new();
    let mut i = 0;
    let bytes = normalized.as_bytes();

    while i < bytes.len() {
        if let Some(_rest) = normalized[i..].strip_prefix("#[terrarium::test]") {
            let line = line_number(&normalized, i);
            i += "#[terrarium::test]".len();
            let (name, body, body_start, consumed) = match parse_test_fn(&normalized[i..], line) {
                Ok(v) => v,
                Err(d) => {
                    out.diagnostics.push(d);
                    i += 1;
                    continue;
                }
            };
            i += consumed;

            if seen.contains(&name) {
                out.diagnostics
                    .push(diag_error(line, format!("duplicate test `{name}`")));
                continue;
            }
            seen.insert(name.clone());

            let (spec, diags) = parse_test_body(&name, &body, body_start);
            out.diagnostics.extend(diags);
            if spec.ticks == 0 {
                out.diagnostics.push(diag_error(
                    body_start,
                    format!("test `{name}` must call run_ticks(n)"),
                ));
            } else {
                out.tests.push(spec);
            }
            continue;
        }
        i += 1;
    }

    if out.tests.is_empty() && out.diagnostics.is_empty() && !normalized.trim().is_empty() {
        out.diagnostics.push(diag_error(
            1,
            "expected at least one `#[terrarium::test] fn name() { ... }` block",
        ));
    }

    out
}

fn parse_test_fn(src: &str, attr_line: u32) -> Result<(String, String, u32, usize), Diagnostic> {
    let trimmed = src.trim_start();
    let skip = src.len() - trimmed.len();
    let (_, name) = parse_fn_header(trimmed)?;
    let brace_idx = trimmed.find('{').ok_or_else(|| {
        diag_error(
            attr_line + count_newlines(&src[..skip]),
            "expected fn name() {",
        )
    })?;
    let body_start = attr_line + count_newlines(&src[..skip + brace_idx + 1]);
    let (body, body_len) = extract_brace_body(&trimmed[brace_idx..])
        .ok_or_else(|| diag_error(body_start, "unclosed test function body `{`"))?;
    Ok((name, body, body_start, skip + brace_idx + body_len))
}

fn parse_fn_header(trimmed: &str) -> Result<(String, String), Diagnostic> {
    let rest = trimmed
        .strip_prefix("fn ")
        .ok_or_else(|| diag_error(1, "expected fn name() {"))?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() || !rest[name.len()..].starts_with("() {") {
        return Err(diag_error(1, "expected fn name() {"));
    }
    let matched = format!("fn {name}() {{");
    Ok((matched, name))
}

fn parse_test_body(name: &str, body: &str, body_start: u32) -> (TestSpec, Vec<Diagnostic>) {
    let mut spec = TestSpec {
        name: name.to_string(),
        ticks: 0,
        facing: 0,
        start_energy: DEFAULT_START_ENERGY,
        tiles: vec![],
        assertions: vec![],
    };
    let mut diags = vec![];
    let mut ahead_facing = 0u8;

    for (idx, raw_line) in body.lines().enumerate() {
        let line = body_start + idx as u32;
        let line_text = strip_comment(raw_line).trim();
        if line_text.is_empty() {
            continue;
        }
        let stmt = line_text.trim_end_matches(';').trim();
        if stmt.is_empty() {
            continue;
        }

        if let Some(n) = parse_usize_arg(stmt, "run_ticks") {
            spec.ticks = (n as u64).clamp(1, MAX_TICKS);
            continue;
        }
        if let Some(n) = parse_usize_arg(stmt, "facing") {
            ahead_facing = (n as u8) % 6;
            spec.facing = ahead_facing;
            continue;
        }
        if let Some(n) = parse_i64_arg(stmt, "energy") {
            spec.start_energy = n;
            continue;
        }
        if let Some(tile) = parse_tile_stmt(stmt, line, ahead_facing) {
            match tile {
                Ok(t) => spec.tiles.push(t),
                Err(d) => diags.push(d),
            }
            continue;
        }
        if let Some(assertion) = parse_assertion(stmt, line) {
            match assertion {
                Ok(a) => spec.assertions.push(a),
                Err(d) => diags.push(d),
            }
            continue;
        }

        diags.push(diag_error(line, format!("unknown test statement `{stmt}`")));
    }

    (spec, diags)
}

fn parse_tile_stmt(
    stmt: &str,
    line: u32,
    ahead_facing: u8,
) -> Option<Result<TilePlacement, Diagnostic>> {
    if let Some(inner) = stmt
        .strip_prefix("tile_ahead(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(
            parse_tile_kind(inner.trim(), line).map(|kind| TilePlacement::Ahead {
                kind,
                facing: ahead_facing,
            }),
        );
    }
    if let Some(args) = stmt.strip_prefix("tile(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = args.split(',').map(str::trim).collect();
        if parts.len() != 3 {
            return Some(Err(diag_error(
                line,
                "tile(x, y, kind) expects three arguments",
            )));
        }
        let x = parts[0].parse::<i32>().ok();
        let y = parts[1].parse::<i32>().ok();
        let (Some(x), Some(y)) = (x, y) else {
            return Some(Err(diag_error(line, "tile coordinates must be integers")));
        };
        return Some(parse_tile_kind(parts[2], line).map(|kind| TilePlacement::At { x, y, kind }));
    }
    None
}

fn parse_tile_kind(text: &str, line: u32) -> Result<TileKind, Diagnostic> {
    if text == "solid()" {
        return Ok(TileKind::Solid);
    }
    if text == "food()" {
        return Ok(TileKind::Food { energy: None });
    }
    if let Some(n) = parse_usize_arg(text, "food") {
        return Ok(TileKind::Food {
            energy: Some(n as i64),
        });
    }
    if text == "corpse()" {
        return Ok(TileKind::Corpse { energy: None });
    }
    if let Some(n) = parse_usize_arg(text, "corpse") {
        return Ok(TileKind::Corpse {
            energy: Some(n as i64),
        });
    }
    Err(diag_error(
        line,
        "expected solid(), food(), food(n), corpse(), or corpse(n)",
    ))
}

fn parse_assertion(stmt: &str, line: u32) -> Option<Result<Assertion, Diagnostic>> {
    if stmt == "assert!(alive())" {
        return Some(Ok(Assertion::Alive {
            expected: true,
            line,
        }));
    }
    if stmt == "assert!(!alive())" {
        return Some(Ok(Assertion::Alive {
            expected: false,
            line,
        }));
    }
    if let Some(inner) = stmt
        .strip_prefix("assert_eq!(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(
            parse_field_value(inner, line).map(|(field, value)| Assertion::Compare {
                field,
                op: AssertionOp::Eq,
                value,
                at_tick: None,
                line,
            }),
        );
    }
    if let Some(inner) = stmt
        .strip_prefix("assert_at!(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() != 3 {
            return Some(Err(diag_error(
                line,
                "assert_at!(tick, field(), value) expects three arguments",
            )));
        }
        let tick = parts[0].parse::<u64>().ok();
        let (Some(tick), Ok((field, _))) = (tick, parse_field_value(parts[1], line)) else {
            return Some(Err(diag_error(
                line,
                "assert_at!(tick, field(), value) has invalid field",
            )));
        };
        let value = parts[2]
            .parse::<i64>()
            .ok()
            .ok_or_else(|| diag_error(line, "assert_at value must be an integer"));
        return Some(Ok(Assertion::Compare {
            field,
            op: AssertionOp::Eq,
            value: value.ok()?,
            at_tick: Some(tick),
            line,
        }));
    }
    if let Some(inner) = stmt
        .strip_prefix("assert!(")
        .and_then(|s| s.strip_suffix(')'))
    {
        if let Some((field, op, value)) = parse_compare_expr(inner, line) {
            return Some(Ok(Assertion::Compare {
                field,
                op,
                value,
                at_tick: None,
                line,
            }));
        }
    }
    None
}

fn parse_field_value(inner: &str, line: u32) -> Result<(String, i64), Diagnostic> {
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 2 {
        return Err(diag_error(
            line,
            "expected field(), value — e.g. assert_eq!(x(), 0)",
        ));
    }
    let field = parse_field_name(parts[0], line)?;
    let value = parts[1]
        .parse::<i64>()
        .map_err(|_| diag_error(line, "assertion value must be an integer"))?;
    Ok((field, value))
}

fn parse_field_name(text: &str, line: u32) -> Result<String, Diagnostic> {
    match text {
        "x()" => Ok("x".into()),
        "y()" => Ok("y".into()),
        "facing()" => Ok("facing".into()),
        "energy()" => Ok("energy".into()),
        _ => Err(diag_error(line, "expected x(), y(), facing(), or energy()")),
    }
}

fn parse_compare_expr(inner: &str, line: u32) -> Option<(String, AssertionOp, i64)> {
    for (op_text, op) in [
        ("==", AssertionOp::Eq),
        ("!=", AssertionOp::Ne),
        (">=", AssertionOp::Gte),
        ("<=", AssertionOp::Lte),
        (">", AssertionOp::Gt),
        ("<", AssertionOp::Lt),
    ] {
        if let Some((left, right)) = inner.split_once(op_text) {
            let field = parse_field_name(left.trim(), line).ok()?;
            let value = right.trim().parse::<i64>().ok()?;
            return Some((field, op, value));
        }
    }
    None
}

pub fn evaluate_assertions(
    assertions: &[Assertion],
    frames: &[FrameSnapshot],
) -> Vec<AssertionResult> {
    assertions.iter().map(|a| evaluate_one(a, frames)).collect()
}

fn evaluate_one(assertion: &Assertion, frames: &[FrameSnapshot]) -> AssertionResult {
    match assertion {
        Assertion::Alive { expected, line } => {
            let alive = frames.last().map(|f| f.alive).unwrap_or(false);
            let passed = alive == *expected;
            AssertionResult {
                passed,
                message: if passed {
                    if *expected {
                        "alive".into()
                    } else {
                        "not alive".into()
                    }
                } else if *expected {
                    "expected alive".into()
                } else {
                    "expected dead".into()
                },
                line: Some(*line),
            }
        }
        Assertion::Compare {
            field,
            op,
            value,
            at_tick,
            line,
        } => {
            let frame = pick_frame(frames, *at_tick);
            let actual = frame.and_then(|f| field_value(f, field));
            let passed = actual.is_some_and(|a| compare(*op, a, *value));
            let tick_note = at_tick.map(|t| format!(" at tick {t}")).unwrap_or_default();
            AssertionResult {
                passed,
                message: if passed {
                    format!("{field}{tick_note} ok")
                } else {
                    format!(
                        "{field}{tick_note}: expected {}{value}, got {}",
                        op_label(*op),
                        actual
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "no frame".into())
                    )
                },
                line: Some(*line),
            }
        }
    }
}

fn pick_frame(frames: &[FrameSnapshot], at_tick: Option<u64>) -> Option<&FrameSnapshot> {
    match at_tick {
        Some(tick) => frames
            .iter()
            .find(|f| f.tick == tick)
            .or_else(|| frames.last()),
        None => frames.last(),
    }
}

fn field_value(frame: &FrameSnapshot, field: &str) -> Option<i64> {
    match field {
        "x" => Some(i64::from(frame.x)),
        "y" => Some(i64::from(frame.y)),
        "facing" => Some(i64::from(frame.facing)),
        "energy" => Some(frame.energy),
        _ => None,
    }
}

fn compare(op: AssertionOp, actual: i64, expected: i64) -> bool {
    match op {
        AssertionOp::Eq => actual == expected,
        AssertionOp::Ne => actual != expected,
        AssertionOp::Gt => actual > expected,
        AssertionOp::Gte => actual >= expected,
        AssertionOp::Lt => actual < expected,
        AssertionOp::Lte => actual <= expected,
    }
}

fn op_label(op: AssertionOp) -> &'static str {
    match op {
        AssertionOp::Eq => "",
        AssertionOp::Ne => "!=",
        AssertionOp::Gt => ">",
        AssertionOp::Gte => ">=",
        AssertionOp::Lt => "<",
        AssertionOp::Lte => "<=",
    }
}

fn diag_error(line: u32, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        level: "error".into(),
        message: message.into(),
        line: Some(line),
        column: None,
        area: Some("tests".into()),
    }
}

fn strip_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or(line)
}

fn line_number(src: &str, byte_idx: usize) -> u32 {
    src[..byte_idx].matches('\n').count() as u32 + 1
}

fn count_newlines(s: &str) -> u32 {
    s.matches('\n').count() as u32
}

fn extract_brace_body(src: &str) -> Option<(String, usize)> {
    let mut depth = 0;
    let mut start = None;
    for (i, ch) in src.char_indices() {
        if ch == '{' {
            depth += 1;
            if depth == 1 {
                start = Some(i + 1);
            }
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let body_start = start?;
                return Some((src[body_start..i].to_string(), i + 1));
            }
        }
    }
    None
}

fn parse_usize_arg(stmt: &str, name: &str) -> Option<usize> {
    let prefix = format!("{name}(");
    let suffix = ')';
    if !stmt.starts_with(&prefix) || !stmt.ends_with(suffix) {
        return None;
    }
    let inner = &stmt[prefix.len()..stmt.len() - 1];
    inner.trim().parse().ok()
}

fn parse_i64_arg(stmt: &str, name: &str) -> Option<i64> {
    let prefix = format!("{name}(");
    let suffix = ')';
    if !stmt.starts_with(&prefix) || !stmt.ends_with(suffix) {
        return None;
    }
    let inner = &stmt[prefix.len()..stmt.len() - 1];
    inner.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open_field_test() {
        let src = r#"
#[terrarium::test]
fn open_field() {
    run_ticks(100);
    assert!(alive());
}
"#;
        let parsed = parse_tests(src);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.tests.len(), 1);
        assert_eq!(parsed.tests[0].name, "open_field");
        assert_eq!(parsed.tests[0].ticks, 100);
    }

    #[test]
    fn parses_wall_test_with_tile_ahead() {
        let src = r#"
#[terrarium::test]
fn wall_blocked() {
    tile_ahead(solid());
    run_ticks(10);
    assert_eq!(x(), 0);
}
"#;
        let parsed = parse_tests(src);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.tests[0].tiles.len(), 1);
        assert!(matches!(
            &parsed.tests[0].tiles[0],
            TilePlacement::Ahead {
                kind: TileKind::Solid,
                facing: 0
            }
        ));
    }

    #[test]
    fn reports_unknown_statement() {
        let src = r#"
#[terrarium::test]
fn bad() {
    wall_ahead();
}
"#;
        let parsed = parse_tests(src);
        assert!(parsed
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unknown")));
    }

    #[test]
    fn evaluates_assertions() {
        let frames = vec![FrameSnapshot {
            tick: 10,
            x: 0,
            y: 0,
            facing: 0,
            energy: 100,
            alive: true,
        }];
        let results = evaluate_assertions(
            &[
                Assertion::Alive {
                    expected: true,
                    line: 1,
                },
                Assertion::Compare {
                    field: "x".into(),
                    op: AssertionOp::Eq,
                    value: 0,
                    at_tick: None,
                    line: 2,
                },
            ],
            &frames,
        );
        assert!(results.iter().all(|r| r.passed));
    }
}
