(function () {
  "use strict";

  var meta = document.querySelector('meta[name="terrarium-env"]');
  var env = ((meta && meta.getAttribute("content")) || "staging").trim() || "staging";
  var badge = document.getElementById("env-badge");
  if (badge) {
    badge.textContent = env;
    badge.setAttribute("data-env", env);
  }
  document.documentElement.setAttribute("data-env", env);

  var canvas = document.getElementById("dish");
  if (!canvas || !canvas.getContext) return;
  if (window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    return;
  }

  var ctx = canvas.getContext("2d");
  var blobs = [
    { x: 0.38, y: 0.42, r: 0.09, vx: 0.00021, vy: -0.00013, hue: "120, 70%, 55%" },
    { x: 0.58, y: 0.50, r: 0.07, vx: -0.00017, vy: 0.00019, hue: "38, 70%, 58%" },
    { x: 0.47, y: 0.62, r: 0.055, vx: 0.00012, vy: 0.00011, hue: "176, 45%, 60%" },
    { x: 0.33, y: 0.55, r: 0.04, vx: 0.00009, vy: -0.00018, hue: "150, 50%, 48%" },
    { x: 0.62, y: 0.38, r: 0.035, vx: -0.00014, vy: 0.00008, hue: "88, 55%, 52%" }
  ];

  function resize() {
    var size = canvas.parentElement.clientWidth;
    var dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = size * dpr;
    canvas.height = size * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    canvas._css = size;
  }

  function step() {
    var n = canvas._css;
    ctx.clearRect(0, 0, n, n);
    ctx.save();
    ctx.beginPath();
    ctx.arc(n / 2, n / 2, n * 0.48, 0, Math.PI * 2);
    ctx.clip();

    for (var i = 0; i < blobs.length; i++) {
      var b = blobs[i];
      b.x += b.vx;
      b.y += b.vy;

      var dx = b.x - 0.5;
      var dy = b.y - 0.5;
      var dist = Math.sqrt(dx * dx + dy * dy);
      var max = 0.5 - b.r - 0.08;
      if (dist > max) {
        b.x = 0.5 + (dx / dist) * max;
        b.y = 0.5 + (dy / dist) * max;
        b.vx *= -0.7;
        b.vy *= -0.7;
      }

      b.vx += (Math.random() - 0.5) * 0.00002;
      b.vy += (Math.random() - 0.5) * 0.00002;
      b.vx *= 0.999;
      b.vy *= 0.999;

      var pulse = 1 + 0.06 * Math.sin(Date.now() / 900 + i);
      var px = b.x * n;
      var py = b.y * n;
      var pr = b.r * n * pulse;
      var g = ctx.createRadialGradient(px - pr * 0.2, py - pr * 0.2, pr * 0.1, px, py, pr);
      g.addColorStop(0, "hsla(" + b.hue + ", 0.85)");
      g.addColorStop(0.55, "hsla(" + b.hue + ", 0.28)");
      g.addColorStop(1, "hsla(" + b.hue + ", 0)");
      ctx.globalCompositeOperation = "lighter";
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(px, py, pr, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.restore();
    ctx.globalCompositeOperation = "source-over";
    requestAnimationFrame(step);
  }

  window.addEventListener("resize", resize);
  resize();
  requestAnimationFrame(step);
})();
