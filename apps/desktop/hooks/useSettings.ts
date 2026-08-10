import { useEffect, useState } from "react";
import { loadSettings, resetSettings, saveSettings, SettingsLoadResult, validateSettings } from "../services/settingsService";
import { useAppState } from "../store/appState";

export function useSettings() {
  const { workspaceState } = useAppState();
  const [settings, setSettings] = useState<SettingsLoadResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = () => {
    setLoading(true);
    setError(null);
    return loadSettings()
      .then((result) => {
        setSettings(result);
        return result;
      })
      .catch((err) => {
        setError(String(err));
        throw err;
      })
      .finally(() => {
        setLoading(false);
      });
  };

  const save = async (jsonText: string) => {
    setError(null);
    const validation = await validateSettings(jsonText);
    if (!validation.valid) {
      const message = validation.errors.join("\n");
      setError(message);
      throw new Error(message);
    }
    const result = await saveSettings(jsonText);
    setSettings(result);
    return result;
  };

  const reset = async () => {
    setError(null);
    setLoading(true);
    return resetSettings()
      .then((result) => {
        setSettings(result);
        return result;
      })
      .catch((err) => {
        setError(String(err));
        throw err;
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    let mounted = true;
    loadSettings()
      .then((result) => {
        if (mounted) setSettings(result);
      })
      .catch((err) => {
        if (mounted) setError(String(err));
      })
      .finally(() => {
        if (mounted) setLoading(false);
      });
    return () => {
      mounted = false;
    };
  }, [workspaceState.current_workspace]);

  return { settings, loading, error, reload, save, reset, validate: validateSettings };
}
