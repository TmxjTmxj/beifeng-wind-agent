//! Markdown report generation for Wind O&M fault analysis results.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::FaultAnalysisResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindReportGenerateInput {
    pub problem: String,
    pub component: Option<String>,
    pub symptom: Option<String>,
    pub report_type: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindReportGeneration {
    pub report_path: String,
    pub report_markdown: String,
    pub fault_analysis: FaultAnalysisResult,
}

pub fn generate_wind_report(
    input: &WindReportGenerateInput,
    fault_analysis: &FaultAnalysisResult,
    reports_dir: &Path,
) -> Result<WindReportGeneration, String> {
    std::fs::create_dir_all(reports_dir)
        .map_err(|e| format!("create reports dir {}: {e}", reports_dir.display()))?;
    let (report_path, generated_time) = unique_report_path(reports_dir, SystemTime::now());
    let report_markdown = build_wind_report_markdown(input, fault_analysis, &generated_time);
    std::fs::write(&report_path, &report_markdown)
        .map_err(|e| format!("write report {}: {e}", report_path.display()))?;
    Ok(WindReportGeneration {
        report_path: display_path(&report_path),
        report_markdown,
        fault_analysis: fault_analysis.clone(),
    })
}

pub fn build_wind_report_markdown(
    input: &WindReportGenerateInput,
    fault_analysis: &FaultAnalysisResult,
    generated_time: &str,
) -> String {
    let report_type = normalized_report_type(input.report_type.as_deref());
    let title = input
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("风力发电运维报告");
    let component = input.component.as_deref().unwrap_or("unknown");
    let symptom = input.symptom.as_deref().unwrap_or("unknown");

    format!(
        "# 风力发电运维报告\n\n\
## 1. 基本信息\n\n\
- 报告标题：{title}\n\
- 报告类型：{report_type}\n\
- 生成时间：{generated_time}\n\
- Component：{component}\n\
- Symptom：{symptom}\n\n\
## 2. 故障描述\n\n\
- 问题描述：{}\n\
- 故障摘要：{}\n\
{}\n\n\
## 3. 故障原因分析\n\n{}\n\n\
{}\n\n\
## 4. 检测建议\n\n\
### 建议检查项目\n\n{}\n\n\
### 建议检测方式\n\n{}\n\n\
## 5. 风险评估\n\n\
- 风险等级：{}\n\
- 是否需要停机评估：{}\n\
- 是否需要人工确认：{}\n\n\
## 6. 维护建议\n\n{}\n\n\
## 7. 安全注意事项\n\n{}\n\n\
## 8. 证据来源\n\n\
### 命中文档\n\n{}\n\n\
### 图谱节点\n\n{}\n\n\
### 风险评估依据\n\n- 风险等级：{}\n\n\
## 9. 缺失数据\n\n{}\n\n\
## 10. 建议复查周期\n\n{}\n\n\
## 11. 置信度\n\n{:.2}\n\n\
## 免责声明\n\n\
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
        fault_analysis.problem_summary,
        history_summary_markdown(fault_analysis.history_summary.as_deref()),
        markdown_list(&fault_analysis.possible_causes),
        relation_context_markdown(&fault_analysis.additional_context),
        markdown_list(&fault_analysis.inspection_items),
        markdown_list(&fault_analysis.inspection_methods),
        fault_analysis.risk_level,
        yes_no(fault_analysis.shutdown_evaluation_required),
        yes_no(fault_analysis.human_confirmation_required),
        markdown_list(&fault_analysis.maintenance_actions),
        markdown_list(&fault_analysis.safety_notes),
        markdown_list(&fault_analysis.evidence_summary.hit_documents),
        markdown_list(&fault_analysis.evidence_summary.graph_nodes),
        fault_analysis.evidence_summary.risk_level,
        markdown_list(&fault_analysis.missing_data),
        fault_analysis.recommended_interval,
        fault_analysis.confidence
    )
}

fn history_summary_markdown(summary: Option<&str>) -> String {
    summary
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("- 历史摘要：{value}"))
        .unwrap_or_default()
}

fn relation_context_markdown(items: &[String]) -> String {
    let relation_items = items
        .iter()
        .filter(|item| item.contains("故障升级风险") || item.contains("伴随症状"))
        .cloned()
        .collect::<Vec<_>>();
    if relation_items.is_empty() {
        "### 故障升级风险\n\n- 暂无明确升级路径，需要结合现场趋势持续复核。".to_string()
    } else {
        format!("### 故障升级风险\n\n{}", markdown_list(&relation_items))
    }
}

