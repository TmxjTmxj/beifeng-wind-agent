import { createContext, ReactNode, useContext, useEffect, useMemo, useState } from "react";
import { desktopApi } from "../services/desktopApi";
import enUS from "../locales/en-US.json";
import zhCN from "../locales/zh-CN.json";

export type Language = "en-US" | "zh-CN";

type Messages = Record<string, string>;

const dictionaries: Record<Language, Messages> = {
  "en-US": enUS,
  "zh-CN": zhCN
};

type I18nContextValue = {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: string) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguage] = useState<Language>("zh-CN");
  const [loadedPreference, setLoadedPreference] = useState(false);

  useEffect(() => {
    let mounted = true;
    void desktopApi.getLanguagePreference().then((value) => {
      if (mounted && (value === "zh-CN" || value === "en-US")) {
        setLanguage(value);
      }
      if (mounted) {
        setLoadedPreference(true);
      }
    });
    return () => {
      mounted = false;
    };
  }, []);

  const updateLanguage = (nextLanguage: Language) => {
    setLanguage(nextLanguage);
    if (loadedPreference) {
      void desktopApi.setLanguagePreference(nextLanguage).catch(() => undefined);
    }
  };

  const value = useMemo<I18nContextValue>(() => {
    const messages = dictionaries[language];
    return {
      language,
      setLanguage: updateLanguage,
      t: (key: string) => messages[key] ?? dictionaries["en-US"][key] ?? key
    };
  }, [language, loadedPreference]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used inside I18nProvider");
  }
  return context;
}
