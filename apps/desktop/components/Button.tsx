import { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  icon?: ReactNode;
  variant?: "default" | "primary" | "ghost" | "danger";
};

export function Button({ icon, children, variant = "default", className = "", ...props }: ButtonProps) {
  return (
    <button className={`bf-button bf-button-${variant} ${className}`} type="button" {...props}>
      {icon ? <span className="bf-button-icon">{icon}</span> : null}
      {children ? <span>{children}</span> : null}
    </button>
  );
}
