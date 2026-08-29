import { useCallback, useEffect, useState } from "react";
import {
  getServerPowerStatus,
  postServerPower,
  wakeServer,
  type ServerPowerStatus,
} from "../lib/api";
import { isLocalDev } from "../lib/config";

type Props = {
  signedIn: boolean;
  online: boolean;
  busy: boolean;
  onWakeComplete?: () => void;
};

export function ServerPowerControl({ signedIn, online, busy, onWakeComplete }: Props) {
  const [status, setStatus] = useState<ServerPowerStatus | null>(null);
  const [waking, setWaking] = useState(false);
  const [powerBusy, setPowerBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!signedIn) {
      setStatus(null);
      return;
    }
    try {
      setStatus(await getServerPowerStatus());
      setError(null);
    } catch {
      setStatus(null);
    }
  }, [signedIn]);

  useEffect(() => {
    void refresh();
  }, [refresh, online]);

  const wake = async () => {
    setWaking(true);
    setError(null);
    try {
      await wakeServer();
      onWakeComplete?.();
    } catch {
      setError("Wake timed out — try again in a minute");
    } finally {
      setWaking(false);
    }
  };

  const toggle = async (enabled: boolean) => {
    setPowerBusy(true);
    setError(null);
    try {
      await postServerPower(enabled);
      await refresh();
      if (enabled) onWakeComplete?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Power toggle failed");
    } finally {
      setPowerBusy(false);
    }
  };

  const showWake = !online && !waking && !isLocalDev();
  const showAdmin =
    !isLocalDev() &&
    signedIn &&
    status?.is_admin &&
    status.power_control_available &&
    online;

  if (!showWake && !showAdmin && !waking) return null;

  return (
    <div className="mt-1.5 border-t border-white/[0.05] pt-1.5">
      <div className="mb-1 text-[9px] uppercase tracking-wide text-white/25">Server</div>
        {waking ? (
          <span className="text-white/50">Waking… (cold start ~30s)</span>
        ) : showWake ? (
          <button
            type="button"
            className="hud-btn-sm hud-btn-accent"
            disabled={busy || waking}
            onClick={() => void wake()}
          >
            Wake server
          </button>
        ) : showAdmin ? (
          <div className="flex gap-1">
            <button
              type="button"
              className={`hud-btn-sm flex-1 ${status?.enabled ? "hud-segment-btn-active" : ""}`}
              disabled={powerBusy || status?.enabled === true}
              onClick={() => void toggle(true)}
            >
              On
            </button>
            <button
              type="button"
              className={`hud-btn-sm flex-1 ${status?.enabled === false ? "hud-segment-btn-active" : ""}`}
              disabled={powerBusy || status?.enabled === false}
              onClick={() => void toggle(false)}
            >
              Off
            </button>
          </div>
        ) : null}
        {error && <span className="text-red-400/75">{error}</span>}
        {showAdmin && status?.min_instances != null && (
          <span className="text-[9px] text-white/30">min {status.min_instances}</span>
        )}
    </div>
  );
}
