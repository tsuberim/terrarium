const SESSION_KEY = "terrarium_session";
const API_BASE = window.location.origin;

function session() {
  return localStorage.getItem(SESSION_KEY);
}

function setSession(token) {
  localStorage.setItem(SESSION_KEY, token);
}

async function api(path, options = {}) {
  const headers = { "Content-Type": "application/json", ...(options.headers || {}) };
  const tok = session();
  if (tok) headers.Authorization = `Bearer ${tok}`;
  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  const body = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(body.message || body.error || res.statusText);
  return body;
}

function show(id) {
  document.getElementById(id)?.classList.remove("hidden");
}

function hide(id) {
  document.getElementById(id)?.classList.add("hidden");
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
  const tok = session();
  if (!tok) {
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
  document.getElementById("account-id").textContent = `Account ${me.account_id}`;
  document.getElementById("env-label").textContent = `Environment: ${me.environment}`;

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
    li.innerHTML = `<span><strong>${t.label}</strong><br><span class="muted">${t.id.slice(0, 8)}…</span></span>`;
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

document.getElementById("create-account").addEventListener("click", async () => {
  const res = await api("/v1/accounts", { method: "POST" });
  setSession(res.session_token);
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
  const res = await api("/dashboard/api/tokens", {
    method: "POST",
    body: JSON.stringify({ label }),
  });
  const box = document.getElementById("new-token");
  box.classList.remove("hidden");
  box.textContent = `Copy now — shown once:\n${res.token}`;
  updateCurl(res.token);
  await refresh();
});

refresh().catch((err) => {
  console.error(err);
  localStorage.removeItem(SESSION_KEY);
  show("onboard");
});
