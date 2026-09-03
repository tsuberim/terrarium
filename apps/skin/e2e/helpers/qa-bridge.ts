import type { Page } from "@playwright/test";

export type QaState = {
  ready: boolean;
  signedIn: boolean;
  studioOpen: boolean;
  deployCell: { x: number; y: number } | null;
  deployDialogOpen: boolean;
  credits: number | null;
  testing: boolean;
  wasmReady: boolean;
  playback: "idle" | "playing" | "paused";
  error: string | null;
  busy: boolean;
};

export async function getQaState(page: Page): Promise<QaState | null> {
  return page.evaluate(() => window.__TERRARIUM_QA__?.getState() ?? null);
}

export async function waitForQaReady(page: Page, timeoutMs = 30_000) {
  await page.waitForFunction(
    () => document.body.dataset.qaReady === "true",
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForWasmReady(page: Page, timeoutMs = 45_000) {
  await page.waitForFunction(
    () => {
      const s = window.__TERRARIUM_QA__?.getState();
      return !!s && !s.testing && s.wasmReady;
    },
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForPlayback(page: Page, playback: QaState["playback"], timeoutMs = 10_000) {
  await page.waitForFunction(
    (expected) => window.__TERRARIUM_QA__?.getState()?.playback === expected,
    playback,
    { timeout: timeoutMs },
  );
}

export async function waitForDeployCell(page: Page, timeoutMs = 5_000) {
  await page.waitForFunction(
    () => window.__TERRARIUM_QA__?.getState()?.deployCell != null,
    undefined,
    { timeout: timeoutMs },
  );
}

export async function waitForDeployDialog(page: Page, open: boolean, timeoutMs = 5_000) {
  await page.waitForFunction(
    (expected) => window.__TERRARIUM_QA__?.getState()?.deployDialogOpen === expected,
    open,
    { timeout: timeoutMs },
  );
}
