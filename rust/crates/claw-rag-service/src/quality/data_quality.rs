use serde::{Deserialize, Serialize};

use crate::ScadaDataPoint;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataQualitySeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataQualityIssueKind {
    MissingValue,
    Outlier,
    TimestampError,
    DuplicateData,
    UnitError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataQualityIssue {
    pub kind: DataQualityIssueKind,
    pub severity: DataQualitySeverity,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataQualityReport {
    pub score: u8,
    pub issues: Vec<DataQualityIssue>,
    pub human_confirmation_required: bool,
    pub confidence_multiplier: f32,
}

impl DataQualityReport {
    #[must_use]
    pub fn clean() -> Self {
        Self {
            score: 100,
            issues: Vec::new(),
            human_confirmation_required: false,
            confidence_multiplier: 1.0,
        }
    }
}

pub fn assess_records<T>(records: &[T]) -> DataQualityReport {
    if records.is_empty() {
        return report(vec![DataQualityIssue {
            kind: DataQualityIssueKind::MissingValue,
            severity: DataQualitySeverity::Warning,
            field: "records".to_string(),
            message: "connector returned no records".to_string(),
        }]);
    }
    DataQualityReport::clean()
}

pub fn assess_scada_points(points: &[ScadaDataPoint]) -> DataQualityReport {
    let mut issues = Vec::new();
    if points.is_empty() {
        issues.push(DataQualityIssue {
            kind: DataQualityIssueKind::MissingValue,
            severity: DataQualitySeverity::Warning,
            field: "scada_points".to_string(),
            message: "SCADA connector returned no points".to_string(),
        });
        return report(issues);
    }

    let mut seen = std::collections::BTreeSet::new();
    for point in points {
        if point.timestamp.trim().is_empty() {
            issues.push(issue(
                DataQualityIssueKind::TimestampError,
                DataQualitySeverity::Critical,
                "timestamp",
                "missing timestamp",
            ));
        }
        let key = format!("{}:{}", point.turbine_id, point.timestamp);
        if !seen.insert(key) {
            issues.push(issue(
                DataQualityIssueKind::DuplicateData,
                DataQualitySeverity::Warning,
                "timestamp",
                "duplicate turbine timestamp",
            ));
        }
        if point.wind_speed.is_none() && point.power.is_none() {
            issues.push(issue(
                DataQualityIssueKind::MissingValue,
                DataQualitySeverity::Warning,
                "wind_speed,power",
                "both wind speed and power are missing",
            ));
        }
        if point
            .wind_speed
            .is_some_and(|value| !(0.0..=80.0).contains(&value))
        {
            issues.push(issue(
                DataQualityIssueKind::UnitError,
                DataQualitySeverity::Critical,
                "wind_speed",
                "wind speed outside plausible m/s range",
            ));
        }
        if point.power.is_some_and(|value| value < -50.0) {
            issues.push(issue(
                DataQualityIssueKind::Outlier,
                DataQualitySeverity::Warning,
                "power",
                "power is unexpectedly negative",
            ));
        }
        if point
            .gearbox_oil_temp
            .is_some_and(|value| !(-40.0..=160.0).contains(&value))
        {
            issues.push(issue(
                DataQualityIssueKind::UnitError,
                DataQualitySeverity::Critical,
                "gearbox_oil_temp",
                "gearbox oil temperature outside plausible Celsius range",
            ));
        }
    }
    report(issues)
}

fn issue(
    kind: DataQualityIssueKind,
    severity: DataQualitySeverity,
    field: &str,
    message: &str,
) -> DataQualityIssue {
    DataQualityIssue {
        kind,
        severity,
        field: field.to_string(),
        message: message.to_string(),
    }
}

fn report(issues: Vec<DataQualityIssue>) -> DataQualityReport {
    let penalty = issues
        .iter()
        .map(|issue| match issue.severity {
            DataQualitySeverity::Info => 2,
            DataQualitySeverity::Warning => 8,
            DataQualitySeverity::Critical => 25,
        })
        .sum::<u16>();
    let score = 100_u16.saturating_sub(penalty).min(100) as u8;
    DataQualityReport {
        score,
        human_confirmation_required: score < 70
            || issues
                .iter()
                .any(|issue| issue.severity == DataQualitySeverity::Critical),
        confidence_multiplier: (f32::from(score) / 100.0).clamp(0.1, 1.0),
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scada_quality_flags_implausible_units() {
        let report = assess_scada_points(&[ScadaDataPoint {
            turbine_id: "T-01".to_string(),
            timestamp: "2026-06-05T10:00:00Z".to_string(),
            wind_speed: Some(180.0),
            power: Some(100.0),
            gearbox_oil_temp: None,
            generator_temp: None,
            vibration: None,
            alarm_codes: Vec::new(),
        }]);
        assert!(report.score < 100);
        assert!(report.human_confirmation_required);
    }
}
