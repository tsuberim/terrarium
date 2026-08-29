import { motion } from "framer-motion";

type Props = {
  view: "god" | "follow";
  onViewChange: (view: "god" | "follow") => void;
  accountLabel: string;
  worldNote: string;
};

export function Sidebar({ view, onViewChange, accountLabel, worldNote }: Props) {
  return (
    <motion.aside
      className="glass-panel flex flex-col gap-6 border-r border-line p-5 md:min-h-[calc(100vh-3.5rem)]"
      initial={{ opacity: 0, x: -8 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay: 0.1 }}
    >
      <section>
        <h2 className="label mb-3">View</h2>
        <div className="space-y-2">
          <label className="flex cursor-pointer items-center gap-2 text-sm text-mist">
            <input
              type="radio"
              name="view"
              checked={view === "god"}
              onChange={() => onViewChange("god")}
              className="accent-biolume"
            />
            God view
          </label>
          <label className="flex cursor-not-allowed items-center gap-2 text-sm text-fog/60">
            <input type="radio" name="view" disabled className="accent-biolume" />
            Follow creature
            <span className="text-[0.65rem] uppercase tracking-wider text-fog/50">soon</span>
          </label>
        </div>
      </section>

      <section>
        <h2 className="label mb-3">Account</h2>
        <p className="text-sm leading-relaxed text-mist">{accountLabel}</p>
      </section>

      <section>
        <h2 className="label mb-3">World</h2>
        <p className="text-sm leading-relaxed text-fog">{worldNote}</p>
      </section>

      <section className="mt-auto hidden md:block">
        <p className="font-display text-2xl leading-tight text-white/90">
          A living simulation.
          <span className="block text-base italic text-fog">Deploy. Watch. Survive.</span>
        </p>
      </section>
    </motion.aside>
  );
}
