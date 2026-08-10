import { ShieldAlert } from "lucide-react";
import { useI18n } from "../store/i18n";

export function RiskPanel() {
  const { t } = useI18n();
  return (
    <section className="risk-panel">
      <div className="risk-title">
        <ShieldAlert size={16} />
        <strong>{t("risk.high")}</strong>
        <span>Score: 0.82</span>
      </div>
      <div className="risk-meter" aria-label={t("inspector.riskLevel")}>
        <span style={{ width: "82%" }} />
      </div>
      <div className="risk-scale">
        <span>0</span>
        <span>0.33</span>
        <span>0.66</span>
        <span>1.00</span>
      </div>
      <p>Human confirmation required for safety-sensitive recommendations.</p>
    </section>
  );
}
