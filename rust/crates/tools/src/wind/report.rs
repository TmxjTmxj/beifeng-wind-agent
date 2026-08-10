use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::fault::{execute_wind_fault_analysis, value_string_array};
use super::{WindFaultAnalysisInput, WindReportGenerateInput};

pub(super) fn execute_wind_report_generate(
    input: &WindReportGenerateInput,
) -> Result<Value, String> {
    let fault_analysis = execute_wind_fault_analysis(&WindFaultAnalysisInput {
        problem: input.problem.clone(),
        component: input.component.clone(),
        symptom: input.symptom.clone(),
    })?;
    let report_dir = default_wind_report_dir()?;
    std::fs::create_dir_all(&report_dir)
        .map_err(|error| format!("create reports dir {}: {error}", report_dir.display()))?;
    let (report_path, generated_time) = unique_report_path(&report_dir);
    let report_markdown = wind_report_markdown(input, &fault_analysis, &generated_time);
    std::fs::write(&report_path, &report_markdown)
        .map_err(|error| format!("write report {}: {error}", report_path.display()))?;

    Ok(json!({
        "report_path": workspace_relative_report_path(&report_path),
        "report_markdown": report_markdown,
        "fault_analysis": fault_analysis
    }))
}

pub(crate) fn wind_report_markdown(
    input: &WindReportGenerateInput,
    fault_analysis: &Value,
    generated_time: &str,
) -> String {
    let report_type = normalize_report_type(input.report_type.as_deref());
    let title = input
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("风力发电运维报告");
    let component = input.component.as_deref().unwrap_or("unknown");
    let symptom = input.symptom.as_deref().unwrap_or("unknown");
    let evidence = fault_analysis
        .get("evidence_summary")
        .unwrap_or(&Value::Null);

    format!(
        "# 风力发电运维报告\n\n\
## 1. 基本信息\n\n\
- 报告标题：{title}\n\
- 报告类型：{report_type}\n\
- 生成时间：{generated_time}\n\
- 问题描述：{}\n\
- Component：{component}\n\
- Symptom：{symptom}\n\n\
## 2. 问题判断\n\n{}\n\n\
## 3. 可能原因\n\n{}\n\n\
## 4. 建议检查项目\n\n{}\n\n\
## 5. 建议检测方式\n\n{}\n\n\
## 6. 建议复检周期\n\n{}\n\n\
## 7. 维修建议\n\n{}\n\n\
## 8. 风险评估\n\n\
- 风险等级：{}\n\
- 是否需要停机评估：{}\n\
- 是否需要人工确认：{}\n\n\
## 9. 安全提示\n\n{}\n\n\
## 10. 证据来源\n\n\
### 命中文档\n\n{}\n\n\
### 图谱节点\n\n{}\n\n\
### 风险评估依据\n\n- 风险等级：{}\n\n\
## 11. 缺失数据\n\n{}\n\n\
## 12. 置信度\n\n{:.2}\n\n\
## 13. 免责声明\n\n\
本报告由 Wind O&M Agent 基于知识库、故障图谱和规则化风险评估生成，仅用于辅助运维决策。\n\n\
本报告不能替代：\n\n\
- 现场工程师判断\n\
- 厂家技术文件\n\
- 工作票制度\n\
- 调度规程\n\
- 安全规程\n\n\
涉及：\n\n\
- 高压电\n\
- 吊装\n\
- 远程停机\n\
- 远程复位\n\
- 并网操作\n\n\
必须遵循现场审批流程和人工确认要求。\n",
        input.problem,
        fault_analysis
            .get("problem_summary")
            .and_then(Value::as_str)
            .unwrap_or("不确定，需要补充数据。"),
        markdown_value_list(fault_analysis.get("possible_causes")),
        markdown_value_list(fault_analysis.get("inspection_items")),
        markdown_value_list(fault_analysis.get("inspection_methods")),
        fault_analysis
            .get("recommended_interval")
            .and_then(Value::as_str)
            .unwrap_or("不确定，需要补充数据。"),
        markdown_value_list(fault_analysis.get("maintenance_actions")),
        fault_analysis
            .get("risk_level")
            .and_then(Value::as_str)
            .unwrap_or("Unknown"),
        yes_no_value(fault_analysis.get("shutdown_evaluation_required")),
        yes_no_value(fault_analysis.get("human_confirmation_required")),
        markdown_value_list(fault_analysis.get("safety_notes")),
        markdown_value_list(evidence.get("hit_documents")),
        markdown_value_list(evidence.get("graph_nodes")),
        evidence
            .get("risk_level")
            .and_then(Value::as_str)
            .or_else(|| fault_analysis.get("risk_level").and_then(Value::as_str))
            .unwrap_or("Unknown"),
        markdown_value_list(fault_analysis.get("missing_data")),
        fault_analysis
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    )
}

