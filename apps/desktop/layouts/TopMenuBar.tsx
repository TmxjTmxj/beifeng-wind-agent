import { ChevronDown, Info, Languages, Moon, Sun } from "lucide-react";
import { KeyboardEvent, useEffect, useRef, useState } from "react";
import { Button } from "../components/Button";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";
import { useSettings } from "../store/settingsStore";
import logoDark from "../assets/branding/logo-dark.svg";
import logoLight from "../assets/branding/logo-light.svg";

export function TopMenuBar() {
  const { theme, setTheme, setAboutOpen, setActiveSection } = useAppState();
  const { settings, updateSettings } = useSettings();
  const { language, setLanguage, t } = useI18n();
  const [toolsOpen, setToolsOpen] = useState(false);
  const [languageOpen, setLanguageOpen] = useState(false);
  const closeTimer = useRef<number | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);

  const openTools = () => {
    if (closeTimer.current) { window.clearTimeout(closeTimer.current); closeTimer.current = null; }
    setToolsOpen(true);
  };
  const closeTools = () => { setToolsOpen(false); setLanguageOpen(false); };
  const closeToolsSoon = () => {
    if (closeTimer.current) window.clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(closeTools, 240);
  };
  const runMenuAction = (action: () => void) => { action(); closeTools(); };

  const handleMenuKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>("[data-menu-item]") ?? []);
    const currentIndex = items.findIndex((item) => item === document.activeElement);
    if (event.key === "Escape") { event.preventDefault(); closeTools(); }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault(); openTools();
      const direction = event.key === "ArrowDown" ? 1 : -1;
      const next = items[(currentIndex + direction + items.length) % items.length] ?? items[0];
      next?.focus();
    }
    if (event.key === "Enter" && document.activeElement instanceof HTMLButtonElement) {
      document.activeElement.click();
    }
  };

  useEffect(() => {
    const closeOnOutside = (event: MouseEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) closeTools();
    };
    document.addEventListener("mousedown", closeOnOutside);
    return () => {
      document.removeEventListener("mousedown", closeOnOutside);
      if (closeTimer.current) window.clearTimeout(closeTimer.current);
    };
  }, []);

  const changeLanguage = (lang: "zh-CN" | "en-US") => {
    setLanguage(lang);
    updateSettings({ language: lang });
  };

  const toggleTheme = () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    updateSettings({ theme: next });
  };

  return (
    <header className="top-menu">
      <div className="brand">
        <img className="brand-logo" src={theme === "dark" ? logoDark : logoLight} alt="" />
        <strong>{t("app.title")}</strong>
      </div>
      <nav className="menu-items" aria-label="Application menu">
        <div
          className={`menu-dropdown ${toolsOpen ? "is-open" : ""}`}
          ref={menuRef}
          onMouseEnter={openTools}
          onMouseLeave={closeToolsSoon}
          onKeyDown={handleMenuKeyDown}
        >
          <button type="button" className="menu-dropdown-trigger" onClick={openTools} onFocus={openTools} aria-expanded={toolsOpen}>
            {t("menu.file")}
            <ChevronDown size={13} />
          </button>
          <div className="menu-dropdown-panel">
            <button type="button" data-menu-item onClick={() => runMenuAction(() => setActiveSection("settings"))}>{t("nav.settings")}</button>
            <button type="button" data-menu-item onClick={() => runMenuAction(() => setActiveSection("workspace"))}>{t("nav.workspace")}</button>
            <div className="menu-divider" />
            <button type="button" data-menu-item onClick={() => runMenuAction(() => setActiveSection("system"))}>{t("nav.system")}</button>
            <button type="button" data-menu-item onClick={() => runMenuAction(() => setActiveSection("console"))}>{t("nav.console")}</button>
          </div>
        </div>
        <button type="button" onClick={() => setActiveSection("chats")} style={{ border: 0, padding: 0, background: "transparent", color: "inherit", font: "inherit", cursor: "pointer" }}>{t("menu.edit")}</button>
        <button type="button" onClick={() => setActiveSection("reports")} style={{ border: 0, padding: 0, background: "transparent", color: "inherit", font: "inherit", cursor: "pointer" }}>{t("menu.view")}</button>
        <div
          className={`menu-dropdown ${languageOpen ? "is-open" : ""}`}
          onMouseEnter={() => setLanguageOpen(true)}
          onMouseLeave={() => setLanguageOpen(false)}
        >
          <button type="button" className="menu-dropdown-trigger" onClick={() => setLanguageOpen(true)}>
            {t("menu.help")}
            <ChevronDown size={13} />
          </button>
          <div className="menu-dropdown-panel">
            <button type="button" data-menu-item onClick={() => runMenuAction(() => changeLanguage("zh-CN"))}>{t("language.zh")}</button>
            <button type="button" data-menu-item onClick={() => runMenuAction(() => changeLanguage("en-US"))}>{t("language.en")}</button>
            <div className="menu-divider" />
            <button type="button" data-menu-item onClick={() => runMenuAction(() => setAboutOpen(true))}>{t("about.title")}</button>
          </div>
        </div>
      </nav>
      <div className="top-actions">
        <div className="language-menu compact-language" aria-label={t("actions.language")}>
          <Languages size={15} />
          <span>{language}</span>
        </div>
        <Button
          variant="ghost"
          icon={theme === "dark" ? <Moon size={15} /> : <Sun size={15} />}
          title={t("actions.theme")}
          onClick={toggleTheme}
        >
          {theme === "dark" ? t("theme.dark") : t("theme.light")}
        </Button>
        <Button variant="ghost" icon={<Info size={15} />} title={t("about.title")} onClick={() => setAboutOpen(true)} />
      </div>
    </header>
  );
}
