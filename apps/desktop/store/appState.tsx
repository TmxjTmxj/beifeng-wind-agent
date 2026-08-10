import { createContext, ReactNode, useContext, useEffect, useMemo, useState } from "react";
import { desktopApi, WorkspaceState } from "../services/desktopApi";

export type Section =
  | "home"
  | "workspace"
  | "chats"
  | "files"
  | "memory"
  | "reports"
  | "benchmark"
  | "skills"
  | "connectors"
  | "system"
  | "console"
  | "settings";

export type ThemeMode = "dark" | "light";
export type TabId = "chat" | "report" | "settings";

type AppState = {
  activeSection: Section;
  setActiveSection: (section: Section) => void;
  activeTab: TabId;
  setActiveTab: (tab: TabId) => void;
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (collapsed: boolean) => void;
  inspectorCollapsed: boolean;
  setInspectorCollapsed: (collapsed: boolean) => void;
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
  aboutOpen: boolean;
  setAboutOpen: (open: boolean) => void;
  workspaceState: WorkspaceState;
  workspaceLoading: boolean;
  refreshWorkspaceState: () => Promise<WorkspaceState>;
  selectWorkspaceFolder: () => Promise<WorkspaceState>;
  createWorkspacePath: (path: string) => Promise<WorkspaceState>;
  importWorkspaceFolder: () => Promise<WorkspaceState>;
  setCurrentWorkspacePath: (path: string) => Promise<WorkspaceState>;
  archiveWorkspacePath: (path: string) => Promise<WorkspaceState>;
  removeWorkspacePath: (path: string) => Promise<WorkspaceState>;
};

const AppStateContext = createContext<AppState | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [activeSection, setActiveSection] = useState<Section>("home");
  const [activeTab, setActiveTab] = useState<TabId>("chat");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [inspectorCollapsed, setInspectorCollapsed] = useState(true);
  const [theme, setTheme] = useState<ThemeMode>("dark");
  const [aboutOpen, setAboutOpen] = useState(false);
  const [workspaceState, setWorkspaceState] = useState<WorkspaceState>({
    current_workspace: null,
    recent_workspaces: []
  });
  const [workspaceLoading, setWorkspaceLoading] = useState(true);

  const refreshWorkspaceState = async () => {
    setWorkspaceLoading(true);
    const state = await desktopApi.getCurrentWorkspace();
    setWorkspaceState(state);
    setWorkspaceLoading(false);
    return state;
  };

  const selectWorkspaceFolder = async () => {
    const state = await desktopApi.selectWorkspaceFolder();
    setWorkspaceState(state);
    return state;
  };

  const createWorkspacePath = async (path: string) => {
    const state = await desktopApi.createWorkspace(path);
    setWorkspaceState(state);
    return state;
  };

  const importWorkspaceFolder = async () => {
    const state = await desktopApi.importWorkspaceFolder();
    setWorkspaceState(state);
    return state;
  };

  const setCurrentWorkspacePath = async (path: string) => {
    const state = await desktopApi.setCurrentWorkspace(path);
    setWorkspaceState(state);
    return state;
  };

  const archiveWorkspacePath = async (path: string) => {
    const state = await desktopApi.archiveWorkspace(path);
    setWorkspaceState(state);
    return state;
  };

  const removeWorkspacePath = async (path: string) => {
    const state = await desktopApi.removeWorkspaceFromList(path);
    setWorkspaceState(state);
    return state;
  };

  useEffect(() => {
    refreshWorkspaceState();
  }, []);

  const value = useMemo(
    () => ({
      activeSection,
      setActiveSection,
      activeTab,
      setActiveTab,
      sidebarCollapsed,
      setSidebarCollapsed,
      inspectorCollapsed,
      setInspectorCollapsed,
      theme,
      setTheme,
      aboutOpen,
      setAboutOpen,
      workspaceState,
      workspaceLoading,
      refreshWorkspaceState,
      selectWorkspaceFolder,
      createWorkspacePath,
      importWorkspaceFolder,
      setCurrentWorkspacePath,
      archiveWorkspacePath,
      removeWorkspacePath
    }),
    [activeSection, activeTab, sidebarCollapsed, inspectorCollapsed, theme, aboutOpen, workspaceState, workspaceLoading]
  );

  return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>;
}

export function useAppState() {
  const context = useContext(AppStateContext);
  if (!context) {
    throw new Error("useAppState must be used inside AppStateProvider");
  }
  return context;
}
