import { useEffect, useState } from "react";
import type {
  KeyboardEffect,
  KeyboardState,
  Profile,
  Rgb,
} from "../../types";
import { keyboardSetEffect } from "../../backend";
import { CUSTOM_EFFECT_ID } from "../../constants";
import { rgbCss } from "../../hooks";
import { useRun } from "../Toast";
import { Panel, Field } from "../ui/Panel";
import { Slider } from "../ui/Slider";
import { Toggle } from "../ui/Toggle";
import { ColorPicker } from "../ui/ColorPicker";
import { SegmentedControl } from "../ui/SegmentedControl";
import { KeyboardDiagram } from "./KeyboardDiagram";

interface KeyboardLightingProps {
  state: KeyboardState | null;
  effects: KeyboardEffect[];
  profile: Profile;
  applied: string;
  onApplied: () => void;
}

function scale([r, g, b]: Rgb, factor: number): string {
  return rgbCss([
    Math.round(r * factor),
    Math.round(g * factor),
    Math.round(b * factor),
  ]);
}

function rainbow(slot: number, factor: number): string {
  const hue = (slot * 11) % 360;
  return `hsl(${hue} 90% ${Math.round(30 + 30 * factor)}%)`;
}

export function KeyboardLighting({
  state,
  effects,
  profile,
  applied,
  onApplied,
}: KeyboardLightingProps) {
  const run = useRun();

  const [effect, setEffect] = useState<number>(6);
  const [color, setColor] = useState<Rgb>([216, 31, 38]);
  const [brightness, setBrightness] = useState(5);
  const [speed, setSpeed] = useState(3);
  const [direction, setDirection] = useState(0);
  const [fullRgb, setFullRgb] = useState(false);

  useEffect(() => {
    if (!state) return;
    if (state.effect != null) setEffect(state.effect);
    if (state.color) setColor(state.color);
    if (state.brightness != null) setBrightness(state.brightness);
    if (state.speed != null) setSpeed(state.speed);
    if (state.direction != null) setDirection(state.direction);
    if (state.full_rgb != null) setFullRgb(state.full_rgb);
  }, [state, profile]);

  const meta = effects.find((e) => e.id === effect);
  const hasColor = meta?.has_color ?? false;
  const hasDirection = meta?.has_direction ?? false;
  const isCustom = effect === CUSTOM_EFFECT_ID;
  const factor = brightness / 5;

  const colorAt = (slot: number): string => {
    if (brightness === 0) return "#141519";
    if (hasColor && !fullRgb) return scale(color, 0.25 + 0.75 * factor);
    return rainbow(slot, factor);
  };

  const apply = async () => {
    const ok = await run(
      () =>
        keyboardSetEffect({
          profile,
          effect,
          r: color[0],
          g: color[1],
          b: color[2],
          brightness,
          speed,
          direction,
          full_rgb: fullRgb,
        }),
      applied,
    );
    if (ok) onApplied();
  };

  return (
    <div className="tabgrid">
      <Panel
        title="Iluminação"
        subtitle={`Efeito de luz do Perfil ${profile}`}
        actions={
          <button type="button" className="btn btn--primary" onClick={apply}>
            Aplicar
          </button>
        }
      >
        <Field label="Efeito">
          <select
            className="input select"
            value={effect}
            onChange={(e) => setEffect(Number(e.target.value))}
          >
            {effects.map((e) => (
              <option key={e.id} value={e.id}>
                {e.name}
              </option>
            ))}
          </select>
        </Field>

        {isCustom && (
          <p className="note">
            Modo Customizável: pinte cada tecla na aba{" "}
            <strong>Cores por tecla</strong>.
          </p>
        )}

        {hasColor && (
          <Field label="Multicolor (Full RGB)">
            <Toggle
              checked={fullRgb}
              onChange={setFullRgb}
              label={fullRgb ? "Ligado" : "Desligado"}
            />
          </Field>
        )}

        {hasColor && !fullRgb && (
          <Field label="Cor">
            <ColorPicker value={color} onChange={setColor} />
          </Field>
        )}

        {hasDirection && (
          <Field label="Direção">
            <SegmentedControl<number>
              value={direction}
              onChange={setDirection}
              options={[
                { value: 0, label: "→ Direita" },
                { value: 1, label: "← Esquerda" },
              ]}
            />
          </Field>
        )}

        <Field label="Brilho">
          <Slider
            min={0}
            max={5}
            value={brightness}
            onChange={setBrightness}
            format={(v) => `${v}/5`}
          />
        </Field>

        <Field label="Velocidade" hint="Menor = mais rápido">
          <Slider
            min={1}
            max={5}
            value={speed}
            onChange={setSpeed}
            format={(v) => `${v}/5`}
          />
        </Field>
      </Panel>

      <Panel title="Pré-visualização" className="preview-panel">
        <div className="preview-stage preview-stage--kbd">
          <KeyboardDiagram colorAt={colorAt} />
        </div>
      </Panel>
    </div>
  );
}
