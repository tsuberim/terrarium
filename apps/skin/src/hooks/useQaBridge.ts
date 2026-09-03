import { useEffect, useRef } from "react";
import { qaMode } from "../lib/config";
import type { QaBridge, QaState } from "../lib/qaBridge";

export function useQaBridge(state: QaState) {
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    if (!qaMode()) {
      delete window.__TERRARIUM_QA__;
      delete document.body.dataset.qaReady;
      return;
    }

    const bridge: QaBridge = {
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
              reject(new Error("QA waitFor timeout"));
              return;
            }
            window.setTimeout(tick, 100);
          };
          tick();
        }),
    };

    window.__TERRARIUM_QA__ = bridge;
    return () => {
      delete window.__TERRARIUM_QA__;
    };
  }, []);

  useEffect(() => {
    if (!qaMode()) return;
    const s = stateRef.current;
    const ready =
      s.ready && s.signedIn && s.studioOpen && !s.testing && !s.busy;
    if (ready) document.body.dataset.qaReady = "true";
    else delete document.body.dataset.qaReady;
  }, [state]);
}
