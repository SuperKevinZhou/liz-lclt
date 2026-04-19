import type { NavKey } from "../types/app";

const navItems: Array<{ key: NavKey; label: string; hint: string }> = [
  { key: "overview", label: "Overview", hint: "Run and inspect task state" },
  {
    key: "settings",
    label: "Base Settings",
    hint: "Core translation and path options",
  },
  {
    key: "models",
    label: "Models",
    hint: "Slots, endpoints, and temperatures",
  },
  {
    key: "strategies",
    label: "Strategies",
    hint: "Pattern rules, prompt and terminology bindings",
  },
  {
    key: "resources",
    label: "Resources",
    hint: "Prompts, terminology, and blacklist",
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
          <p>Desktop translator workbench</p>
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
