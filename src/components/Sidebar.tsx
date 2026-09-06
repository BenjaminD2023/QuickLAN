import {
  Network,
  Plus,
  UserRoundPlus,
  Settings,
  Info,
  Activity,
  Power,
  BookOpen,
} from "lucide-react";
import type { SavedNetwork } from "../lib/types";
import { BRAND } from "../lib/brand";
import { useText } from "../lib/i18n";
export type Page =
  | "networks"
  | "preferences"
  | "diagnostics"
  | "about"
  | "help";
export function Sidebar({
  networks,
  selected,
  page,
  onPage,
  onSelect,
  onCreate,
  onJoin,
  onQuit,
}: {
  networks: SavedNetwork[];
  selected?: string;
  page: Page;
  onPage: (p: Page) => void;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onJoin: () => void;
  onQuit: () => void;
}) {
  const t = useText();
  return (
    <aside className="sidebar">
      <div className="brand">
        <Network aria-hidden="true" size={30} />
        <span>{BRAND.name}</span>
      </div>
      <nav aria-label={t("networks")} className="network-nav">
        <button
          className={`nav-row ${page === "networks" ? "selected" : ""}`}
          onClick={() => onPage("networks")}
        >
          <Network />
          {t("networks")}
        </button>
        <div className="saved-networks">
          {networks.map((n) => (
            <button
              key={n.id}
              className={`saved-network ${page === "networks" && selected === n.id ? "active" : ""}`}
              title={n.label}
              onClick={() => onSelect(n.id)}
            >
              <span className="network-dot" />
              <span>{n.label}</span>
            </button>
          ))}
        </div>
        <button className="nav-row" onClick={onCreate}>
          <Plus />
          {t("create")}
        </button>
        <button className="nav-row" onClick={onJoin}>
          <UserRoundPlus />
          {t("join")}
        </button>
      </nav>
      <nav className="utility-nav" aria-label={t("preferences")}>
        <button
          className={`nav-row ${page === "diagnostics" ? "selected" : ""}`}
          onClick={() => onPage("diagnostics")}
        >
          <Activity />
          {t("diagnostics")}
        </button>
        <button
          className={`nav-row ${page === "help" ? "selected" : ""}`}
          onClick={() => onPage("help")}
        >
          <BookOpen />
          {t("help")}
        </button>
        <button
          className={`nav-row ${page === "preferences" ? "selected" : ""}`}
          onClick={() => onPage("preferences")}
        >
          <Settings />
          {t("preferences")}
        </button>
        <button
          className={`nav-row ${page === "about" ? "selected" : ""}`}
          onClick={() => onPage("about")}
        >
          <Info />
          {t("about")}
        </button>
        <button className="nav-row quit" onClick={onQuit}>
          <Power />
          {t("quit")}
        </button>
      </nav>
    </aside>
  );
}
