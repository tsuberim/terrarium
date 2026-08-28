import init, { JsWorld } from "./pkg/terrarium_kernel.js";

const DEMOS = {
  wander: `# wander — fixed thrust loop (deterministic)
thrust 50 20
sleep
thrust -40 45
sleep
thrust -30 -55
sleep
thrust 55 -15
sleep
jump 0
`,
  chase: `# chase — sense nearest body, thrust toward it
sense
jnz 0 4
sleep
jump 0
thrust_toward 70
sleep
jump 0
`,
  sit: `# sit — sleep is free
sleep
jump 0
`,
};

/* Flat palette — chunky pixels, no soft glow */
const CELL_COLORS = [
  "#4dff88",
  "#e0a84a",
  "#5ecfc8",
  "#9ad64a",
  "#3dbf7a",
];
const INERT_COLOR = "#8a6a2e";
const WORLD_BG = "#0a1210";
const SELECT_COLOR = "#e8f0ea";
const GHOST_ALPHA = 0.38;

const TICKS_PER_FRAME = 1;
const MS_PER_TICK = 50;
/** Short-axis pixel count; long axis follows viewport aspect (square pixels). */
const PIXEL_SHORT = 480;

let world = null;
let selectedCell = 0;
let lastSnapshot = null;
let running = true;
let lastTickAt = 0;
let statusEl;
let programEl;
let cellSelect;
let consoleEl;
let toggleBtn;
let canvas;
let ctx;
let bufW = PIXEL_SHORT;
let bufH = PIXEL_SHORT;

function setStatus(msg, kind) {
  statusEl.textContent = msg || "";
  statusEl.className = "status" + (kind ? " " + kind : "");
}

function envBadge() {
  var meta = document.querySelector('meta[name="terrarium-env"]');
  var env = ((meta && meta.getAttribute("content")) || "staging").trim() || "staging";
  var badge = document.getElementById("env-badge");
  if (badge) {
    badge.textContent = env;
    badge.setAttribute("data-env", env);
  }
  document.documentElement.setAttribute("data-env", env);
}

function setConsoleOpen(open) {
  if (open) {
    consoleEl.hidden = false;
    toggleBtn.setAttribute("aria-expanded", "true");
  } else {
    consoleEl.hidden = true;
    toggleBtn.setAttribute("aria-expanded", "false");
  }
}

function seedWorld() {
  world = new JsWorld();
  var a = world.spawnCell(5000, -120000, -80000);
  var b = world.spawnCell(4000, 100000, 60000);
  var c = world.spawnCell(3500, -40000, 140000);
  world.setProgramText(a, DEMOS.wander);
  world.setProgramText(b, DEMOS.chase);
  world.setProgramText(c, DEMOS.sit);
  // Inert crumb already in the box — absorb is an explicit verb.
  var dumper = world.spawnCell(800, 80000, -100000);
  world.dumpMatter(dumper, 400);
  world.setProgramText(dumper, DEMOS.sit);
  selectedCell = a;
  refreshCellSelect();
  programEl.value = DEMOS.wander;
  markDemo("wander");
  setStatus("world seeded. wander on cell " + a + ".", "ok");
}

function cellLabel(id) {
  return "cell " + id;
}

function refreshCellSelect() {
  var snap = JSON.parse(world.snapshot());
  lastSnapshot = snap;
  cellSelect.innerHTML = "";
  snap.cells.forEach(function (c) {
    var opt = document.createElement("option");
    opt.value = String(c.id);
    opt.textContent = cellLabel(c.id);
    cellSelect.appendChild(opt);
  });
  if (snap.cells.some(function (c) { return c.id === selectedCell; })) {
    cellSelect.value = String(selectedCell);
  } else if (snap.cells.length) {
    selectedCell = snap.cells[0].id;
    cellSelect.value = String(selectedCell);
  }
}

function markDemo(name) {
  document.querySelectorAll(".demos button").forEach(function (btn) {
    btn.classList.toggle("active", btn.getAttribute("data-demo") === name);
  });
}

function setupCanvas() {
  var vw = Math.max(1, window.innerWidth || 1);
  var vh = Math.max(1, window.innerHeight || 1);
  if (vw >= vh) {
    bufH = PIXEL_SHORT;
    bufW = Math.max(PIXEL_SHORT, Math.round(PIXEL_SHORT * (vw / vh)));
  } else {
    bufW = PIXEL_SHORT;
    bufH = Math.max(PIXEL_SHORT, Math.round(PIXEL_SHORT * (vh / vw)));
  }
  if (canvas.width !== bufW || canvas.height !== bufH) {
    canvas.width = bufW;
    canvas.height = bufH;
  }
  ctx.imageSmoothingEnabled = false;
}

