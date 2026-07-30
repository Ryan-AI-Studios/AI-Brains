import { NavLink, Outlet } from "react-router";
import {
  BookOpen,
  Eraser,
  FileSearch,
  Home,
  Link2,
  ListChecks,
  MapPin,
  Search,
  Shield,
} from "lucide-react";
import { ScopeIndicator } from "./ScopeIndicator";

const nav: {
  to: string;
  label: string;
  icon: typeof Home;
  end?: boolean;
}[] = [
  { to: "/", label: "Home", icon: Home, end: true },
  { to: "/review", label: "Review", icon: ListChecks },
  { to: "/scope", label: "Scope", icon: MapPin },
  { to: "/query", label: "Query", icon: Search },
  { to: "/evidence", label: "Evidence", icon: BookOpen },
  { to: "/source", label: "Source", icon: FileSearch },
  { to: "/erasure", label: "Erasure", icon: Eraser },
  { to: "/connectors", label: "Connectors", icon: Link2 },
  { to: "/claim", label: "Claim", icon: Shield },
];

export function Layout() {
  return (
    <div className="shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand">
          <strong>AI-Brains</strong>
          <span className="muted small">Desktop · adapter-only</span>
        </div>
        <nav className="nav">
          {nav.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end ?? false}
              className={({ isActive }) =>
                isActive ? "nav-link active" : "nav-link"
              }
            >
              <Icon size={16} aria-hidden />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>
      </aside>
      <div className="main-column">
        <header className="topbar">
          <ScopeIndicator />
        </header>
        <main className="content">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
