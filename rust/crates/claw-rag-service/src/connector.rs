//! SCADA connector MVP for CSV-backed turbine telemetry.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScadaDataPoint {
    pub turbine_id: String,
    pub timestamp: String,
    pub wind_speed: Option<f64>,
    pub power: Option<f64>,
    pub gearbox_oil_temp: Option<f64>,
    pub generator_temp: Option<f64>,
    pub vibration: Option<f64>,
    pub alarm_codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScadaCsvConnector {
    pub data_dir: PathBuf,
}

impl ScadaCsvConnector {
    pub fn load_latest(&self, turbine_id: Option<&str>) -> Result<Vec<ScadaDataPoint>> {
        let mut points = Vec::new();
        if !self.data_dir.is_dir() {
            return Ok(points);
        }

        for entry in std::fs::read_dir(&self.data_dir)
            .with_context(|| format!("read SCADA dir {}", self.data_dir.display()))?
        {
            let path = entry?.path();
            if !is_csv(&path) {
                continue;
            }
            points.extend(read_scada_csv(&path, turbine_id)?);
        }

        points.sort_by(|a, b| {
            b.timestamp
                .cmp(&a.timestamp)
                .then_with(|| a.turbine_id.cmp(&b.turbine_id))
        });
        points.truncate(32);
        Ok(points)
    }

    pub fn to_context_summary(&self, points: &[ScadaDataPoint]) -> String {
        if points.is_empty() {
            return "SCADA未提供可用数据点。".to_string();
        }

        let mut lines = Vec::new();
        for point in points.iter().take(8) {
            let mut metrics = Vec::new();
            if let Some(value) = point.wind_speed {
                metrics.push(format!("风速{value:.1}m/s"));
            }
            if let Some(value) = point.power {
                metrics.push(format!("功率{value:.0}kW"));
            }
            if let Some(value) = point.gearbox_oil_temp {
                metrics.push(format!("齿轮箱油温{value:.1}C"));
            }
            if let Some(value) = point.generator_temp {
                metrics.push(format!("发电机温度{value:.1}C"));
            }
            if let Some(value) = point.vibration {
                metrics.push(format!("振动{value:.2}mm/s"));
            }
            if !point.alarm_codes.is_empty() {
                metrics.push(format!("报警{}", point.alarm_codes.join("/")));
            }

            let anomalies = scada_anomalies(point);
            let anomaly_text = if anomalies.is_empty() {
                "未见明显阈值异常".to_string()
            } else {
                format!("异常：{}", anomalies.join("；"))
            };
            lines.push(format!(
                "{} {}：{}；{}",
                point.turbine_id,
                point.timestamp,
                metrics.join("，"),
                anomaly_text
            ));
        }
        format!("SCADA最新摘要：{}", lines.join(" | "))
    }
}

fn read_scada_csv(path: &Path, turbine_id: Option<&str>) -> Result<Vec<ScadaDataPoint>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("open SCADA csv {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("read SCADA csv headers {}", path.display()))?
        .clone();
    let mut points = Vec::new();

    for record in reader.records() {
        let record = record.with_context(|| format!("read SCADA csv row {}", path.display()))?;
        let point = ScadaDataPoint {
            turbine_id: field(&headers, &record, "turbine_id").unwrap_or_default(),
            timestamp: field(&headers, &record, "timestamp").unwrap_or_default(),
            wind_speed: field_f64(&headers, &record, "wind_speed"),
            power: field_f64(&headers, &record, "power"),
            gearbox_oil_temp: field_f64(&headers, &record, "gearbox_oil_temp"),
            generator_temp: field_f64(&headers, &record, "generator_temp"),
            vibration: field_f64(&headers, &record, "vibration"),
            alarm_codes: split_alarm_codes(field(&headers, &record, "alarm_codes").as_deref()),
        };
        if point.turbine_id.is_empty() || point.timestamp.is_empty() {
            continue;
        }
        if turbine_id.is_none_or(|expected| expected == point.turbine_id) {
            points.push(point);
        }
    }
    Ok(points)
}

fn field(headers: &csv::StringRecord, record: &csv::StringRecord, name: &str) -> Option<String> {
    headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(name))
        .and_then(|index| record.get(index))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn field_f64(headers: &csv::StringRecord, record: &csv::StringRecord, name: &str) -> Option<f64> {
    field(headers, record, name).and_then(|value| value.parse::<f64>().ok())
}

fn split_alarm_codes(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .trim_matches(|c| c == '[' || c == ']')
        .split(|c| matches!(c, ';' | '|' | ','))
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn is_csv(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
}

fn scada_anomalies(point: &ScadaDataPoint) -> Vec<String> {
    let mut anomalies = Vec::new();
    if point.gearbox_oil_temp.is_some_and(|value| value >= 80.0) {
        anomalies.push("齿轮箱油温偏高".to_string());
    }
    if point.generator_temp.is_some_and(|value| value >= 95.0) {
        anomalies.push("发电机温度偏高".to_string());
    }
    if point.vibration.is_some_and(|value| value >= 8.0) {
        anomalies.push("振动超限".to_string());
    }
    if matches!((point.wind_speed, point.power), (Some(wind), Some(power)) if wind >= 7.0 && power < 500.0)
    {
        anomalies.push("风速正常但功率偏低".to_string());
    }
    if !point.alarm_codes.is_empty() {
        anomalies.push("存在SCADA报警".to_string());
    }
    anomalies
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_latest_scada_points_and_summarizes_anomalies() {
        let dir = tempdir().unwrap();
        let csv = dir.path().join("scada.csv");
        std::fs::write(
            &csv,
            "turbine_id,timestamp,wind_speed,power,gearbox_oil_temp,generator_temp,vibration,alarm_codes\n\
             T-01,2026-06-05T10:00:00Z,8.2,430,82,88,9.1,A01;A02\n\
             T-02,2026-06-05T09:00:00Z,6.0,1200,60,70,2.0,\n",
        )
        .unwrap();
        let connector = ScadaCsvConnector {
            data_dir: dir.path().to_path_buf(),
        };

        let points = connector.load_latest(Some("T-01")).unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].alarm_codes, vec!["A01", "A02"]);

        let summary = connector.to_context_summary(&points);
        assert!(summary.contains("齿轮箱油温偏高"));
        assert!(summary.contains("振动超限"));
    }
}
