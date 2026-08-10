import { ReactNode } from "react";

export function PanelHeader({ title, actions }: { title: string; actions?: ReactNode }) {
  return (
    <div className="panel-header">
      <span>{title}</span>
      {actions ? <div className="panel-actions">{actions}</div> : null}
    </div>
  );
}