fn normalize_report_type(value: Option<&str>) -> &'static str {
    match value.unwrap_or("inspection_report") {
        "fault_report" => "fault_report",
        "maintenance_advice" => "maintenance_advice",
        "risk_assessment_report" => "risk_assessment_report",
        _ => "inspection_report",
    }
}

fn markdown_value_list(value: Option<&Value>) -> String {
    let items = value_string_array(value);
    if items.is_empty() {
        return "- 不确定，需要补充数据。".to_string();
    }
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn yes_no_value(value: Option<&Value>) -> &'static str {
    if value.and_then(Value::as_bool).unwrap_or(false) {
        "是"
    } else {
        "否"
    }
}

fn workspace_relative_report_path(path: &Path) -> String {
    project_root_for_wind_paths()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn default_wind_report_dir() -> Result<PathBuf, String> {
    Ok(project_root_for_wind_paths()?
        .join("beifeng")
        .join("reports")
        .join("generated"))
}

fn project_root_for_wind_paths() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    for ancestor in cwd.ancestors().take(6) {
        if ancestor.join("beifeng").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Ok(cwd)
}

fn unique_report_path(report_dir: &Path) -> (PathBuf, String) {
    let base_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    for offset in 0..3_600 {
        let timestamp = report_timestamp(base_seconds + offset);
        let path = report_dir.join(format!("wind_report_{timestamp}.md"));
        if !path.exists() {
            return (path, timestamp);
        }
    }
    let timestamp = report_timestamp(base_seconds + 3_600);
    (
        report_dir.join(format!("wind_report_{timestamp}.md")),
        timestamp,
    )
}

fn report_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_epoch() {
        // 1970-01-01 = day 0
        let (y, m, d) = civil_from_days(0);
        assert_eq!(y, 1970);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
    }

    #[test]
    fn civil_from_days_known_date() {
        // 2026-06-04 — verify against a known date
        // Days from 1970-01-01 to 2026-06-04 ≈ 20628
        let (y, m, d) = civil_from_days(20_628);
        assert_eq!(y, 2026);
        assert_eq!(m, 6);
        assert_eq!(d, 4);
    }

    #[test]
    fn report_timestamp_format() {
        let ts = report_timestamp(20_628 * 86_400 + 10 * 3_600 + 30 * 60 + 45);
        assert!(ts.starts_with("20260604_"));
        assert!(ts.contains("103045"));
    }

    #[test]
    fn normalize_report_type_defaults() {
        assert_eq!(normalize_report_type(None), "inspection_report");
        assert_eq!(normalize_report_type(Some("fault_report")), "fault_report");
        assert_eq!(
            normalize_report_type(Some("maintenance_advice")),
            "maintenance_advice"
        );
        assert_eq!(
            normalize_report_type(Some("risk_assessment_report")),
            "risk_assessment_report"
        );
        assert_eq!(
            normalize_report_type(Some("unknown_type")),
            "inspection_report"
        );
    }

    #[test]
    fn markdown_value_list_empty() {
        let result = markdown_value_list(None);
        assert!(result.contains("不确定"));
    }

    #[test]
    fn markdown_value_list_items() {
        let value = Some(&json!(["item1", "item2"]));
        let result = markdown_value_list(value);
        assert!(result.contains("- item1"));
        assert!(result.contains("- item2"));
    }

    #[test]
    fn yes_no_value_bools() {
        assert_eq!(yes_no_value(Some(&json!(true))), "是");
        assert_eq!(yes_no_value(Some(&json!(false))), "否");
        assert_eq!(yes_no_value(None), "否");
    }

    #[test]
    fn wind_report_markdown_contains_sections() {
        let input = WindReportGenerateInput {
            problem: "test problem".to_string(),
            component: Some("Blade".to_string()),
            symptom: Some("裂纹".to_string()),
            report_type: None,
            title: None,
        };
        let fault_analysis = json!({
            "problem_summary": "test summary",
            "possible_causes": ["cause1"],
            "inspection_items": ["item1"],
            "inspection_methods": ["method1"],
            "recommended_interval": "30天",
            "maintenance_actions": ["action1"],
            "risk_level": "Medium",
            "shutdown_evaluation_required": false,
            "human_confirmation_required": true,
            "safety_notes": ["注意安全"],
            "evidence_summary": {
                "hit_documents": ["doc1.md"],
                "graph_nodes": ["node1"],
                "risk_level": "Medium"
            },
            "missing_data": [],
            "confidence": 0.75
        });
        let md = wind_report_markdown(&input, &fault_analysis, "20260604_103045");
        assert!(md.contains("# 风力发电运维报告"));
        assert!(md.contains("## 1. 基本信息"));
        assert!(md.contains("## 8. 风险评估"));
        assert!(md.contains("## 13. 免责声明"));
        assert!(md.contains("test problem"));
        assert!(md.contains("Medium"));
        assert!(md.contains("否")); // shutdown_evaluation_required = false
        assert!(md.contains("是")); // human_confirmation_required = true
        assert!(md.contains("0.75")); // confidence
    }
}
