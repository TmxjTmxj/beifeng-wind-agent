import logoDark from "../assets/branding/logo-dark.svg";
import { Button } from "./Button";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

const capabilities = ["RAG", "Memory Runtime", "Knowledge Graph", "Risk Engine", "Report Generation", "Connector Framework"];

export function AboutDialog() {
  const { aboutOpen, setAboutOpen } = useAppState();
  const { t } = useI18n();
  if (!aboutOpen) return null;

  return (
    <div className="about-backdrop" role="presentation" onClick={() => setAboutOpen(false)}>
      <section className="about-dialog" role="dialog" aria-modal="true" aria-labelledby="about-title" onClick={(event) => event.stopPropagation()}>
        <div className="about-header">
          <img src={logoDark} alt="" />
          <div>
            <h2 id="about-title">{t("about.title")}</h2>
            <p>{t("about.subtitle")}</p>
          </div>
        </div>
        <div className="about-body">
          <strong>{t("about.version")}: 1.0</strong>
          <span>{t("about.capabilities")}</span>
          <ul>
            {capabilities.map((capability) => <li key={capability}>{capability}</li>)}
          </ul>
        </div>
        <div className="about-actions">
          <Button variant="primary" onClick={() => setAboutOpen(false)}>{t("actions.close")}</Button>
        </div>
      </section>
    </div>
  );
}
