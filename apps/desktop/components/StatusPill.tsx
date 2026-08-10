export function StatusPill({ label, tone = "default" }: { label: string; tone?: "default" | "success" | "warning" | "risk" }) {
  return <span className={`status-pill status-pill-${tone}`}>{label}</span>;
}
