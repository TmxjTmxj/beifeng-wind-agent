import { useEffect, useState } from "react";
import { BenchmarkPage } from "../pages/BenchmarkPage";
import { AgentConsolePage } from "../pages/AgentConsolePage";
import { ChatPage } from "../pages/ChatPage";
import { ConnectorsPage } from "../pages/ConnectorsPage";
import { FilesPage } from "../pages/FilesPage";
import { HomePage } from "../pages/HomePage";
import { MemoryPage } from "../pages/MemoryPage";
import { ReportsPage } from "../pages/ReportsPage";
import { SettingsPage } from "../pages/SettingsPage";
import { SkillsPage } from "../pages/SkillsPage";
import { SystemPage } from "../pages/SystemPage";
import { WorkspacePage } from "../pages/WorkspacePage";
import { useAppState } from "../store/appState";

export function Workbench() {
  const { activeSection } = useAppState();
  const [chatMounted, setChatMounted] = useState(activeSection === "chats");
  const isHome = activeSection === "home";
  const isChat = activeSection === "chats";
  const shouldRenderChat = chatMounted || isChat;

  useEffect(() => {
    if (isChat) {
      setChatMounted(true);
    }
  }, [isChat]);

  // Chat page has its own header. Keep it mounted after first visit so
  // in-progress conversations and message anchors survive section switches.
  return (
    <>
      {shouldRenderChat ? (
        <section
          className="workbench"
          style={{ display: isChat ? "grid" : "none", gridTemplateRows: "minmax(0, 1fr)" }}
        >
          <ChatPage />
        </section>
      ) : null}
      {isHome ? (
        <section className="workbench workbench-home">
          <HomePage />
        </section>
      ) : null}
      {!isHome && !isChat ? (
        <section className="workbench" style={{ gridTemplateRows: "minmax(0, 1fr)" }}>
          {activeSection === "workspace" ? <WorkspacePage /> : null}
          {activeSection === "files" ? <FilesPage /> : null}
          {activeSection === "memory" ? <MemoryPage /> : null}
          {activeSection === "reports" ? <ReportsPage /> : null}
          {activeSection === "benchmark" ? <BenchmarkPage /> : null}
          {activeSection === "skills" ? <SkillsPage /> : null}
          {activeSection === "connectors" ? <ConnectorsPage /> : null}
          {activeSection === "system" ? <SystemPage /> : null}
          {activeSection === "console" ? <AgentConsolePage /> : null}
          {activeSection === "settings" ? <SettingsPage /> : null}
        </section>
      ) : null}
    </>
  );
}
