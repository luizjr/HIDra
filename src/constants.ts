// Static lookup tables: HID usages, media usages, mouse button mapping and the
// keyboard layout used by the per-key painter and the diagrams.

import type { KeyboardEffect } from "./types";

// ---------------------------------------------------------------------------
// HID keyboard usages
// ---------------------------------------------------------------------------

export interface Usage {
  label: string;
  code: number;
}

function range(start: number, labels: string[]): Usage[] {
  return labels.map((label, i) => ({ label, code: start + i }));
}

const LETTERS = range(
  0x04,
  "abcdefghijklmnopqrstuvwxyz".split(""),
).map((u) => ({ label: u.label.toUpperCase(), code: u.code }));

const DIGITS: Usage[] = [
  ...range(0x1e, ["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
  { label: "0", code: 0x27 },
];

const FKEYS = range(
  0x3a,
  Array.from({ length: 12 }, (_v, i) => `F${i + 1}`),
);

const SPECIALS: Usage[] = [
  { label: "Enter", code: 0x28 },
  { label: "Esc", code: 0x29 },
  { label: "Backspace", code: 0x2a },
  { label: "Tab", code: 0x2b },
  { label: "Espaço", code: 0x2c },
  { label: "Insert", code: 0x49 },
  { label: "Home", code: 0x4a },
  { label: "PageUp", code: 0x4b },
  { label: "Delete", code: 0x4c },
  { label: "End", code: 0x4d },
  { label: "PageDown", code: 0x4e },
  { label: "Seta →", code: 0x4f },
  { label: "Seta ←", code: 0x50 },
  { label: "Seta ↓", code: 0x51 },
  { label: "Seta ↑", code: 0x52 },
];

/** Full ordered list of assignable HID keys (target-key dropdowns). */
export const KEY_USAGES: Usage[] = [
  ...LETTERS,
  ...DIGITS,
  ...FKEYS,
  ...SPECIALS,
];

// ---------------------------------------------------------------------------
// HID modifiers (bitmask)
// ---------------------------------------------------------------------------

export const MODIFIERS: Usage[] = [
  { label: "Ctrl", code: 1 },
  { label: "Shift", code: 2 },
  { label: "Alt", code: 4 },
  { label: "Win", code: 8 },
];

// ---------------------------------------------------------------------------
// Consumer-page media usages (16-bit)
// ---------------------------------------------------------------------------

export interface MediaUsage {
  label: string;
  usage: number;
}

export const MEDIA_USAGES: MediaUsage[] = [
  { label: "Play / Pause", usage: 0x00cd },
  { label: "Próxima", usage: 0x00b5 },
  { label: "Anterior", usage: 0x00b6 },
  { label: "Parar", usage: 0x00b7 },
  { label: "Mudo", usage: 0x00e2 },
  { label: "Volume +", usage: 0x00e9 },
  { label: "Volume -", usage: 0x00ea },
];

// ---------------------------------------------------------------------------
// Mouse buttons
// ---------------------------------------------------------------------------

export interface MouseButtonSlot {
  /** UI number shown on the diagram (1..8). */
  num: number;
  /** Index into the hardware button table. */
  index: number;
  label: string;
  /** Position on the top-view diagram, in percentages. */
  x: number;
  y: number;
}

/** UI button -> table index: 1->0,2->1,3->2,4->3,5->4,6->10,7->11,8->9 */
export const MOUSE_BUTTONS: MouseButtonSlot[] = [
  { num: 1, index: 0, label: "Botão esquerdo", x: 27, y: 22 },
  { num: 2, index: 1, label: "Botão direito", x: 73, y: 22 },
  { num: 3, index: 2, label: "Roda / meio", x: 50, y: 30 },
  { num: 4, index: 3, label: "Lateral frontal", x: 8, y: 40 },
  { num: 5, index: 4, label: "Lateral traseiro", x: 8, y: 52 },
  { num: 6, index: 10, label: "Atrás da roda (1)", x: 50, y: 46 },
  { num: 7, index: 11, label: "Atrás da roda (2)", x: 38, y: 55 },
  { num: 8, index: 9, label: "Atrás da roda (3)", x: 62, y: 55 },
];

export const POLLING_RATES = [125, 250, 500, 1000] as const;

// ---------------------------------------------------------------------------
// Keyboard effects (fallback used only by the browser MOCK layer)
// ---------------------------------------------------------------------------

export const KEYBOARD_EFFECTS_FALLBACK: KeyboardEffect[] = [
  { id: 1, name: "Corredor", has_color: false, has_direction: true },
  { id: 2, name: "A Nuvem", has_color: false, has_direction: false },
  { id: 3, name: "Lâmina Veloz", has_color: false, has_direction: true },
  { id: 4, name: "Spectrum", has_color: false, has_direction: true },
  { id: 5, name: "Respiração", has_color: true, has_direction: false },
  { id: 6, name: "Sólido", has_color: true, has_direction: false },
  { id: 7, name: "Reativo", has_color: true, has_direction: false },
  { id: 8, name: "Ondular", has_color: false, has_direction: true },
  { id: 9, name: "Reativo (Horizontal)", has_color: true, has_direction: true },
  { id: 10, name: "Florescer Gélido", has_color: false, has_direction: false },
  { id: 11, name: "Rainbow", has_color: false, has_direction: true },
  { id: 12, name: "Corrida das Sombras", has_color: false, has_direction: true },
  { id: 13, name: "Tornado", has_color: false, has_direction: true },
  { id: 14, name: "Recarregar", has_color: false, has_direction: false },
  { id: 15, name: "A Matrix", has_color: false, has_direction: false },
  { id: 16, name: "Surmount", has_color: false, has_direction: false },
  { id: 17, name: "Passagem Dupla", has_color: false, has_direction: true },
  { id: 18, name: "Vulto do Spectro", has_color: false, has_direction: false },
  { id: 20, name: "Customizável", has_color: true, has_direction: false },
];

export const CUSTOM_EFFECT_ID = 20;

// ---------------------------------------------------------------------------
// Keyboard layout (ABNT2 / ANSI TKL-ish) for the diagram and per-key painter
// ---------------------------------------------------------------------------

export interface KeyDef {
  label: string;
  /** Width in key units (1 = standard key). */
  w?: number;
  /** Invisible filler for cluster gaps. */
  spacer?: boolean;
}

const K = (label: string, w?: number): KeyDef => ({ label, w });
const GAP = (w = 0.5): KeyDef => ({ label: "", w, spacer: true });

export const KEYBOARD_ROWS: KeyDef[][] = [
  [
    K("Esc"),
    GAP(),
    K("F1"), K("F2"), K("F3"), K("F4"),
    GAP(0.4),
    K("F5"), K("F6"), K("F7"), K("F8"),
    GAP(0.4),
    K("F9"), K("F10"), K("F11"), K("F12"),
    GAP(0.35),
    K("PrtSc"), K("Scroll"), K("Pause"),
  ],
  [
    K("'"), K("1"), K("2"), K("3"), K("4"), K("5"), K("6"), K("7"), K("8"),
    K("9"), K("0"), K("-"), K("="), K("Backspace", 2),
    GAP(0.35),
    K("Ins"), K("Home"), K("PgUp"),
  ],
  [
    K("Tab", 1.5),
    K("Q"), K("W"), K("E"), K("R"), K("T"), K("Y"), K("U"), K("I"), K("O"),
    K("P"), K("´"), K("["),
    K("Enter", 1.5),
    GAP(0.35),
    K("Del"), K("End"), K("PgDn"),
  ],
  [
    K("Caps", 1.75),
    K("A"), K("S"), K("D"), K("F"), K("G"), K("H"), K("J"), K("K"), K("L"),
    K("Ç"), K("~"), K("]"),
    K("Enter", 1.25),
  ],
  [
    K("Shift", 2.25),
    K("\\"),
    K("Z"), K("X"), K("C"), K("V"), K("B"), K("N"), K("M"), K(","), K("."),
    K(";"), K("Shift", 1.75),
    GAP(0.35),
    K("↑"),
  ],
  [
    K("Ctrl", 1.4), K("Win", 1.1), K("Alt", 1.1),
    K("Espaço", 6.25),
    K("AltGr", 1.1), K("Fn", 1.1), K("Menu", 1.1), K("Ctrl", 1.4),
    GAP(0.35),
    K("←"), K("↓"), K("→"),
  ],
];

/** Ordered, paintable keys (spacers removed) — index = per-key LED slot. */
export interface LayoutSlot {
  label: string;
  row: number;
  col: number;
}

export const KEY_SLOTS: LayoutSlot[] = (() => {
  const slots: LayoutSlot[] = [];
  KEYBOARD_ROWS.forEach((row, r) => {
    row.forEach((key, c) => {
      if (!key.spacer) slots.push({ label: key.label, row: r, col: c });
    });
  });
  return slots;
})();

/** Hardware per-key buffer holds up to 144 RGB slots. */
export const MAX_KEY_SLOTS = 144;
