interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
  id?: string;
}

export function Toggle({ checked, onChange, label, disabled, id }: ToggleProps) {
  return (
    <label className={`toggle${disabled ? " toggle--disabled" : ""}`}>
      <input
        id={id}
        type="checkbox"
        className="toggle__input"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="toggle__track" aria-hidden="true">
        <span className="toggle__thumb" />
      </span>
      {label && <span className="toggle__label">{label}</span>}
    </label>
  );
}