fn normalized_report_type(value: Option<&str>) -> &'static str {
    match value.unwrap_or("inspection_report") {
        "fault_report" => "fault_report",
        "maintenance_advice" => "maintenance_advice",
        "risk_assessment_report" => "risk_assessment_report",
        _ => "inspection_report",
    }
}

fn markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        return "- 不确定，需要补充数据。".to_string();
    }
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn unique_report_path(reports_dir: &Path, time: SystemTime) -> (PathBuf, String) {
    let base_seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    for offset in 0..3_600 {
        let generated_time = utc_report_timestamp(base_seconds + offset);
        let report_path = reports_dir.join(format!("wind_report_{generated_time}.md"));
        if !report_path.exists() {
            return (report_path, generated_time);
        }
    }
    let generated_time = utc_report_timestamp(base_seconds + 3_600);
    (
        reports_dir.join(format!("wind_report_{generated_time}.md")),
        generated_time,
    )
}

fn utc_report_timestamp(seconds: i64) -> String {
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

#[must_use]
pub fn default_reports_dir() -> PathBuf {
    let relative = PathBuf::from("beifeng").join("reports").join("generated");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvidenceSummary;

    fn fault_result(risk_level: &str) -> FaultAnalysisResult {
        FaultAnalysisResult {
            problem_summary: "发现叶片裂纹，需要复核缺陷范围。".to_string(),
            possible_causes: vec!["前缘冲蚀或外物冲击。".to_string()],
            inspection_items: vec!["裂纹长度".to_string(), "裂纹宽度".to_string()],
            inspection_methods: vec!["无人机复检".to_string(), "超声检测".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["建立跟踪工单".to_string()],
            additional_context: Vec::new(),
            history_summary: None,
            risk_level: risk_level.to_string(),
            shutdown_evaluation_required: true,
            human_confirmation_required: true,
            safety_notes: vec!["高空作业需工作票。".to_string()],
            evidence_summary: EvidenceSummary {
                hit_documents: vec!["knowledge_base/fault_cases/blade.md".to_string()],
                graph_nodes: vec!["Blade:叶片裂纹".to_string()],
                risk_level: risk_level.to_string(),
            },
            missing_data: vec!["裂纹位置".to_string()],
            confidence: 0.72,
        }
    }

    #[test]
    fn inspection_report_contains_risk_and_shutdown_evaluation() {
        let markdown = build_wind_report_markdown(
            &WindReportGenerateInput {
                problem: "叶片裂纹".to_string(),
                component: Some("Blade".to_string()),
                symptom: Some("裂纹".to_string()),
                report_type: Some("inspection_report".to_string()),
                title: Some("叶片裂纹巡检分析报告".to_string()),
            },
            &fault_result("Medium"),
            "20260603_213000",
        );
        assert!(markdown.contains("# 风力发电运维报告"));
        assert!(markdown.contains("风险等级：Medium"));
        assert!(markdown.contains("是否需要停机评估：是"));
        assert!(markdown.contains("## 免责声明"));
    }

    #[test]
    fn gearbox_fault_report_contains_high_risk_inspection_and_maintenance() {
        let markdown = build_wind_report_markdown(
            &WindReportGenerateInput {
                problem: "齿轮箱油温升高".to_string(),
                component: Some("Gearbox".to_string()),
                symptom: Some("油温升高".to_string()),
                report_type: Some("fault_report".to_string()),
                title: None,
            },
            &fault_result("High"),
            "20260603_213000",
        );
        assert!(markdown.contains("风险等级：High"));
        assert!(markdown.contains("## 4. 检测建议"));
        assert!(markdown.contains("## 6. 维护建议"));
    }

    #[test]
    fn risk_assessment_report_contains_human_confirmation() {
        let markdown = build_wind_report_markdown(
            &WindReportGenerateInput {
                problem: "功率曲线异常".to_string(),
                component: Some("SCADA".to_string()),
                symptom: Some("功率曲线异常".to_string()),
                report_type: Some("risk_assessment_report".to_string()),
                title: None,
            },
            &fault_result("Medium"),
            "20260603_213000",
        );
        assert!(markdown.contains("风险等级：Medium"));
        assert!(markdown.contains("是否需要人工确认：是"));
        assert!(markdown.contains("### 风险评估依据"));
    }
}
