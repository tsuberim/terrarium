import type { Page } from "@playwright/test";

export type E2eState = {
  ready: boolean;
  signedIn: boolean;
  studioOpen: boolean;
  deployCell: { x: number; y: number } | null;
  deployDialogOpen: boolean;
  credits: number | null;
  testing: boolean;
  wasmReady: boolean;
  allTestsPassed: boolean;
  playback: "idle" | "playing" | "paused";
  error: string | null;
  busy: boolean;
};

export async function getE2eState(page: Page): Promise<E2eState | null> {
  return page.evaluate(() => window.__TERRARIUM_E2E__?.getState() ?? null);
}

export async function waitForE2eReady(page: Page, timeoutMs = 30_000) {
  await page.waitForFunction(
    () => document.body.dataset.e2eReady === "true",
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForWasmReady(page: Page, timeoutMs = 45_000) {
  await page.waitForFunction(
    () => {
      const s = window.__TERRARIUM_E2E__?.getState();
      return !!s && !s.testing && s.wasmReady;
    },
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForPlayback(page: Page, playback: E2eState["playback"], timeoutMs = 10_000) {
  await page.waitForFunction(
    (expected) => window.__TERRARIUM_E2E__?.getState()?.playback === expected,
    playback,
    { timeout: timeoutMs },
  );
}

export async function waitForDeployCell(page: Page, timeoutMs = 5_000) {
  await page.waitForFunction(
    () => window.__TERRARIUM_E2E__?.getState()?.deployCell != null,
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForDeployDialog(page: Page, open: boolean, timeoutMs = 5_000) {
  await page.waitForFunction(
    (expected) => window.__TERRARIUM_E2E__?.getState()?.deployDialogOpen === expected,
    open,
    { timeout: timeoutMs },
  );
}
