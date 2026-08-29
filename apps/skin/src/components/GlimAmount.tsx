import { formatGlimValue, glimCount, GLIM_NAME, GLIM_NAME_PLURAL } from "../lib/glim";

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
  /** Hide the unit word — icon + number only. */
  compact?: boolean;
};

export function GlimAmount({ amount, className, iconClassName, compact }: Props) {
  const unit = Math.abs(glimCount(amount)) === 1 ? GLIM_NAME : GLIM_NAME_PLURAL;
  return (
    <span className={`inline-flex items-center gap-0.5 tabular-nums ${className ?? ""}`}>
      <GlimIcon className={iconClassName} />
      <span>
        {formatGlimValue(amount)}
        {!compact && `\u00a0${unit}`}
      </span>
    </span>
  );
}