function bodyRadiusPx(mass, scale) {
  var r = Math.sqrt(mass * 2000);
  r = Math.max(5000, Math.min(r, 24000));
  return Math.max(3, Math.round(r * scale * 1.15));
}

function fillCircle(x, y, r, color, alpha) {
  var cx = Math.round(x);
  var cy = Math.round(y);
  var rr = Math.max(1, Math.round(r));
  ctx.globalAlpha = alpha == null ? 1 : alpha;
  ctx.fillStyle = color;
  for (var dy = -rr; dy <= rr; dy++) {
    for (var dx = -rr; dx <= rr; dx++) {
      if (dx * dx + dy * dy <= rr * rr) {
        ctx.fillRect(cx + dx, cy + dy, 1, 1);
      }
    }
  }
  ctx.globalAlpha = 1;
}

function strokeCircle(x, y, r, color, alpha) {
  var cx = Math.round(x);
  var cy = Math.round(y);
  var rr = Math.max(1, Math.round(r));
  var inner = (rr - 1) * (rr - 1);
  var outer = rr * rr;
  ctx.globalAlpha = alpha == null ? 1 : alpha;
  ctx.fillStyle = color;
  for (var dy = -rr; dy <= rr; dy++) {
    for (var dx = -rr; dx <= rr; dx++) {
      var d2 = dx * dx + dy * dy;
      if (d2 <= outer && d2 >= inner) {
        ctx.fillRect(cx + dx, cy + dy, 1, 1);
      }
    }
  }
  ctx.globalAlpha = 1;
}

function worldToScreen(x, y, view) {
  return {
    px: view.cx + x * view.scale,
    py: view.cy + y * view.scale,
  };
}

function wrapOffsets(px, py, pr, view) {
  var offsets = [[0, 0]];
  var left = view.cx - view.widthPx / 2;
  var right = view.cx + view.widthPx / 2;
  var top = view.cy - view.heightPx / 2;
  var bottom = view.cy + view.heightPx / 2;
  var band = pr + 3;
  if (px - band < left) offsets.push([view.widthPx, 0]);
  if (px + band > right) offsets.push([-view.widthPx, 0]);
  if (py - band < top) offsets.push([0, view.heightPx]);
  if (py + band > bottom) offsets.push([0, -view.heightPx]);
  if (px - band < left && py - band < top) offsets.push([view.widthPx, view.heightPx]);
  if (px + band > right && py - band < top) offsets.push([-view.widthPx, view.heightPx]);
  if (px - band < left && py + band > bottom) offsets.push([view.widthPx, -view.heightPx]);
  if (px + band > right && py + band > bottom) offsets.push([-view.widthPx, -view.heightPx]);
  return offsets;
}

function drawBlob(px, py, pr, color, selected) {
  fillCircle(px, py, pr, color);
  if (selected) {
    strokeCircle(px, py, pr + 2, SELECT_COLOR);
  }
}

function drawBlobWithWraps(px, py, pr, color, selected, view) {
  var offsets = wrapOffsets(px, py, pr, view);
  for (var i = 0; i < offsets.length; i++) {
    var ox = offsets[i][0];
    var oy = offsets[i][1];
    var ghost = ox !== 0 || oy !== 0;
    var alpha = ghost ? GHOST_ALPHA : 1;
    fillCircle(px + ox, py + oy, pr, color, alpha);
    if (selected && !ghost) {
      strokeCircle(px + ox, py + oy, pr + 2, SELECT_COLOR);
    }
  }
}

function draw() {
  ctx.fillStyle = WORLD_BG;
  ctx.fillRect(0, 0, bufW, bufH);
  if (!lastSnapshot) return;

  var worldWidth = lastSnapshot.width || 800000;
  var worldHeight = lastSnapshot.height || 800000;
  var scale = Math.max(bufW / worldWidth, bufH / worldHeight);
  var view = {
    cx: bufW / 2,
    cy: bufH / 2,
    scale: scale,
    widthPx: worldWidth * scale,
    heightPx: worldHeight * scale,
    left: bufW / 2 - (worldWidth * scale) / 2,
    top: bufH / 2 - (worldHeight * scale) / 2,
  };

  lastSnapshot.inert.forEach(function (d) {
    var pos = worldToScreen(d.x, d.y, view);
    var pr = Math.max(1, Math.round(bodyRadiusPx(d.mass, scale) * 0.65));
    drawBlobWithWraps(pos.px, pos.py, pr, INERT_COLOR, false, view);
  });

  lastSnapshot.cells.forEach(function (c) {
    var color = CELL_COLORS[c.id % CELL_COLORS.length];
    var pos = worldToScreen(c.x, c.y, view);
    var pr = bodyRadiusPx(c.mass, scale);
    drawBlobWithWraps(pos.px, pos.py, pr, color, c.id === selectedCell, view);
  });
}

