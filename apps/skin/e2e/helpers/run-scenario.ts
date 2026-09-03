import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { expect, type Page } from "@playwright/test";
import { parse } from "yaml";

import {
  getQaState,
  waitForDeployCell,
  waitForDeployDialog,
  waitForPlayback,
  waitForQaReady,
  waitForWasmReady,
  type QaState,
} from "./qa-bridge";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const SCENARIOS_DIR = path.join(REPO_ROOT, "docs/internal/qa/scenarios");

/** ~110 glims at GLIM_SCALE — minimum deploy with default extra */
const MIN_DEPLOY_CREDITS = 11_000_000;

type QaStateExpect = Partial<QaState> & { deployCell?: "not_null" | null };

type ScenarioStep =
  | { note: string }
  | { click: string }
  | { clickMap: Record<string, never> | { x?: number; y?: number } }
  | { faucet: boolean }
  | { assert: { qaState: QaStateExpect } }
  | { waitFor: { qaState: QaStateExpect; timeout?: number } };

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

function assertQaState(actual: QaState | null, expected: QaStateExpect) {
  expect(actual, "QA state missing").not.toBeNull();
  for (const [key, value] of Object.entries(expected)) {
    if (key === "deployCell") {
      if (value === "not_null") {
        expect(actual!.deployCell).not.toBeNull();
      } else {
        expect(actual!.deployCell).toBe(value);
      }
      continue;
    }
    expect(actual![key as keyof QaState]).toEqual(value);
  }
}

async function waitForQaStateMatch(page: Page, expected: QaStateExpect, timeoutMs: number) {
  await page.waitForFunction(
    ({ exp }) => {
      const state = window.__TERRARIUM_QA__?.getState();
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

async function runWaitFor(page: Page, expected: QaStateExpect, timeoutMs: number) {
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
  await waitForQaStateMatch(page, expected, timeoutMs);
}

async function maybeFaucet(page: Page) {
  let state = await getQaState(page);
  for (let i = 0; i < 3 && (state?.credits ?? 0) < MIN_DEPLOY_CREDITS; i++) {
    await page.getByTestId("qa-hud-faucet").click();
    await page.waitForTimeout(400);
    state = await getQaState(page);
  }
  expect((state?.credits ?? 0) >= MIN_DEPLOY_CREDITS).toBeTruthy();
}

export async function runScenario(page: Page, id: string, opts?: { skipGoto?: boolean }) {
  const scenario = loadScenario(id);

  if (!opts?.skipGoto) {
    await page.goto("/");
    await waitForQaReady(page);
  }

  for (const step of scenario.steps) {
    if ("note" in step) continue;

    if ("click" in step) {
      if (step.click === "qa-deploy-confirm") {
        await expect(page.getByTestId("qa-deploy-confirm")).toBeEnabled({ timeout: 5_000 });
      }
      await page.getByTestId(step.click).click();
      continue;
    }

    if ("clickMap" in step) {
      const map = page.getByTestId("qa-world-map");
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
      assertQaState(await getQaState(page), step.assert.qaState);
      continue;
    }

    if ("waitFor" in step) {
      await runWaitFor(page, step.waitFor.qaState, step.waitFor.timeout ?? 10_000);
    }
  }
}
