import { KEYBOARD_ROWS } from "../../constants";

interface KeyboardDiagramProps {
  /** Returns the CSS colour for the face of the key at this LED slot index. */
  colorAt: (slotIndex: number) => string;
  onKeyClick?: (slotIndex: number) => void;
  selectedSlot?: number | null;
  interactive?: boolean;
}

export function KeyboardDiagram({
  colorAt,
  onKeyClick,
  selectedSlot = null,
  interactive = false,
}: KeyboardDiagramProps) {
  let slot = -1;

  return (
    <div className={`kbd${interactive ? " kbd--interactive" : ""}`}>
      {KEYBOARD_ROWS.map((row, r) => (
        <div className="kbd__row" key={r}>
          {row.map((key, c) => {
            if (key.spacer) {
              return (
                <span
                  key={`s${c}`}
                  className="kbd__spacer"
                  style={{ flexBasis: `calc(${key.w ?? 0.5} * var(--u))` }}
                />
              );
            }
            slot += 1;
            const thisSlot = slot;
            const face = colorAt(thisSlot);
            const width = `calc(${key.w ?? 1} * var(--u))`;
            const common = {
              className: `kbd__key${
                thisSlot === selectedSlot ? " is-selected" : ""
              }`,
              style: {
                flexBasis: width,
                background: face,
              },
              title: key.label,
            };
            return interactive ? (
              <button
                key={c}
                type="button"
                {...common}
                onClick={() => onKeyClick?.(thisSlot)}
              >
                <span className="kbd__cap">{key.label}</span>
              </button>
            ) : (
              <span key={c} {...common}>
                <span className="kbd__cap">{key.label}</span>
              </span>
            );
          })}
        </div>
      ))}
    </div>
  );
}