function frame(now) {
  if (running && world && now - lastTickAt >= MS_PER_TICK) {
    lastTickAt = now;
    for (var i = 0; i < TICKS_PER_FRAME; i++) {
      world.tick();
    }
    lastSnapshot = JSON.parse(world.snapshot());
    syncSelectLabels(lastSnapshot);
  }
  draw();
  requestAnimationFrame(frame);
}

function syncSelectLabels(snap) {
  var opts = cellSelect.options;
  for (var i = 0; i < opts.length; i++) {
    var id = Number(opts[i].value);
    var cell = snap.cells.find(function (c) { return c.id === id; });
    if (cell) {
      opts[i].textContent = cellLabel(cell.id);
    }
  }
  var living = new Set(snap.cells.map(function (c) { return String(c.id); }));
  for (var j = opts.length - 1; j >= 0; j--) {
    if (!living.has(opts[j].value)) {
      opts[j].remove();
    }
  }
  snap.cells.forEach(function (c) {
    var found = false;
    for (var k = 0; k < opts.length; k++) {
      if (opts[k].value === String(c.id)) {
        found = true;
        break;
      }
    }
    if (!found) {
      var opt = document.createElement("option");
      opt.value = String(c.id);
      opt.textContent = cellLabel(c.id);
      cellSelect.appendChild(opt);
    }
  });
}

function ensureLivingSelection() {
  if (!lastSnapshot || !lastSnapshot.cells.length) return false;
  if (lastSnapshot.cells.some(function (c) { return c.id === selectedCell; })) {
    return true;
  }
  selectedCell = lastSnapshot.cells[0].id;
  cellSelect.value = String(selectedCell);
  return true;
}

function onRun() {
  if (!world) return;
  if (!ensureLivingSelection()) {
    setStatus("no living cells — reset the world.", "error");
    return;
  }
  var src = programEl.value;
  try {
    world.setProgramText(selectedCell, src);
    setStatus("running on cell " + selectedCell + ".", "ok");
    markDemo(null);
  } catch (err) {
    setStatus(String(err.message || err), "error");
  }
}

function onReset() {
  seedWorld();
  lastSnapshot = JSON.parse(world.snapshot());
}

async function main() {
  envBadge();
  statusEl = document.getElementById("status");
  programEl = document.getElementById("program");
  cellSelect = document.getElementById("cell-select");
  consoleEl = document.getElementById("console");
  toggleBtn = document.getElementById("btn-toggle-console");
  canvas = document.getElementById("world");
  if (!canvas || !canvas.getContext) {
    setStatus("canvas unavailable", "error");
    return;
  }
  ctx = canvas.getContext("2d");
  setupCanvas();

  toggleBtn.addEventListener("click", function () {
    setConsoleOpen(consoleEl.hidden);
  });
  document.getElementById("btn-close-console").addEventListener("click", function () {
    setConsoleOpen(false);
  });
  document.addEventListener("keydown", function (ev) {
    if (ev.key === "Escape" && !consoleEl.hidden) {
      setConsoleOpen(false);
    }
  });

  setStatus("loading kernel…");
  await init();
  seedWorld();
  lastSnapshot = JSON.parse(world.snapshot());

  document.getElementById("btn-run").addEventListener("click", onRun);
  document.getElementById("btn-reset").addEventListener("click", onReset);
  cellSelect.addEventListener("change", function () {
    selectedCell = Number(cellSelect.value);
  });
  document.querySelectorAll(".demos button").forEach(function (btn) {
    btn.addEventListener("click", function () {
      var name = btn.getAttribute("data-demo");
      programEl.value = DEMOS[name];
      markDemo(name);
      onRun();
    });
  });

  window.addEventListener("resize", setupCanvas);
  requestAnimationFrame(frame);
}

main().catch(function (err) {
  console.error(err);
  var el = document.getElementById("status");
  if (el) {
    el.textContent = "kernel failed to load: " + (err && err.message ? err.message : err);
    el.className = "status error";
  }
  setConsoleOpen(true);
});
