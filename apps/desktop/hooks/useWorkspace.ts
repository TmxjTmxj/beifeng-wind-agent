import { desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";

export function useWorkspace() {
  const {
    workspaceState,
    workspaceLoading,
    refreshWorkspaceState,
    selectWorkspaceFolder,
    createWorkspacePath,
    importWorkspaceFolder,
    setCurrentWorkspacePath,
    archiveWorkspacePath,
    removeWorkspacePath
  } = useAppState();

  return {
    workspaceState,
    loading: workspaceLoading,
    refresh: refreshWorkspaceState,
    selectFolder: selectWorkspaceFolder,
    createWorkspace: createWorkspacePath,
    importWorkspace: importWorkspaceFolder,
    setWorkspace: setCurrentWorkspacePath,
    archiveWorkspace: archiveWorkspacePath,
    removeWorkspace: removeWorkspacePath,
    openInVSCode: desktopApi.openInVSCode,
    revealInExplorer: desktopApi.revealInExplorer
  };
}
