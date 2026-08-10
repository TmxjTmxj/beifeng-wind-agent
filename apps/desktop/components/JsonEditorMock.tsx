import { useI18n } from "../store/i18n";

export function JsonEditorMock({
  json,
  source,
  onChange
}: {
  json: string;
  source: string;
  onChange?: (value: string) => void;
}) {
  const { t } = useI18n();
  const lines = json.split("\n");

  return (
    <section className="json-editor">
      <div className="json-editor-toolbar">
        <span>{t("settings.editor")}</span>
        <span>
          {t("settings.source")}: {source}
        </span>
      </div>
      <div className="json-code">
        <div className="json-lines" aria-hidden="true">
          {lines.map((line, index) => (
            <span className="json-line-number" key={`${index}-${line}`}>
              {index + 1}
            </span>
          ))}
        </div>
        <textarea
          aria-label={t("settings.editor")}
          spellCheck={false}
          value={json}
          onChange={(event) => onChange?.(event.target.value)}
        />
      </div>
      <div className="json-code-fallback" aria-hidden="true">
        {false &&
          lines.map((line, index) => (
          <div className="json-line" key={`${index}-${line}`}>
            <span className="json-line-number">{index + 1}</span>
            <code>{line || " "}</code>
          </div>
        ))}
      </div>
    </section>
  );
}
