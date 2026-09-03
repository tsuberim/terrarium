import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { expect, type Page } from "@playwright/test";
import { parse } from "yaml";

import {
  getE2eState,
  waitForDeployCell,
  waitForDeployDialog,
  waitForPlayback,
  waitForE2eReady,
  waitForWasmReady,
  type E2eState,
} from "./e2e-bridge";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const SCENARIOS_DIR = path.join(REPO_ROOT, "docs/internal/qa/scenarios");

/** ~110 glims at GLIM_SCALE — minimum deploy with default extra */
const MIN_DEPLOY_CREDITS = 11_000_000;

type E2eStateExpect = Partial<E2eState> & { deployCell?: "not_null" | null };

type ScenarioStep =
  | { note: string }
  | { click: string }
  | { clickMap: Record<string, never> | { x?: number; y?: number } }
  | { faucet: boolean }
  | { assert: { e2eState: E2eStateExpect } }
  | { waitFor: { e2eState: E2eStateExpect; timeout?: number } };

export type Scenario = {
  id: string;
  description: string;
  requires?: string[];
  steps: ScenarioStep[];
};

export function loadScenario(id: string): Scenario {
  const file = path.join(SCENARIOS_DIR, `${id}.yaml`);
  const raw = parse(fs.readFileSync(file, "utf8")) as Scenario;
  if (raw.id !== id) {
    throw new Error(`Scenario id mismatch in ${file}: ${raw.id} !== ${id}`);
  }
  return raw;
}

export function listScenarioIds(): string[] {
  return fs
    .readdirSync(SCENARIOS_DIR)
    .filter((name) => name.endsWith(".yaml"))
    .map((name) => name.replace(/\.yaml$/, ""))
    .sort();
}

function assertE2eState(actual: E2eState | null, expected: E2eStateExpect) {
  expect(actual, "E2E state missing").not.toBeNull();
  for (const [key, value] of Object.entries(expected)) {
    if (key === "deployCell") {
      if (value === "not_null") {
        expect(actual!.deployCell).not.toBeNull();
      } else {
        expect(actual!.deployCell).toBe(value);
      }
      continue;
    }
    expect(actual![key as keyof E2eState]).toEqual(value);
  }
}

async function waitForE2eStateMatch(page: Page, expected: E2eStateExpect, timeoutMs: number) {
  await page.waitForFunction(
    ({ exp }) => {
      const state = window.__TERRARIUM_E2E__?.getState();
      if (!state) return false;
      for (const [key, value] of Object.entries(exp)) {
        if (key === "deployCell") {
          if (value === "not_null") {
            if (state.deployCell == null) return false;
          } else if (state.deployCell !== value) {
            return false;
          }
          continue;
        }
        if ((state as Record<string, unknown>)[key] !== value) return false;
      }
      return true;
    },
    { exp: expected },
    { timeout: timeoutMs },
  );
}

async function runWaitFor(page: Page, expected: E2eStateExpect, timeoutMs: number) {
  if (expected.testing === false && expected.wasmReady === true) {
    await waitForWasmReady(page, timeoutMs);
    return;
  }
  if (expected.playback === "playing" || expected.playback === "paused" || expected.playback === "idle") {
    await waitForPlayback(page, expected.playback, timeoutMs);
    return;
  }
  if (expected.deployDialogOpen === false) {
    await waitForDeployDialog(page, false, timeoutMs);
    return;
  }
  if (expected.deployCell === "not_null") {
    await waitForDeployCell(page, timeoutMs);
    return;
  }
  await waitForE2eStateMatch(page, expected, timeoutMs);
}

async function maybeFaucet(page: Page) {
  let state = await getE2eState(page);
  for (let i = 0; i < 3 && (state?.credits ?? 0) < MIN_DEPLOY_CREDITS; i++) {
    await page.getByTestId("e2e-hud-faucet").click();
    await page.waitForTimeout(400);
    state = await getE2eState(page);
  }
  expect((state?.credits ?? 0) >= MIN_DEPLOY_CREDITS).toBeTruthy();
}

export async function runScenario(page: Page, id: string, opts?: { skipGoto?: boolean }) {
  const scenario = loadScenario(id);

  if (!opts?.skipGoto) {
    await page.goto("/");
    await waitForE2eReady(page);
  }

  for (const step of scenario.steps) {
    if ("note" in step) continue;

    if ("click" in step) {
      if (step.click === "e2e-deploy-confirm") {
        await expect(page.getByTestId("e2e-deploy-confirm")).toBeEnabled({ timeout: 5_000 });
      }
      await page.getByTestId(step.click).click();
      continue;
    }

    if ("clickMap" in step) {
      const map = page.getByTestId("e2e-world-map");
      const box = await map.boundingBox();
      expect(box).toBeTruthy();
      const relX = step.clickMap?.x ?? 0.75;
      const relY = step.clickMap?.y ?? 0.5;
      await page.mouse.click(box!.x + box!.width * relX, box!.y + box!.height * relY);
      continue;
    }

    if ("faucet" in step) {
      if (step.faucet) await maybeFaucet(page);
      continue;
    }

    if ("assert" in step) {
      assertE2eState(await getE2eState(page), step.assert.e2eState);
      continue;
    }

    if ("waitFor" in step) {
      await runWaitFor(page, step.waitFor.e2eState, step.waitFor.timeout ?? 10_000);
    }
  }
}
