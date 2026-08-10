use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticLevel {
    Ok,
    Warn,
    Fail,
}

impl DiagnosticLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }

    fn is_failure(self) -> bool {
        matches!(self, Self::Fail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticCheck {
    pub(crate) name: &'static str,
    pub(crate) level: DiagnosticLevel,
    pub(crate) summary: String,
    pub(crate) details: Vec<String>,
    pub(crate) data: Map<String, Value>,
    /// Stable remediation hint for warn/fail checks so automation can read
    /// a structured field instead of parsing details prose.
    pub(crate) hint: Option<String>,
}

impl DiagnosticCheck {
    pub(crate) fn new(
        name: &'static str,
        level: DiagnosticLevel,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            name,
            level,
            summary: summary.into(),
            details: Vec::new(),
            data: Map::new(),
            hint: None,
        }
    }

    pub(crate) fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    pub(crate) fn with_data(mut self, data: Map<String, Value>) -> Self {
        self.data = data;
        self
    }

    pub(crate) fn with_hint(mut self, hint: impl Into<String>) -> Self {
        let h = hint.into();
        if !h.is_empty() {
            self.hint = Some(h);
        }
        self
    }

    fn json_value(&self) -> Value {
        let id = self
            .name
            .to_ascii_lowercase()
            .replace(' ', "_")
            .replace('-', "_");
        let mut value = Map::from_iter([
            ("id".to_string(), Value::String(id.clone())),
            (
                "name".to_string(),
                Value::String(self.name.to_ascii_lowercase()),
            ),
            (
                "status".to_string(),
                Value::String(self.level.label().to_string()),
            ),
            ("summary".to_string(), Value::String(self.summary.clone())),
            (
                "details".to_string(),
                Value::Array(
                    self.details
                        .iter()
                        .map(detail_entry_json)
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                "details_prose".to_string(),
                Value::String(self.details.join("\n")),
            ),
            ("data".to_string(), Value::Object(self.data.clone())),
            (
                "kind".to_string(),
                Value::String("doctor_check".to_string()),
            ),
        ]);
        if let Some(hint) = &self.hint {
            value.insert("hint".to_string(), Value::String(hint.clone()));
        }
        value.extend(self.data.clone());
        value.insert("check_id".to_string(), Value::String(id));
        Value::Object(value)
    }
}

fn detail_entry_json(detail: &String) -> Value {
    let (key, value) = split_detail_entry(detail);
    json!({
        "key": key,
        "value": value,
    })
}

fn split_detail_entry(detail: &str) -> (String, String) {
    if let Some((key, value)) = detail.split_once("  ") {
        let key = key.trim();
        let value = value.trim();
        if !key.is_empty() && !value.is_empty() {
            return (
                key.to_string(),
                value.split_whitespace().collect::<Vec<_>>().join(" "),
            );
        }
    }
    (detail.trim().to_string(), String::new())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DoctorReport {
    pub(crate) checks: Vec<DiagnosticCheck>,
}

impl DoctorReport {
    fn counts(&self) -> (usize, usize, usize) {
        (
            self.checks
                .iter()
                .filter(|check| check.level == DiagnosticLevel::Ok)
                .count(),
            self.checks
                .iter()
                .filter(|check| check.level == DiagnosticLevel::Warn)
                .count(),
            self.checks
                .iter()
                .filter(|check| check.level == DiagnosticLevel::Fail)
                .count(),
        )
    }

    pub(crate) fn has_failures(&self) -> bool {
        self.checks.iter().any(|check| check.level.is_failure())
    }

    fn status(&self) -> &'static str {
        let (_, warn_count, fail_count) = self.counts();
        if fail_count > 0 {
            "fail"
        } else if warn_count > 0 {
            "warn"
        } else {
            "ok"
        }
    }

    pub(crate) fn render(&self) -> String {
        let (ok_count, warn_count, fail_count) = self.counts();
        let mut lines = vec![
            "Doctor".to_string(),
            format!(
                "Summary\n  OK               {ok_count}\n  Warnings         {warn_count}\n  Failures         {fail_count}"
            ),
        ];
        lines.extend(self.checks.iter().map(render_diagnostic_check));
        lines.join("\n\n")
    }

    pub(crate) fn json_value(&self) -> Value {
        let report = self.render();
        let (ok_count, warn_count, fail_count) = self.counts();
        json!({
            "kind": "doctor",
            "action": "doctor",
            "status": self.status(),
            "message": report,
            "report": report,
            "has_failures": self.has_failures(),
            "summary": {
                "total": self.checks.len(),
                "ok": ok_count,
                "warnings": warn_count,
                "failures": fail_count,
            },
            "checks": self
                .checks
                .iter()
                .map(DiagnosticCheck::json_value)
                .collect::<Vec<_>>(),
        })
    }
}

fn render_diagnostic_check(check: &DiagnosticCheck) -> String {
    let mut lines = vec![format!(
        "{}\n  Status           {}\n  Summary          {}",
        check.name,
        check.level.label(),
        check.summary
    )];
    if !check.details.is_empty() {
        lines.push("  Details".to_string());
        lines.extend(check.details.iter().map(|detail| format!("    - {detail}")));
    }
    lines.join("\n")
}
