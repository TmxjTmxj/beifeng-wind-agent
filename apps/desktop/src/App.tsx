import { AppShell } from "../layouts/AppShell";
import { AppStateProvider } from "../store/appState";
import { I18nProvider } from "../store/i18n";
import { SettingsProvider } from "../store/settingsStore";

export function App() {
  return (
    <I18nProvider>
      <SettingsProvider>
        <AppStateProvider>
          <AppShell />
        </AppStateProvider>
      </SettingsProvider>
    </I18nProvider>
  );
}
