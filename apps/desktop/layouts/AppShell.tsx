import { useEffect, useRef } from "react";
import { ActivitySidebar } from "./ActivitySidebar";
import { ResourcePanel } from "./ResourcePanel";
import { RightInspector } from "./RightInspector";
import { StatusBar } from "./StatusBar";
import { TopMenuBar } from "./TopMenuBar";
import { Workbench } from "./Workbench";
import { useAppState } from "../store/appState";
import { useSettings } from "../store/settingsStore";
import { AboutDialog } from "../components/AboutDialog";
import { desktopApi } from "../services/desktopApi";

export function AppShell() {
  const { theme, activeSection, sidebarCollapsed, setSidebarCollapsed, setTheme } = useAppState();
  const { settings } = useSettings();
  const didAutoStartRag = useRef(false);
  const isHome = activeSection === "home";

  // Sync theme from settings on mount
  useEffect(() => {
    if (settings.theme && settings.theme !== theme) {
      setTheme(settings.theme);
    }
    if (settings.sidebarCollapsed !== sidebarCollapsed) {
      setSidebarCollapsed(settings.sidebarCollapsed);
    }
  }, []);

  useEffect(() => {
    if (!settings.ragAutoStart || didAutoStartRag.current) {
      return;
    }
    didAutoStartRag.current = true;
    void desktopApi.startRagService().catch(() => {
      didAutoStartRag.current = false;
    });
  }, [settings.ragAutoStart]);

  const showResourcePanel = !isHome && (activeSection === "workspace" || activeSection === "files");
  const noResource = !showResourcePanel && !isHome;
  const noInspector = !settings.inspectorVisible;

  const bodyClass = [
    "app-body",
    isHome ? "is-home" : "",
    noResource ? "no-resource" : "",
    noResource && noInspector ? "no-inspector" : "",
  ].filter(Boolean).join(" ");

  return (
    <div className="app-shell" data-theme={theme}>
      <TopMenuBar />
      <div className={bodyClass}>
        <ActivitySidebar />
        {showResourcePanel ? <ResourcePanel /> : null}
        <Workbench />
        {settings.inspectorVisible ? <RightInspector /> : null}
      </div>
      <StatusBar />
      <AboutDialog />
    </div>
  );
}
