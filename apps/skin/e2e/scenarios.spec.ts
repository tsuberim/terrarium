import { test } from "@playwright/test";

import { listScenarioIds, loadScenario, runScenario } from "./helpers/run-scenario";

for (const id of listScenarioIds()) {
  const scenario = loadScenario(id);

  test(`${id}: ${scenario.description}`, async ({ page }) => {
    await runScenario(page, id);
  });
}
