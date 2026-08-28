const SESSION_KEY = "terrarium_dev_session";
const API_BASE = window.location.origin.includes("127.0.0.1") || window.location.origin.includes("localhost")
  ? "http://127.0.0.1:3000"
  : window.location.origin.replace(/\/dashboard\/?$/, "");

let firebaseAuth = null;
let firebaseReady = false;
let devSession = localStorage.getItem(SESSION_KEY);
let runtimeConfig = null;

function show(id) {
  document.getElementById(id)?.classList.remove("hidden");
}

function hide(id) {
  document.getElementById(id)?.classList.add("hidden");
}

async function api(path, options = {}) {
  const headers = { "Content-Type": "application/json", ...(options.headers || {}) };
  const tok = await authToken();
  if (tok) headers.Authorization = `Bearer ${tok}`;
  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  const body = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(body.message || body.error || res.statusText);
  return body;
}

async function authToken() {
  if (firebaseAuth?.currentUser) {
    return firebaseAuth.currentUser.getIdToken();
  }
  return devSession;
}

async function loadConfig() {
  runtimeConfig = await fetch(`${API_BASE}/dashboard/api/config`).then((r) => r.json());
  document.getElementById("env-label").textContent = `Environment: ${runtimeConfig.environment}`;
  return runtimeConfig;
}

async function initFirebase(cfg) {
  if (!cfg.firebase) return;
  const { initializeApp } = await import("https://www.gstatic.com/firebasejs/11.0.2/firebase-app.js");
  const { getAuth, GoogleAuthProvider, signInWithPopup, signOut, onAuthStateChanged } =
    await import("https://www.gstatic.com/firebasejs/11.0.2/firebase-auth.js");
  const app = initializeApp({
    apiKey: cfg.firebase.api_key,
    authDomain: cfg.firebase.auth_domain,
    projectId: cfg.firebase.project_id,
  });
  firebaseAuth = getAuth(app);
  firebaseReady = true;
  show("firebase-signin");
  document.getElementById("onboard-copy").textContent =
    "Sign in with Firebase to manage credits and API tokens.";

  document.getElementById("google-signin").onclick = async () => {
    await signInWithPopup(firebaseAuth, new GoogleAuthProvider());
  };
  document.getElementById("sign-out").onclick = async () => {
    await signOut(firebaseAuth);
    devSession = null;
    localStorage.removeItem(SESSION_KEY);
    await refresh();
  };

  onAuthStateChanged(firebaseAuth, async (user) => {
    if (user) {
      hide("google-signin");
      show("sign-out");
    } else {
      show("google-signin");
      hide("sign-out");
    }
    await refresh();
  });
}

function updateCurl(apiToken) {
  const el = document.getElementById("curl-example");
  const token = apiToken || "<your-api-token>";
  el.textContent = `curl -s -X POST ${API_BASE}/v1/spawn \\
  -H "Authorization: Bearer ${token}" \\
  -H "Content-Type: application/json" \\
  -d '{"mass":100,"x":0,"y":0,"program":"sleep\\njump 0"}'`;
}

async function refresh() {
  const hasFirebaseUser = firebaseAuth?.currentUser;
  const hasDev = !!devSession;
  if (!hasFirebaseUser && !hasDev) {
    show("onboard");
    hide("account");
    hide("faucet-section");
    hide("billing-section");
    hide("tokens-section");
    updateCurl();
    return;
  }

  hide("onboard");
  show("account");
  show("billing-section");
  show("tokens-section");

  const me = await api("/dashboard/api/me");
  document.getElementById("credits").textContent = me.credits.toLocaleString();
  document.getElementById("account-id").textContent = `Account ${me.account_id} (${me.auth_mode})`;

  if (me.free_mint_enabled) {
    show("faucet-section");
  } else {
    hide("faucet-section");
  }

  const tokens = await api("/dashboard/api/tokens");
  const list = document.getElementById("token-list");
  list.innerHTML = "";
  for (const t of tokens) {
    const li = document.createElement("li");
    if (t.revoked_at) li.classList.add("revoked");
    li.innerHTML = `<span><strong>${t.label}</strong> <code>${t.scopes}</code><br><span class="muted">${t.id.slice(0, 8)}…</span></span>`;
    if (!t.revoked_at) {
      const btn = document.createElement("button");
      btn.textContent = "Revoke";
      btn.className = "danger";
      btn.type = "button";
      btn.onclick = async () => {
        await api(`/dashboard/api/tokens/${t.id}`, { method: "DELETE" });
        await refresh();
      };
      li.appendChild(btn);
    } else {
      const span = document.createElement("span");
      span.className = "muted";
      span.textContent = "revoked";
      li.appendChild(span);
    }
    list.appendChild(li);
  }
  updateCurl();
}

document.getElementById("dev-signin").addEventListener("click", async () => {
  const res = await fetch(`${API_BASE}/v1/accounts`, { method: "POST" });
  const body = await res.json();
  if (!res.ok) throw new Error(body.message || body.error);
  devSession = body.session_token;
  localStorage.setItem(SESSION_KEY, devSession);
  await refresh();
});

document.getElementById("faucet").addEventListener("click", async () => {
  const res = await api("/dashboard/api/faucet", { method: "POST", body: "{}" });
  document.getElementById("credits").textContent = res.credits.toLocaleString();
});

document.getElementById("checkout").addEventListener("click", async () => {
  const res = await api("/dashboard/api/billing/checkout", { method: "POST", body: "{}" });
  document.getElementById("billing-notice").textContent = res.message;
});

document.getElementById("mint-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  const label = document.getElementById("token-label").value.trim() || "default";
  const scopes = [];
  if (document.getElementById("scope-spawn").checked) scopes.push("spawn");
  if (document.getElementById("scope-read").checked) scopes.push("read");
  const res = await api("/dashboard/api/tokens", {
    method: "POST",
    body: JSON.stringify({ label, scopes }),
  });
  const box = document.getElementById("new-token");
  box.classList.remove("hidden");
  box.textContent = `Copy now — shown once:\n${res.token}\nscopes: ${res.scopes}`;
  updateCurl(res.token);
  await refresh();
});

try {
  const cfg = await loadConfig();
  if (cfg.dev_auth_enabled) {
    show("dev-signin");
    document.getElementById("onboard-copy").textContent =
      "Local dev: use dev sign-in, or configure Firebase for production-like auth.";
  }
  await initFirebase(cfg);
  if (!firebaseReady) {
    show("onboard");
    if (cfg.dev_auth_enabled) show("dev-signin");
  }
  await refresh();
} catch (err) {
  console.error(err);
  show("onboard");
  document.getElementById("onboard-copy").textContent = `Could not reach API at ${API_BASE}`;
}
