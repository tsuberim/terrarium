import { useMemo } from "react";
import type { TimestampedEvent } from "../lib/events";
import { eventTone, formatWorldEvent } from "../lib/events";

type Props = {
  events: TimestampedEvent[];
};

const TONE_CLASS: Record<ReturnType<typeof eventTone>, string> = {
  combat: "text-red-300/90",
  food: "text-amber-300/90",
  life: "text-teal-300/85",
  signal: "text-violet-300/85",
  neutral: "text-white/60",
};

export function EventFeed({ events }: Props) {
  const rows = useMemo(() => {
    const now = Date.now();
    return [...events]
      .reverse()
      .filter((e) => now - e.at < 12_000)
      .slice(0, 10);
  }, [events]);

  if (!rows.length) return null;

  return (
    <div className="pointer-events-none absolute bottom-3 left-3 z-10 max-w-[min(20rem,calc(100vw-1.5rem))] sm:bottom-4 sm:left-4">
      <div className="hud-panel flex flex-col gap-0.5 px-2.5 py-2">
        <div className="text-[10px] font-medium uppercase tracking-wider text-white/35">Live</div>
        {rows.map((event, i) => {
          const age = (Date.now() - event.at) / 12_000;
          const opacity = Math.max(0.35, 1 - age * 0.65);
          const key = `${event.type}-${event.at}-${i}`;
          return (
            <div
              key={key}
              className={`truncate text-[11px] leading-snug ${TONE_CLASS[eventTone(event)]}`}
              style={{ opacity }}
            >
              {formatWorldEvent(event)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
