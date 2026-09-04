import { formatGlimValue, glimCount, GLIM_NAME, GLIM_NAME_PLURAL, GLIM_VALUE_WIDTH_CH } from "../lib/glim";

type IconProps = {
  className?: string;
};

export function GlimIcon({ className = "h-2.5 w-2.5 shrink-0 text-biolume/85" }: IconProps) {
  return (
    <svg viewBox="0 0 8 8" className={className} aria-hidden fill="currentColor">
      <path d="M4 0.5 7.5 4 4 7.5.5 4Z" />
    </svg>
  );
}

type Props = {
  amount: number;
  className?: string;
  iconClassName?: string;
  /** Optional label before the icon, e.g. "Energy". */
  label?: string;
  /** Text before the amount, e.g. "+". */
  prefix?: string;
  /** Hide the unit word — icon + number only. */
  compact?: boolean;
};

export function GlimAmount({ amount, className, iconClassName, label, prefix, compact }: Props) {
  const unit = Math.abs(glimCount(amount)) === 1 ? GLIM_NAME : GLIM_NAME_PLURAL;
  return (
    <span className={`inline-flex items-center gap-0.5 tabular-nums ${className ?? ""}`}>
      {label && <span className="shrink-0">{label}</span>}
      <GlimIcon className={iconClassName} />
      <span
        className="inline-block text-right"
        style={{ minWidth: `${GLIM_VALUE_WIDTH_CH}ch` }}
      >
        {prefix}
        {formatGlimValue(amount)}
        {!compact && `\u00a0${unit}`}
      </span>
    </span>
  );
}
