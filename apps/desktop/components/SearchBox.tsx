import { Search } from "lucide-react";
import { useI18n } from "../store/i18n";

export function SearchBox({ value = "", onChange }: { value?: string; onChange?: (value: string) => void }) {
  const { t } = useI18n();
  return (
    <label className="search-box">
      <Search size={15} />
      <input
        aria-label={t("actions.search")}
        placeholder={t("actions.search")}
        value={value}
        onChange={(event) => onChange?.(event.target.value)}
      />
    </label>
  );
}
