import type { KeyboardEffect, KeyboardState, Profile } from "../../types";
import { useLocalStorage } from "../../hooks";
import { Tabs } from "../ui/Tabs";
import type { TabItem } from "../ui/Tabs";
import { KeyboardLighting } from "./KeyboardLighting";
import { KeyboardPerKey } from "./KeyboardPerKey";
import { KeyboardRemap } from "./KeyboardRemap";
import { KeyboardProfiles } from "./KeyboardProfiles";

type KeyboardTab = "led" | "perkey" | "remap" | "profiles";

const TABS: TabItem<KeyboardTab>[] = [
  { id: "led", label: "Iluminação" },
  { id: "perkey", label: "Cores por tecla" },
  { id: "remap", label: "Remapear teclas" },
  { id: "profiles", label: "Perfis" },
];

interface KeyboardPanelProps {
  state: KeyboardState | null;
  effects: KeyboardEffect[];
  present: boolean;
  profile: Profile;
  onProfile: (profile: Profile) => void;
  applied: string;
  onApplied: () => void;
}

export function KeyboardPanel({
  state,
  effects,
  present,
  profile,
  onProfile,
  applied,
  onApplied,
}: KeyboardPanelProps) {
  const [tab, setTab] = useLocalStorage<KeyboardTab>("rdo.keyboardTab", "led");

  return (
    <div className="workspace">
      <div className="workspace__top">
        <div className="workspace__heading">
          <h1>Brahma Pro</h1>
          <span className="chip chip--muted">Perfil {profile}</span>
          <span className={`chip${present ? " chip--on" : " chip--off"}`}>
            {present ? "Conectado" : "Desconectado"}
          </span>
        </div>
        <Tabs tabs={TABS} active={tab} onChange={setTab} />
      </div>

      {!present && (
        <p className="disconnected-note">
          Teclado não detectado. Conecte o cabo para aplicar alterações — os
          controles abaixo continuam editáveis.
        </p>
      )}

      <div className="workspace__body">
        {tab === "led" && (
          <KeyboardLighting
            state={state}
            effects={effects}
            profile={profile}
            applied={applied}
            onApplied={onApplied}
          />
        )}
        {tab === "perkey" && <KeyboardPerKey profile={profile} applied={applied} />}
        {tab === "remap" && <KeyboardRemap applied={applied} />}
        {tab === "profiles" && (
          <KeyboardProfiles
            profile={profile}
            onProfile={onProfile}
            applied={applied}
          />
        )}
      </div>
    </div>
  );
}
