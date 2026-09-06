import { useCallback, useEffect, useRef, useState } from "react";
import type { Rgb } from "./types";

// ---------------------------------------------------------------------------
// localStorage-backed state (wrapped in try/catch — storage may be unavailable)
// ---------------------------------------------------------------------------

export function useLocalStorage<T>(
  key: string,
  initial: T,
): [T, (value: T) => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      const raw = localStorage.getItem(key);
      return raw === null ? initial : (JSON.parse(raw) as T);
    } catch {
      return initial;
    }
  });

  const set = useCallback(
    (next: T) => {
      setValue(next);
      try {
        localStorage.setItem(key, JSON.stringify(next));
      } catch {
        // ignore write failures (private mode, quota, etc.)
      }
    },
    [key],
  );

  return [value, set];
}

// ---------------------------------------------------------------------------
// Debounced callback — used to throttle writes while a slider is dragged
// ---------------------------------------------------------------------------

export function useDebouncedCallback<A extends unknown[]>(
  fn: (...args: A) => void,
  delay: number,
): (...args: A) => void {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const fnRef = useRef(fn);
  fnRef.current = fn;

  useEffect(
    () => () => {
      if (timer.current !== null) clearTimeout(timer.current);
    },
    [],
  );

  return useCallback(
    (...args: A) => {
      if (timer.current !== null) clearTimeout(timer.current);
      timer.current = setTimeout(() => {
        fnRef.current(...args);
      }, delay);
    },
    [delay],
  );
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

function toHexByte(n: number): string {
  return Math.max(0, Math.min(255, Math.round(n)))
    .toString(16)
    .padStart(2, "0");
}

export function rgbToHex([r, g, b]: Rgb): string {
  return `#${toHexByte(r)}${toHexByte(g)}${toHexByte(b)}`;
}

export function hexToRgb(hex: string): Rgb | null {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return null;
  const int = parseInt(m[1], 16);
  return [(int >> 16) & 0xff, (int >> 8) & 0xff, int & 0xff];
}

export function rgbCss([r, g, b]: Rgb): string {
  return `rgb(${r}, ${g}, ${b})`;
}
