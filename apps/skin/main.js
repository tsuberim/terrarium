import init, { JsWorld } from "./pkg/terrarium_kernel.js";

const DEMOS = {
  wander: `# wander — fixed thrust loop (deterministic)
thrust 120 40
sleep
thrust -90 110
sleep
thrust -70 -130
sleep
thrust 140 -30
sleep
jump 0
`,
  chase: `# chase — sense nearest body, thrust toward it
sense
jnz 0 4
sleep
jump 0
thrust_toward 160
sleep
jump 0
`,
  sit: `# sit — sleep is free
sleep
jump 0
`,
};

const CELL_HUES = [
  "120, 70%, 55%",
  "38, 70%, 58%",
  "176, 45%, 60%",
  "88, 55%, 52%",
  "150, 50%, 48%",
];

const TICKS_PER_FRAME = 1;
const MS_PER_TICK = 50;

let world = null;
let selectedCell = 0;
let lastSnapshot = null;
let running = true;
let lastTickAt = 0;
let statusEl;
let hudMass;
let hudBurned;
let hudTick;
let programEl;
let cellSelect;
let canvas;
let ctx;
let cssSize = 420;

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

function seedDish() {
  world = new JsWorld();
  var a = world.spawnCell(800, -28000, -12000);
  var b = world.spawnCell(600, 22000, 8000);
  var c = world.spawnCell(450, -5000, 28000);
  world.setProgramText(a, DEMOS.wander);
  world.setProgramText(b, DEMOS.chase);
  world.setProgramText(c, DEMOS.sit);
  // Inert crumb already in the box — absorb is an explicit verb.
  var dumper = world.spawnCell(100, 12000, -22000);
  world.dumpMatter(dumper, 60);
  world.setProgramText(dumper, DEMOS.sit);
  selectedCell = a;
  refreshCellSelect();
  programEl.value = DEMOS.wander;
  markDemo("wander");
  setStatus("dish seeded. wander is running on cell " + a + ".", "ok");
}

function refreshCellSelect() {
  var snap = JSON.parse(world.snapshot());
  lastSnapshot = snap;
  cellSelect.innerHTML = "";
  snap.cells.forEach(function (c) {
    var opt = document.createElement("option");
    opt.value = String(c.id);
    opt.textContent = "cell " + c.id + " · mass " + c.mass;
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

function resize() {
  var size = canvas.parentElement.clientWidth;
  var dpr = Math.min(window.devicePixelRatio || 1, 2);
  canvas.width = size * dpr;
  canvas.height = size * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  cssSize = size;
}

function bodyRadius(mass, radius) {
  var r = Math.sqrt(mass * 80);
  r = Math.max(1500, Math.min(r, 18000));
  return (r / radius) * (cssSize * 0.48);
}

function draw() {
  var n = cssSize;
  ctx.clearRect(0, 0, n, n);
  if (!lastSnapshot) return;

  var radius = lastSnapshot.radius || 100000;
  var scale = (n * 0.48) / radius;

  ctx.save();
  ctx.beginPath();
  ctx.arc(n / 2, n / 2, n * 0.48, 0, Math.PI * 2);
  ctx.clip();

  // inert dumps — dim amber crumbs
  lastSnapshot.inert.forEach(function (d) {
    var px = n / 2 + d.x * scale;
    var py = n / 2 + d.y * scale;
    var pr = Math.max(2.5, bodyRadius(d.mass, radius) * 0.7);
    ctx.globalCompositeOperation = "lighter";
    ctx.fillStyle = "hsla(38, 55%, 50%, 0.55)";
    ctx.beginPath();
    ctx.arc(px, py, pr, 0, Math.PI * 2);
    ctx.fill();
  });

  lastSnapshot.cells.forEach(function (c, i) {
    var hue = CELL_HUES[c.id % CELL_HUES.length];
    var px = n / 2 + c.x * scale;
    var py = n / 2 + c.y * scale;
    var pr = bodyRadius(c.mass, radius);
    var pulse = 1 + 0.04 * Math.sin(Date.now() / 900 + i);
    pr *= pulse;

    var g = ctx.createRadialGradient(px - pr * 0.2, py - pr * 0.2, pr * 0.1, px, py, pr);
    g.addColorStop(0, "hsla(" + hue + ", 0.9)");
    g.addColorStop(0.55, "hsla(" + hue + ", 0.3)");
    g.addColorStop(1, "hsla(" + hue + ", 0)");
    ctx.globalCompositeOperation = "lighter";
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.arc(px, py, pr, 0, Math.PI * 2);
    ctx.fill();

    if (c.id === selectedCell) {
      ctx.globalCompositeOperation = "source-over";
      ctx.strokeStyle = "rgba(213, 228, 218, 0.55)";
      ctx.lineWidth = 1.25;
      ctx.beginPath();
      ctx.arc(px, py, pr + 3, 0, Math.PI * 2);
      ctx.stroke();
    }
  });

  ctx.restore();
  ctx.globalCompositeOperation = "source-over";
}

function updateHud(snap) {
  hudMass.textContent = String(snap.total_mass);
  hudBurned.textContent = String(snap.house_burned);
  hudBurned.classList.toggle("burn", snap.house_burned > 0);
  hudTick.textContent = String(snap.tick);
}

function frame(now) {
  if (running && world && now - lastTickAt >= MS_PER_TICK) {
    lastTickAt = now;
    for (var i = 0; i < TICKS_PER_FRAME; i++) {
      world.tick();
    }
    lastSnapshot = JSON.parse(world.snapshot());
    updateHud(lastSnapshot);
    // Keep select labels fresh without stealing focus every frame.
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
      opts[i].textContent = "cell " + cell.id + " · mass " + cell.mass;
    }
  }
  // Drop dead cells from the list.
  var living = new Set(snap.cells.map(function (c) { return String(c.id); }));
  for (var j = opts.length - 1; j >= 0; j--) {
    if (!living.has(opts[j].value)) {
      opts[j].remove();
    }
  }
  // Add newly appeared cells (rare).
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
      opt.textContent = "cell " + c.id + " · mass " + c.mass;
      cellSelect.appendChild(opt);
    }
  });
}

function onRun() {
  if (!world) return;
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
  seedDish();
  lastSnapshot = JSON.parse(world.snapshot());
  updateHud(lastSnapshot);
}

async function main() {
  envBadge();
  statusEl = document.getElementById("status");
  hudMass = document.getElementById("hud-mass");
  hudBurned = document.getElementById("hud-burned");
  hudTick = document.getElementById("hud-tick");
  programEl = document.getElementById("program");
  cellSelect = document.getElementById("cell-select");
  canvas = document.getElementById("dish");
  if (!canvas || !canvas.getContext) {
    setStatus("canvas unavailable", "error");
    return;
  }
  ctx = canvas.getContext("2d");

  setStatus("loading kernel…");
  await init();
  seedDish();
  lastSnapshot = JSON.parse(world.snapshot());
  updateHud(lastSnapshot);

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

  window.addEventListener("resize", resize);
  resize();
  requestAnimationFrame(frame);
}

main().catch(function (err) {
  console.error(err);
  setStatus("kernel failed to load: " + (err && err.message ? err.message : err), "error");
});
