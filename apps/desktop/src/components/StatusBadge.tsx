import {
  AlertTriangle,
  Ban,
  CheckCircle2,
  CircleHelp,
  Clock,
  CloudOff,
  MinusCircle,
} from "lucide-react";

export type StatusBadgeKind =
  | "ok"
  | "warn"
  | "offline"
  | "denied"
  | "stale"
  | "unavailable"
  | "error";

interface StatusBadgeProps {
  kind: StatusBadgeKind;
  label: string;
  title?: string;
  className?: string;
}

const ICONS: Record<StatusBadgeKind, typeof CheckCircle2> = {
  ok: CheckCircle2,
  warn: AlertTriangle,
  offline: CloudOff,
  denied: Ban,
  stale: Clock,
  unavailable: MinusCircle,
  error: AlertTriangle,
};

const KIND_CLASS: Record<StatusBadgeKind, string> = {
  ok: "badge badge-ok",
  warn: "badge badge-warn",
  offline: "badge badge-warn",
  denied: "badge badge-warn",
  stale: "badge badge-warn",
  unavailable: "badge badge-muted",
  error: "badge badge-warn",
};

/**
 * Non-color-only status chip (U7): lucide icon + text.
 * lucide-react pin is 0.468.0 — do not bump from here.
 */
export function StatusBadge({
  kind,
  label,
  title,
  className,
}: StatusBadgeProps) {
  const Icon = ICONS[kind] ?? CircleHelp;
  const base = KIND_CLASS[kind] ?? "badge badge-muted";
  return (
    <span
      className={className ? `${base} ${className}` : base}
      title={title}
      data-status={kind}
    >
      <Icon size={12} aria-hidden className="status-badge-icon" />
      <span>{label}</span>
    </span>
  );
}
