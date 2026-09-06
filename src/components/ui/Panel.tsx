import type { ReactNode } from "react";

interface PanelProps {
  title?: ReactNode;
  subtitle?: ReactNode;
  badge?: ReactNode;
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
}

export function Panel({
  title,
  subtitle,
  badge,
  actions,
  className,
  children,
}: PanelProps) {
  return (
    <section className={`panel${className ? ` ${className}` : ""}`}>
      {(title || actions) && (
        <header className="panel__head">
          <div className="panel__titles">
            {title && (
              <h2 className="panel__title">
                {title}
                {badge && <span className="badge">{badge}</span>}
              </h2>
            )}
            {subtitle && <p className="panel__subtitle">{subtitle}</p>}
          </div>
          {actions && <div className="panel__actions">{actions}</div>}
        </header>
      )}
      <div className="panel__body">{children}</div>
    </section>
  );
}

interface FieldProps {
  label: ReactNode;
  hint?: ReactNode;
  htmlFor?: string;
  children: ReactNode;
}

/** A labelled form row. */
export function Field({ label, hint, htmlFor, children }: FieldProps) {
  return (
    <div className="field">
      <label className="field__label" htmlFor={htmlFor}>
        {label}
      </label>
      <div className="field__control">{children}</div>
      {hint && <p className="field__hint">{hint}</p>}
    </div>
  );
}
