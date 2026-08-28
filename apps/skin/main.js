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
const VOID_BG = "#020403";
const RING_COLOR = "#1a2e24";
const SELECT_COLOR = "#e8f0ea";

/** Short-axis pixel count; long axis follows viewport aspect (square pixels). */
const PIXEL_SHORT = 160;

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

let socket = null;
let selectedCell = 0;
let lastSnapshot = null;
let statusEl;
let programEl;
let cellSelect;
let consoleEl;
let toggleBtn;
let canvas;
let ctx;
let bufW = PIXEL_SHORT;
let bufH = PIXEL_SHORT;
let reconnectTimer = null;

function setStatus(msg, kind) {
  statusEl.textContent = msg || "";
  statusEl.className = "status" + (kind ? " " + kind : "");
}

function envBadge(env) {
  var value = (env || "staging").trim() || "staging";
  var badge = document.getElementById("env-badge");
  if (badge) {
    badge.textContent = value;
    badge.setAttribute("data-env", value);
  }
  document.documentElement.setAttribute("data-env", value);
  var meta = document.querySelector('meta[name="terrarium-env"]');
  if (meta) meta.setAttribute("content", value);
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

function cellLabel(id) {
  return "cell " + id;
}

function applySnapshot(snap) {
  lastSnapshot = snap;
  if (!cellSelect.options.length) {
    refreshCellSelect(snap);
  } else {
    syncSelectLabels(snap);
  }
}

function refreshCellSelect(snap) {
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

function worldRadiusPx() {
  return Math.round(Math.min(bufW, bufH) * 0.46);
}

function bodyRadiusPx(mass, worldRadius) {
  var r = Math.sqrt(mass * 2000);
  r = Math.max(5000, Math.min(r, 24000));
  return Math.max(3, Math.round((r / worldRadius) * worldRadiusPx() * 1.15));
}

function fillCircle(x, y, r, color) {
  var cx = Math.round(x);
  var cy = Math.round(y);
  var rr = Math.max(1, Math.round(r));
  ctx.fillStyle = color;
  for (var dy = -rr; dy <= rr; dy++) {
    for (var dx = -rr; dx <= rr; dx++) {
      if (dx * dx + dy * dy <= rr * rr) {
        ctx.fillRect(cx + dx, cy + dy, 1, 1);
      }
    }
  }
}

function strokeCircle(x, y, r, color) {
  var cx = Math.round(x);
  var cy = Math.round(y);
  var rr = Math.max(1, Math.round(r));
  var inner = (rr - 1) * (rr - 1);
  var outer = rr * rr;
  ctx.fillStyle = color;
  for (var dy = -rr; dy <= rr; dy++) {
    for (var dx = -rr; dx <= rr; dx++) {
      var d2 = dx * dx + dy * dy;
      if (d2 <= outer && d2 >= inner) {
        ctx.fillRect(cx + dx, cy + dy, 1, 1);
      }
    }
  }
}

function draw() {
  ctx.fillStyle = VOID_BG;
  ctx.fillRect(0, 0, bufW, bufH);
  if (!lastSnapshot) return;

  var radius = lastSnapshot.radius || 100000;
  var ringR = worldRadiusPx();
  var scale = ringR / radius;
  var cx = bufW / 2;
  var cy = bufH / 2;

  fillCircle(cx, cy, ringR, WORLD_BG);
  strokeCircle(cx, cy, ringR, RING_COLOR);

  lastSnapshot.inert.forEach(function (d) {
    var px = cx + d.x * scale;
    var py = cy + d.y * scale;
    var pr = Math.max(1, Math.round(bodyRadiusPx(d.mass, radius) * 0.65));
    fillCircle(px, py, pr, INERT_COLOR);
  });

  lastSnapshot.cells.forEach(function (c) {
    var color = CELL_COLORS[c.id % CELL_COLORS.length];
    var px = cx + c.x * scale;
    var py = cy + c.y * scale;
    var pr = bodyRadiusPx(c.mass, radius);
    fillCircle(px, py, pr, color);
    if (c.id === selectedCell) {
      strokeCircle(px, py, pr + 2, SELECT_COLOR);
    }
  });
}

function frame() {
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

function sendJson(obj) {
  if (!socket || socket.readyState !== WebSocket.OPEN) {
    setStatus("not connected to host.", "error");
    return false;
  }
  socket.send(JSON.stringify(obj));
  return true;
}

function onRun() {
  if (!ensureLivingSelection()) {
    setStatus("no living cells — reset the world.", "error");
    return;
  }
  if (sendJson({ type: "set_program", cell_id: selectedCell, source: programEl.value })) {
    markDemo(null);
  }
}

function onReset() {
  sendJson({ type: "reset" });
}

function wsUrl() {
  var proto = location.protocol === "https:" ? "wss:" : "ws:";
  return proto + "//" + location.host + "/ws";
}

function connect() {
  setStatus("connecting to host…");
  var ws = new WebSocket(wsUrl());
  socket = ws;

  ws.onopen = function () {
    setStatus("connected. camera only — host owns the tick.", "ok");
  };

  ws.onmessage = function (ev) {
    var msg;
    try {
      msg = JSON.parse(ev.data);
    } catch (err) {
      setStatus("bad host message", "error");
      return;
    }
    if (msg.type === "hello") {
      envBadge(msg.env);
      return;
    }
    if (msg.type === "snapshot") {
      applySnapshot(msg.world);
      return;
    }
    if (msg.type === "ok") {
      setStatus(msg.message || "ok", "ok");
      return;
    }
    if (msg.type === "error") {
      setStatus(msg.message || "error", "error");
      setConsoleOpen(true);
    }
  };

  ws.onclose = function () {
    setStatus("host disconnected — retrying…", "error");
    socket = null;
    if (reconnectTimer) clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(connect, 1500);
  };

  ws.onerror = function () {
    ws.close();
  };
}

function main() {
  var meta = document.querySelector('meta[name="terrarium-env"]');
  envBadge((meta && meta.getAttribute("content")) || "staging");

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

  programEl.value = DEMOS.wander;
  markDemo("wander");

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
  connect();
  requestAnimationFrame(frame);
}

main();
