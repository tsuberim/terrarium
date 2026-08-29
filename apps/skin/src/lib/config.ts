export const config = {
  apiBase: import.meta.env.VITE_API_BASE ?? "",
  firebase: {
    apiKey: import.meta.env.VITE_FIREBASE_API_KEY,
    authDomain: import.meta.env.VITE_FIREBASE_AUTH_DOMAIN,
    projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID,
    appId: import.meta.env.VITE_FIREBASE_APP_ID,
  },
} as const;

export function apiRoot() {
  return `${config.apiBase}/api`;
}

export function wsRoot() {
  // Firebase Hosting rewrites /api for HTTP but cannot proxy WebSocket upgrades;
  // prod builds set VITE_WS_BASE to the Cloud Run URL (see generate-config.sh).
  const override = import.meta.env.VITE_WS_BASE as string | undefined;
  if (override) return override.replace(/\/$/, "");

  // Dev: connect straight to the API — Vite's WS proxy is flaky, and avoids
  // routing through :5173 for upgrades.
  if (import.meta.env.DEV && !config.apiBase) {
    const port = import.meta.env.VITE_API_PORT ?? "8080";
    return `ws://127.0.0.1:${port}/api`;
  }

  const base = config.apiBase || window.location.origin;
  return base.replace(/^http/, "ws") + "/api";
}

export function assertConfig() {
  const missing = Object.entries(config.firebase)
    .filter(([, value]) => !value)
    .map(([key]) => key);
  if (missing.length) {
    throw new Error(`Missing Firebase config: ${missing.join(", ")}`);
  }
}
