use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{MemoryContext, ReportHistoryRecord};

#[derive(Debug, Clone)]
pub struct MemoryService {
    root: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurbineProfile {
    pub turbine_id: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FaultHistoryRecord {
    pub fault_id: String,
    pub turbine_id: String,
    pub component: String,
    pub symptom: String,
    pub risk_level: String,
    pub created_at: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceHistoryRecord {
    pub record_id: String,
    pub turbine_id: String,
    pub component: String,
    pub maintenance_action: String,
    pub performed_at: String,
    #[serde(default)]
    pub notes: Option<String>,
}

impl MemoryService {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn default_root() -> PathBuf {
        if let Ok(path) = std::env::var("CLAW_RAG_MEMORY_DIR") {
            let path = PathBuf::from(path);
            if !path.as_os_str().is_empty() {
                return path;
            }
        }
        let relative = PathBuf::from("beifeng").join("memory");
        if relative.is_dir() {
            return relative;
        }
        if let Ok(cwd) = std::env::current_dir() {
            for ancestor in cwd.ancestors().take(6) {
                let candidate = ancestor.join(&relative);
                if candidate.is_dir() {
                    return candidate;
                }
            }
        }
        relative
    }

    pub fn load_turbine_history(
        &self,
        turbine_id: Option<&str>,
        component: Option<&str>,
        symptom: Option<&str>,
    ) -> Result<MemoryContext, String> {
        let Some(turbine_id) = turbine_id.and_then(non_empty) else {
            return Ok(MemoryContext::default());
        };
        if turbine_id.eq_ignore_ascii_case("UNKNOWN") {
            return Ok(MemoryContext::default());
        }

        let component = component.and_then(non_empty);
        let symptom = symptom.and_then(non_empty);
        let faults = self.load_fault_history()?;
        let maintenance = self.load_maintenance_history()?;
        let reports = self.load_report_history()?;

        let mut matching_faults = faults
            .into_iter()
            .filter(|record| record.turbine_id == turbine_id)
            .filter(|record| {
                component
                    .map(|value| record.component.eq_ignore_ascii_case(value))
                    .unwrap_or(true)
            })
            .filter(|record| {
                symptom
                    .map(|value| same_symptom(&record.symptom, value))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        matching_faults.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let mut matching_maintenance = maintenance
            .into_iter()
            .filter(|record| record.turbine_id == turbine_id)
            .filter(|record| {
                component
                    .map(|value| record.component.eq_ignore_ascii_case(value))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        matching_maintenance.sort_by(|a, b| b.performed_at.cmp(&a.performed_at));

        let previous_reports = reports
            .into_iter()
            .filter(|record| record.turbine_id == turbine_id)
            .map(|record| record.report_path)
            .take(5)
            .collect::<Vec<_>>();

        let last_fault_time = matching_faults
            .first()
            .map(|record| record.created_at.clone());
        let last_risk_level = matching_faults
            .first()
            .map(|record| record.risk_level.clone());
        let short_term_recurrence =
            !matching_faults.is_empty() && recent_maintenance_exists(&matching_maintenance);

        Ok(MemoryContext {
            turbine_id: Some(turbine_id.to_string()),
            component: component.map(str::to_string),
            symptom: symptom.map(str::to_string),
            historical_fault_count: matching_faults.len(),
            last_fault_time,
            last_risk_level,
            maintenance_count: matching_maintenance.len(),
            previous_reports,
            short_term_recurrence,
        })
    }

    pub fn load_fault_history(&self) -> Result<Vec<FaultHistoryRecord>, String> {
        read_json_array(&self.path("fault_history.json"))
    }

    pub fn load_maintenance_history(&self) -> Result<Vec<MaintenanceHistoryRecord>, String> {
        read_json_array(&self.path("maintenance_history.json"))
    }

    pub fn load_report_history(&self) -> Result<Vec<ReportHistoryRecord>, String> {
        read_json_array(&self.path("report_history.json"))
    }

    pub fn load_turbine_profiles(&self) -> Result<Vec<TurbineProfile>, String> {
        read_json_array(&self.path("turbine_profiles.json"))
    }

    pub fn append_fault_record(&self, record: &FaultHistoryRecord) -> Result<(), String> {
        append_json_record(&self.path("fault_history.json"), record)
    }

    pub fn append_maintenance_record(
        &self,
        record: &MaintenanceHistoryRecord,
    ) -> Result<(), String> {
        append_json_record(&self.path("maintenance_history.json"), record)
    }

    pub fn append_report_record(&self, record: &ReportHistoryRecord) -> Result<(), String> {
        append_json_record(&self.path("report_history.json"), record)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

fn read_json_array<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn append_json_record<T>(path: &Path, record: &T) -> Result<(), String>
where
    T: Serialize + DeserializeOwned + Clone,
{
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let mut records: Vec<T> = read_json_array(path)?;
    records.push(record.clone());
    let raw = serde_json::to_string_pretty(&records)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    std::fs::write(path, format!("{raw}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn same_symptom(record_symptom: &str, current_symptom: &str) -> bool {
    record_symptom.eq_ignore_ascii_case(current_symptom)
        || record_symptom.contains(current_symptom)
        || current_symptom.contains(record_symptom)
}

fn recent_maintenance_exists(records: &[MaintenanceHistoryRecord]) -> bool {
    records
        .first()
        .map(|record| !record.performed_at.trim().is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_context_and_appends_records() {
        let dir = tempdir().unwrap();
        let service = MemoryService::new(dir.path().to_path_buf());
        service
            .append_fault_record(&FaultHistoryRecord {
                fault_id: "FH-1".to_string(),
                turbine_id: "A001".to_string(),
                component: "Blade".to_string(),
                symptom: "叶片裂纹".to_string(),
                risk_level: "Medium".to_string(),
                created_at: "2026-03".to_string(),
                query: None,
                notes: None,
            })
            .unwrap();
        service
            .append_maintenance_record(&MaintenanceHistoryRecord {
                record_id: "MH-1".to_string(),
                turbine_id: "A001".to_string(),
                component: "Blade".to_string(),
                maintenance_action: "裂纹修补".to_string(),
                performed_at: "2026-04".to_string(),
                notes: None,
            })
            .unwrap();
        let context = service
            .load_turbine_history(Some("A001"), Some("Blade"), Some("叶片裂纹"))
            .unwrap();
        assert_eq!(context.historical_fault_count, 1);
        assert_eq!(context.maintenance_count, 1);
        assert!(context.short_term_recurrence);
    }
}
