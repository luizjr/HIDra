import { useEffect, useState } from "react";
import type { Rgb } from "../../types";
import { hexToRgb, rgbToHex } from "../../hooks";

const DEFAULT_PRESETS: Rgb[] = [
  [216, 31, 38], // signature red
  [230, 57, 70],
  [255, 120, 0],
  [255, 214, 0],
  [0, 214, 90],
  [0, 190, 255],
  [40, 90, 255],
  [150, 0, 255],
  [255, 0, 180],
  [255, 255, 255],
];

interface ColorPickerProps {
  value: Rgb;
  onChange: (value: Rgb) => void;
  onCommit?: (value: Rgb) => void;
  presets?: Rgb[];
  disabled?: boolean;
}

export function ColorPicker({
  value,
  onChange,
  onCommit,
  presets = DEFAULT_PRESETS,
  disabled,
}: ColorPickerProps) {
  const hex = rgbToHex(value);
  const [hexDraft, setHexDraft] = useState(hex);

  // Keep the text field in sync when the color changes elsewhere.
  useEffect(() => {
    setHexDraft(rgbToHex(value));
  }, [value]);

  const commitHex = () => {
    const parsed = hexToRgb(hexDraft);
    if (parsed) {
      onChange(parsed);
      onCommit?.(parsed);
    } else {
      setHexDraft(hex);
    }
  };

  return (
    <div className={`colorpicker${disabled ? " colorpicker--disabled" : ""}`}>
      <div className="colorpicker__top">
        <label className="colorpicker__swatch" style={{ background: hex }}>
          <input
            type="color"
            value={hex}
            disabled={disabled}
            onChange={(e) => {
              const parsed = hexToRgb(e.target.value);
              if (parsed) onChange(parsed);
            }}
            onBlur={() => {
              const parsed = hexToRgb(hexDraft);
              if (parsed) onCommit?.(parsed);
            }}
          />
        </label>
        <div className="colorpicker__hexwrap">
          <span className="colorpicker__hash">#</span>
          <input
            className="colorpicker__hex"
            type="text"
            inputMode="text"
            maxLength={7}
            spellCheck={false}
            value={hexDraft.replace(/^#/, "")}
            disabled={disabled}
            onChange={(e) => setHexDraft(`#${e.target.value.replace(/[^0-9a-fA-F]/g, "")}`)}
            onBlur={commitHex}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitHex();
            }}
          />
        </div>
      </div>
      <div className="colorpicker__swatches">
        {presets.map((preset) => {
          const pHex = rgbToHex(preset);
          return (
            <button
              key={pHex}
              type="button"
              className={`swatch${pHex === hex ? " is-active" : ""}`}
              style={{ background: pHex }}
              title={pHex}
              disabled={disabled}
              onClick={() => {
                onChange(preset);
                onCommit?.(preset);
              }}
            />
          );
        })}
      </div>
    </div>
  );
}
