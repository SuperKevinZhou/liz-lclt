import { t } from "../lib/i18n";
import type { NavKey } from "../types/app";

const navItems: Array<{ key: NavKey; label: string; hint: string }> = [
  { key: "overview", label: t("navOverview"), hint: t("navOverviewHint") },
  {
    key: "settings",
    label: t("navSettings"),
    hint: t("navSettingsHint"),
  },
  {
    key: "models",
    label: t("navModels"),
    hint: t("navModelsHint"),
  },
  {
    key: "strategies",
    label: t("navStrategies"),
    hint: t("navStrategiesHint"),
  },
  {
    key: "resources",
    label: t("navResources"),
    hint: t("navResourcesHint"),
  },
];

interface SidebarProps {
  active: NavKey;
  onSelect: (key: NavKey) => void;
}

export function Sidebar({ active, onSelect }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand__badge">LCLT</div>
        <div>
          <h1>liz-lclt</h1>
          <p>{t("productHint")}</p>
        </div>
      </div>

      <nav className="nav">
        {navItems.map((item) => (
          <button
            key={item.key}
            className={
              item.key === active ? "nav__item nav__item--active" : "nav__item"
            }
            onClick={() => onSelect(item.key)}
            type="button"
          >
            <span>{item.label}</span>
            <small>{item.hint}</small>
          </button>
        ))}
      </nav>
    </aside>
  );
}
