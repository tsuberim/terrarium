import { useEffect, useRef } from "react";
import { e2eHooksEnabled } from "../lib/config";
import type { E2eBridge, E2eState } from "../lib/e2eBridge";

export function useE2eBridge(state: E2eState) {
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    if (!e2eHooksEnabled()) {
      delete window.__TERRARIUM_E2E__;
      delete document.body.dataset.e2eReady;
      return;
    }

    const bridge: E2eBridge = {
      getState: () => stateRef.current,
      waitFor: (predicate, timeoutMs = 30_000) =>
        new Promise((resolve, reject) => {
          const start = Date.now();
          const tick = () => {
            const current = stateRef.current;
            if (predicate(current)) {
              resolve(current);
              return;
            }
            if (Date.now() - start >= timeoutMs) {
              reject(new Error("E2E waitFor timeout"));
              return;
            }
            window.setTimeout(tick, 100);
          };
          tick();
        }),
    };

    window.__TERRARIUM_E2E__ = bridge;
    return () => {
      delete window.__TERRARIUM_E2E__;
    };
  }, []);

  useEffect(() => {
    if (!e2eHooksEnabled()) return;
    const s = stateRef.current;
    const ready =
      s.ready && s.signedIn && s.studioOpen && !s.testing && !s.busy;
    if (ready) document.body.dataset.e2eReady = "true";
    else delete document.body.dataset.e2eReady;
  }, [state]);
}
