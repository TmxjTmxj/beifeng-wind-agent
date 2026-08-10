import { useEffect, useMemo, useState } from "react";
import { BookOpen, Copy, ExternalLink, FolderOpen, RefreshCw, Search } from "lucide-react";
import { Button } from "../components/Button";
import { SkillSummary, desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";

function formatBytes(size: number) {
  if (size <= 0) return "-";
  if (size < 1024) return `${size} B`;
  return `${(size / 1024).toFixed(1)} KB`;
}

function matchesSkill(skill: SkillSummary, query: string) {
  const text = `${skill.name} ${skill.category} ${skill.description}`.toLowerCase();
  return text.includes(query.trim().toLowerCase());
}

export function SkillsPage() {
  const { t } = useI18n();
  const [skills, setSkills] = useState<SkillSummary[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState("");
  const [error, setError] = useState<string | null>(null);

  const selected = skills.find((skill) => skill.path === selectedPath) ?? skills[0] ?? null;

  const filtered = useMemo(
    () => skills.filter((skill) => matchesSkill(skill, query)),
    [skills, query],
  );

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const rows = await desktopApi.listSkills();
      setSkills(rows);
      setSelectedPath((current) => current && rows.some((row) => row.path === current) ? current : rows[0]?.path ?? null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const copyInvocation = async () => {
    if (!selected) return;
    await navigator.clipboard?.writeText(`$${selected.category}`);
    setMessage(`已复制 $${selected.category}`);
  };

  const openSkill = async (path?: string | null) => {
    if (!path) return;
    try {
      await desktopApi.openFile(path);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <main className="workbench-page skills-page">
      <header className="screen-header">
        <div>
          <h1>{t("nav.skills")}</h1>
          <p>{t("resource.skills.description")}</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<RefreshCw size={15} />} onClick={load} disabled={loading}>
            {loading ? "刷新中" : t("actions.reload")}
          </Button>
        </div>
      </header>

      {message ? <div className="settings-message compact">{message}</div> : null}
      {error ? <div className="settings-message compact is-error">{error}</div> : null}

      <section className="skills-toolbar">
        <label className="search-box">
          <Search size={15} />
          <input
            aria-label={t("actions.search")}
            placeholder="搜索技能名称、目录或说明"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>
        <span>{filtered.length} / {skills.length} skills</span>
      </section>

      <section className="skills-layout">
        <div className="skills-list" role="list">
          {filtered.map((skill) => (
            <button
              className={`panel-block skill-row ${selected?.path === skill.path ? "is-active" : ""}`}
              key={skill.path}
              onClick={() => setSelectedPath(skill.path)}
              type="button"
            >
              <strong>{skill.name}</strong>
              <span>{skill.category} · {skill.updated}</span>
              <p>{skill.description}</p>
            </button>
          ))}
          {!filtered.length ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
        </div>

        <aside className="panel-block skill-detail">
          {selected ? (
            <>
              <div className="panel-heading-row">
                <div>
                  <h2>{selected.name}</h2>
                  <p>{selected.category}</p>
                </div>
                <BookOpen size={18} />
              </div>
              <p>{selected.description}</p>
              <dl>
                <div><dt>Prompt</dt><dd title={selected.path}>{selected.path}</dd></div>
                <div><dt>Updated</dt><dd>{selected.updated}</dd></div>
                <div><dt>Size</dt><dd>{formatBytes(selected.size)}</dd></div>
                <div><dt>Examples</dt><dd>{selected.examples_path ? "Available" : "Missing"}</dd></div>
              </dl>
              <div className="header-actions">
                <Button icon={<ExternalLink size={14} />} onClick={() => openSkill(selected.path)}>打开</Button>
                <Button variant="ghost" icon={<FolderOpen size={14} />} onClick={() => desktopApi.revealInExplorer(selected.directory)}>定位</Button>
                <Button variant="ghost" icon={<BookOpen size={14} />} onClick={() => openSkill(selected.examples_path)} disabled={!selected.examples_path}>示例</Button>
                <Button variant="ghost" icon={<Copy size={14} />} onClick={copyInvocation}>复制调用</Button>
              </div>
            </>
          ) : (
            <p className="empty-copy">{t("inspector.empty")}</p>
          )}
        </aside>
      </section>
    </main>
  );
}
