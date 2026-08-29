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

export function assertConfig() {
  const missing = Object.entries(config.firebase)
    .filter(([, value]) => !value)
    .map(([key]) => key);
  if (missing.length) {
    throw new Error(`Missing Firebase config: ${missing.join(", ")}`);
  }
}
