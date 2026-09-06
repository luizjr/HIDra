import { useEffect, useRef } from "react";
import type { CSSProperties } from "react";

interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  /** Fired on release or after a 250ms debounce — use this for backend writes. */
  onCommit?: (value: number) => void;
  format?: (value: number) => string;
  disabled?: boolean;
  id?: string;
}

export function Slider({
  value,
  min,
  max,
  step = 1,
  onChange,
  onCommit,
  format,
  disabled,
  id,
}: SliderProps) {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latest = useRef(value);
  latest.current = value;

  useEffect(
    () => () => {
      if (timer.current !== null) clearTimeout(timer.current);
    },
    [],
  );

  const commitNow = () => {
    if (timer.current !== null) {
      clearTimeout(timer.current);
      timer.current = null;
    }
    onCommit?.(latest.current);
  };

  const scheduleCommit = () => {
    if (!onCommit) return;
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      onCommit(latest.current);
    }, 250);
  };

  const pct = max > min ? ((value - min) / (max - min)) * 100 : 0;
  const display = format ? format(value) : String(value);

  return (
    <div className={`slider${disabled ? " slider--disabled" : ""}`}>
      <input
        id={id}
        type="range"
        className="slider__input"
        min={min}
        max={max}
        step={step}
        value={value}
        disabled={disabled}
        style={{ "--pct": `${pct}%` } as CSSProperties}
        onChange={(e) => {
          const next = Number(e.target.value);
          latest.current = next;
          onChange(next);
          scheduleCommit();
        }}
        onPointerUp={commitNow}
        onKeyUp={commitNow}
        onBlur={commitNow}
      />
      <output className="slider__bubble">{display}</output>
    </div>
  );
}
