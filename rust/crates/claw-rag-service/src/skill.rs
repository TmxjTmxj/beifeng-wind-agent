//! Skill dispatch and skill-specific output formatting.
//!
//! This module implements a lightweight skill-dispatch mechanism that matches
//! incoming queries to the most appropriate built-in Skill and then formats
//! the output with skill-specific Markdown templates.

use serde::{Deserialize, Serialize};

use crate::{FaultAnalysisResult, GraphSuggestion, WindInspectionAdvice, WindRiskAssessment};

// ---------------------------------------------------------------------------
// SkillType
// ---------------------------------------------------------------------------

/// Built-in skill types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    WindFaultAnalysis,
    BladeInspection,
    GearboxDiagnosis,
    ScadaAnalysis,
    ReportGeneration,
}

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillType::WindFaultAnalysis => write!(f, "wind_fault_analysis"),
            SkillType::BladeInspection => write!(f, "blade_inspection"),
            SkillType::GearboxDiagnosis => write!(f, "gearbox_diagnosis"),
            SkillType::ScadaAnalysis => write!(f, "scada_analysis"),
            SkillType::ReportGeneration => write!(f, "report_generation"),
        }
    }
}

// ---------------------------------------------------------------------------
// SkillDefinition
// ---------------------------------------------------------------------------

/// Skill definition with trigger rules and output template.
#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub skill_type: SkillType,
    pub trigger_components: Vec<&'static str>,
    pub trigger_keywords: Vec<&'static str>,
    pub priority: u8,
}

/// Get all built-in skill definitions ordered by descending priority.
pub fn builtin_skills() -> Vec<SkillDefinition> {
    vec![
        SkillDefinition {
            skill_type: SkillType::ReportGeneration,
            trigger_components: Vec::new(),
            trigger_keywords: vec!["报告", "生成报告", "报告生成"],
            priority: 90,
        },
        SkillDefinition {
            skill_type: SkillType::BladeInspection,
            trigger_components: vec!["Blade", "blade"],
            trigger_keywords: vec!["叶片", "裂纹", "桨叶", "巡检"],
            priority: 80,
        },
        SkillDefinition {
            skill_type: SkillType::GearboxDiagnosis,
            trigger_components: vec!["Gearbox", "gearbox"],
            trigger_keywords: vec!["齿轮箱", "油温", "振动"],
            priority: 80,
        },
        SkillDefinition {
            skill_type: SkillType::ScadaAnalysis,
            trigger_components: vec!["SCADA", "scada"],
            trigger_keywords: vec!["功率曲线", "SCADA", "报警"],
            priority: 80,
        },
        SkillDefinition {
            skill_type: SkillType::WindFaultAnalysis,
            trigger_components: Vec::new(),
            trigger_keywords: Vec::new(),
            priority: 10,
        },
    ]
}

// ---------------------------------------------------------------------------
// dispatch_skill
// ---------------------------------------------------------------------------

/// Dispatch the best matching skill for a query.
///
/// Matching priority:
/// 1. **Component match** – if `component` matches a skill's `trigger_components`.
/// 2. **Keyword match** – if `query` contains any of a skill's `trigger_keywords`.
/// 3. **Fallback** – `WindFaultAnalysis` is the default skill.
pub fn dispatch_skill(query: &str, component: Option<&str>) -> SkillType {
    let skills = builtin_skills();
    let query_lower = query.to_ascii_lowercase();

    // Phase 0: report-generation keywords override component match
    // If query explicitly asks for a report, always use ReportGeneration skill
    let report_keywords = ["报告", "生成报告", "报告生成", "report", "generate report"];
    if report_keywords
        .iter()
        .any(|kw| query_lower.contains(&kw.to_ascii_lowercase()))
    {
        return SkillType::ReportGeneration;
    }

    // Phase 1: component-based match (highest priority among equals)
    if let Some(comp) = component {
        let comp_trimmed = comp.trim();
        for skill in &skills {
            if skill
                .trigger_components
                .iter()
                .any(|c| c.eq_ignore_ascii_case(comp_trimmed))
            {
                return skill.skill_type.clone();
            }
        }
    }

    // Phase 2: keyword-based match – pick the skill with highest priority
    let mut best: Option<(&SkillDefinition, usize)> = None;
    for skill in &skills {
        if skill.trigger_keywords.is_empty() {
            continue;
        }
        let match_count = skill
            .trigger_keywords
            .iter()
            .filter(|kw| query_lower.contains(&kw.to_ascii_lowercase()))
            .count();
        if match_count == 0 {
            continue;
        }
        let is_better = match &best {
            None => true,
            Some((prev, prev_count)) => {
                if match_count > *prev_count {
                    true
                } else if match_count == *prev_count {
                    skill.priority > prev.priority
                } else {
                    false
                }
            }
        };
        if is_better {
            best = Some((skill, match_count));
        }
    }

    if let Some((skill, _)) = best {
        return skill.skill_type.clone();
    }

    // Phase 3: fallback
    SkillType::WindFaultAnalysis
}

// ---------------------------------------------------------------------------
// SkillQueryResponse
// ---------------------------------------------------------------------------

/// Response payload for the `/v1/skill-query` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillQueryResponse {
    pub skill_name: String,
    pub query: String,
    pub component: Option<String>,
    pub fault_analysis: FaultAnalysisResult,
    pub skill_output: String,
}

// ---------------------------------------------------------------------------
// format_skill_output
// ---------------------------------------------------------------------------

/// Format output based on skill type.
pub fn format_skill_output(
    skill_type: &SkillType,
    fault_analysis: &FaultAnalysisResult,
    advice: &WindInspectionAdvice,
    risk: &WindRiskAssessment,
    graph_suggestions: &[GraphSuggestion],
) -> String {
    match skill_type {
        SkillType::WindFaultAnalysis => {
            format_wind_fault_analysis(fault_analysis, advice, risk, graph_suggestions)
        }
        SkillType::BladeInspection => {
            format_blade_inspection(fault_analysis, advice, risk, graph_suggestions)
        }
        SkillType::GearboxDiagnosis => {
            format_gearbox_diagnosis(fault_analysis, advice, risk, graph_suggestions)
        }
        SkillType::ScadaAnalysis => {
            format_scada_analysis(fault_analysis, advice, risk, graph_suggestions)
        }
        SkillType::ReportGeneration => {
            format_report_generation(fault_analysis, advice, risk, graph_suggestions)
        }
    }
}

// ---------------------------------------------------------------------------
// Skill-specific Markdown formatters
// ---------------------------------------------------------------------------

/// General wind fault analysis – covers fault overview, possible causes,
/// inspection suggestions, risk assessment, safety notes, and evidence.
fn format_wind_fault_analysis(
    fault_analysis: &FaultAnalysisResult,
    _advice: &WindInspectionAdvice,
    risk: &WindRiskAssessment,
    graph_suggestions: &[GraphSuggestion],
) -> String {
    let graph_section = if graph_suggestions.is_empty() {
        "- 未命中故障图谱条目".to_string()
    } else {
        graph_suggestions
            .iter()
            .map(|g| format!("- {}: {}（风险 {}）", g.component, g.symptom, g.risk_level))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "# 风电故障分析\n\n\
         ## 1. 故障概述\n\n\
         {problem_summary}\n\n\
         ## 2. 可能原因\n\n\
         {possible_causes}\n\n\
         ## 3. 检测建议\n\n\
         ### 检查项目\n\n\
         {inspection_items}\n\n\
         ### 检测方式\n\n\
         {inspection_methods}\n\n\
         ## 4. 风险评估\n\n\
         - 风险等级：**{risk_level}**\n\
         - 需要停机评估：{shutdown_eval}\n\
         - 需要人工确认：{human_confirm}\n\
         - 升级要求：{escalation}\n\n\
         ## 5. 安全提示\n\n\
         {safety_notes}\n\n\
         ## 6. 证据来源\n\n\
         ### 命中文档\n\n\
         {hit_documents}\n\n\
         ### 图谱节点\n\n\
         {graph_section}\n\n\
         ### 缺失数据\n\n\
         {missing_data}\n\n\
         ## 置信度\n\n\
         {confidence:.2}\n",
        problem_summary = fault_analysis.problem_summary,
        possible_causes = md_list(&fault_analysis.possible_causes),
        inspection_items = md_list(&fault_analysis.inspection_items),
        inspection_methods = md_list(&fault_analysis.inspection_methods),
        risk_level = fault_analysis.risk_level,
        shutdown_eval = yes_no(fault_analysis.shutdown_evaluation_required),
        human_confirm = yes_no(fault_analysis.human_confirmation_required),
        escalation = yes_no(risk.escalation_required),
        safety_notes = md_list(&fault_analysis.safety_notes),
        hit_documents = md_list(&fault_analysis.evidence_summary.hit_documents),
        graph_section = graph_section,
        missing_data = md_list(&fault_analysis.missing_data),
        confidence = fault_analysis.confidence,
    )
}

/// Blade inspection – focuses on defect description, inspection methods
/// (UAV / high-res photography), repair suggestions, and re-inspection cycle.
fn format_blade_inspection(
    fault_analysis: &FaultAnalysisResult,
    advice: &WindInspectionAdvice,
    _risk: &WindRiskAssessment,
    graph_suggestions: &[GraphSuggestion],
) -> String {
    let defect_summary = graph_suggestions
        .first()
        .map(|g| format!("{}：{}", g.component, g.symptom))
        .unwrap_or_else(|| "叶片缺陷待确认".to_string());

    let uav_methods: Vec<String> = advice
        .inspection_methods
        .iter()
        .filter(|m| {
            let lower = m.to_ascii_lowercase();
            lower.contains("无人机")
                || lower.contains("uav")
                || lower.contains("高清")
                || lower.contains("摄影")
                || lower.contains("红外")
                || lower.contains("热成像")
        })
        .cloned()
        .collect();

    let other_methods: Vec<String> = advice
        .inspection_methods
        .iter()
        .filter(|m| {
            let lower = m.to_ascii_lowercase();
            !lower.contains("无人机")
                && !lower.contains("uav")
                && !lower.contains("高清")
                && !lower.contains("摄影")
                && !lower.contains("红外")
                && !lower.contains("热成像")
        })
        .cloned()
        .collect();

    format!(
        "# 叶片巡检报告\n\n\
         ## 1. 缺陷描述\n\n\
         {defect_summary}\n\n\
         {problem_summary}\n\n\
         ## 2. 检测方式\n\n\
         ### 无人机/高清摄影检测\n\n\
         {uav_methods}\n\n\
         ### 其他检测方式\n\n\
         {other_methods}\n\n\
         ## 3. 检查项目\n\n\
         {inspection_items}\n\n\
         ## 4. 修补建议\n\n\
         {maintenance_actions}\n\n\
         ## 5. 复检周期\n\n\
         {recommended_interval}\n\n\
         ## 6. 风险评估\n\n\
         - 风险等级：**{risk_level}**\n\
         - 需要停机评估：{shutdown_eval}\n\
         - 需要人工确认：{human_confirm}\n\n\
         ## 7. 安全注意事项\n\n\
         {safety_notes}\n\n\
         ## 8. 证据来源\n\n\
         {evidence_sources}\n\n\
         ## 9. 缺失数据\n\n\
         {missing_data}\n\n\
         ## 置信度\n\n\
         {confidence:.2}\n",
        defect_summary = defect_summary,
        problem_summary = fault_analysis.problem_summary,
        uav_methods = md_list_or_fallback(&uav_methods, "无人机复检"),
        other_methods = md_list_or_fallback(&other_methods, "超声检测、敲击检测"),
        inspection_items = md_list(&fault_analysis.inspection_items),
        maintenance_actions = md_list(&fault_analysis.maintenance_actions),
        recommended_interval = fault_analysis.recommended_interval,
        risk_level = fault_analysis.risk_level,
        shutdown_eval = yes_no(fault_analysis.shutdown_evaluation_required),
        human_confirm = yes_no(fault_analysis.human_confirmation_required),
        safety_notes = md_list(&fault_analysis.safety_notes),
        evidence_sources = md_list(&fault_analysis.evidence_summary.hit_documents),
        missing_data = md_list(&fault_analysis.missing_data),
        confidence = fault_analysis.confidence,
    )
}

/// Gearbox diagnosis – focuses on oil temperature, oil sample, vibration,
/// and lubrication troubleshooting steps.
fn format_gearbox_diagnosis(
    fault_analysis: &FaultAnalysisResult,
    _advice: &WindInspectionAdvice,
    risk: &WindRiskAssessment,
    graph_suggestions: &[GraphSuggestion],
) -> String {
    let symptom_summary = graph_suggestions
        .first()
        .map(|g| format!("{}：{}", g.component, g.symptom))
        .unwrap_or_else(|| "齿轮箱异常待确认".to_string());

    let mut diagnostic_steps = Vec::new();
    diagnostic_steps
        .push("1. **油温排查**：检查 SCADA 油温趋势，确认是否持续上升或接近保护阈值。".to_string());
    diagnostic_steps.push("2. **油样检查**：取样检测油品金属颗粒含量、粘度和水分。".to_string());
    diagnostic_steps
        .push("3. **振动分析**：查看振动频谱特征，关注齿轮啮合频率和轴承特征频率。".to_string());
    diagnostic_steps.push("4. **润滑状态**：确认油位、滤芯压差、冷却系统运行状态。".to_string());
    diagnostic_steps
        .push("5. **综合判断**：结合油温、油样、振动和润滑状态给出诊断结论。".to_string());

    let forbidden = risk
        .forbidden_actions
        .iter()
        .chain(risk.immediate_actions.iter())
        .cloned()
        .collect::<Vec<_>>();

    format!(
        "# 齿轮箱诊断报告\n\n\
         ## 1. 故障概述\n\n\
         {symptom_summary}\n\n\
         {problem_summary}\n\n\
         ## 2. 排查步骤\n\n\
         {diagnostic_steps}\n\n\
         ## 3. 检查项目\n\n\
         {inspection_items}\n\n\
         ## 4. 检测方式\n\n\
         {inspection_methods}\n\n\
         ## 5. 维护建议\n\n\
         {maintenance_actions}\n\n\
         ## 6. 风险评估\n\n\
         - 风险等级：**{risk_level}**\n\
         - 需要停机评估：{shutdown_eval}\n\
         - 需要人工确认：{human_confirm}\n\
         - 升级要求：{escalation}\n\n\
         ## 7. 紧急/禁止操作\n\n\
         {forbidden_actions}\n\n\
         ## 8. 复检周期\n\n\
         {recommended_interval}\n\n\
         ## 9. 证据来源\n\n\
         {evidence_sources}\n\n\
         ## 10. 缺失数据\n\n\
         {missing_data}\n\n\
         ## 置信度\n\n\
         {confidence:.2}\n",
        symptom_summary = symptom_summary,
        problem_summary = fault_analysis.problem_summary,
        diagnostic_steps = diagnostic_steps.join("\n"),
        inspection_items = md_list(&fault_analysis.inspection_items),
        inspection_methods = md_list(&fault_analysis.inspection_methods),
        maintenance_actions = md_list(&fault_analysis.maintenance_actions),
        risk_level = fault_analysis.risk_level,
        shutdown_eval = yes_no(fault_analysis.shutdown_evaluation_required),
        human_confirm = yes_no(fault_analysis.human_confirmation_required),
        escalation = yes_no(risk.escalation_required),
        forbidden_actions = md_list(&forbidden),
        recommended_interval = fault_analysis.recommended_interval,
        evidence_sources = md_list(&fault_analysis.evidence_summary.hit_documents),
        missing_data = md_list(&fault_analysis.missing_data),
        confidence = fault_analysis.confidence,
    )
}

/// SCADA data analysis – focuses on power curve anomalies, measurement errors,
/// power limiting, yaw errors, and supplementary data needs.
fn format_scada_analysis(
    fault_analysis: &FaultAnalysisResult,
    _advice: &WindInspectionAdvice,
    _risk: &WindRiskAssessment,
    graph_suggestions: &[GraphSuggestion],
) -> String {
    let mut possible_scada_causes = Vec::new();
    possible_scada_causes
        .push("- 测风误差：风速计故障或安装位置偏差导致功率曲线偏移。".to_string());
    possible_scada_causes.push("- 限功率运行：调度指令或策略限制实际出力。".to_string());
    possible_scada_causes.push("- 偏航误差：风向测量偏差或偏航系统响应延迟。".to_string());
    possible_scada_causes.push("- 设备输出受限：变桨、变流器或电网侧约束。".to_string());

    let mut supplementary_data = Vec::new();
    supplementary_data.push("- 10 分钟 SCADA 数据（风速、功率、转速、桨距角）".to_string());
    supplementary_data.push("- 报警记录和限功率日志".to_string());
    supplementary_data.push("- 风速计校准记录".to_string());
    supplementary_data.push("- 偏航角度和偏航误差趋势".to_string());

    format!(
        "# SCADA 数据分析报告\n\n\
         ## 1. 异常概述\n\n\
         {problem_summary}\n\n\
         ## 2. 可能原因\n\n\
         {scada_causes}\n\n\
         ## 3. 图谱匹配\n\n\
         {graph_section}\n\n\
         ## 4. 检查项目\n\n\
         {inspection_items}\n\n\
         ## 5. 需要补充的数据\n\n\
         {supplementary_data}\n\n\
         ## 6. 风险评估\n\n\
         - 风险等级：**{risk_level}**\n\
         - 需要人工确认：{human_confirm}\n\n\
         ## 7. 安全注意事项\n\n\
         {safety_notes}\n\n\
         ## 8. 证据来源\n\n\
         {evidence_sources}\n\n\
         ## 置信度\n\n\
         {confidence:.2}\n",
        problem_summary = fault_analysis.problem_summary,
        scada_causes = possible_scada_causes.join("\n"),
        graph_section = md_list_graph(graph_suggestions),
        inspection_items = md_list(&fault_analysis.inspection_items),
        supplementary_data = supplementary_data.join("\n"),
        risk_level = fault_analysis.risk_level,
        human_confirm = yes_no(fault_analysis.human_confirmation_required),
        safety_notes = md_list(&fault_analysis.safety_notes),
        evidence_sources = md_list(&fault_analysis.evidence_summary.hit_documents),
        confidence = fault_analysis.confidence,
    )
}

/// Full report generation – comprehensive Markdown report following the
/// structure of `report::build_wind_report_markdown`.
fn format_report_generation(
    fault_analysis: &FaultAnalysisResult,
    _advice: &WindInspectionAdvice,
    risk: &WindRiskAssessment,
    _graph_suggestions: &[GraphSuggestion],
) -> String {
    let now = std::time::SystemTime::now();
    let generated_time = format_timestamp(now);

    format!(
        "# 风力发电运维报告\n\n\
         ## 1. 基本信息\n\n\
         - 生成时间：{generated_time}\n\
         - 报告类型：skill_dispatched_report\n\n\
         ## 2. 故障描述\n\n\
         - 故障摘要：{problem_summary}\n\n\
         ## 3. 故障原因分析\n\n\
         {possible_causes}\n\n\
         ## 4. 检测建议\n\n\
         ### 建议检查项目\n\n\
         {inspection_items}\n\n\
         ### 建议检测方式\n\n\
         {inspection_methods}\n\n\
         ## 5. 风险评估\n\n\
         - 风险等级：**{risk_level}**\n\
         - 是否需要停机评估：{shutdown_eval}\n\
         - 是否需要人工确认：{human_confirm}\n\
         - 升级要求：{escalation}\n\n\
         ## 6. 维护建议\n\n\
         {maintenance_actions}\n\n\
         ## 7. 安全注意事项\n\n\
         {safety_notes}\n\n\
         ## 8. 证据来源\n\n\
         ### 命中文档\n\n\
         {hit_documents}\n\n\
         ### 图谱节点\n\n\
         {graph_nodes}\n\n\
         ### 风险评估依据\n\n\
         - 风险等级：{evidence_risk_level}\n\n\
         ## 9. 缺失数据\n\n\
         {missing_data}\n\n\
         ## 10. 建议复查周期\n\n\
         {recommended_interval}\n\n\
         ## 11. 置信度\n\n\
         {confidence:.2}\n\n\
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
        generated_time = generated_time,
        problem_summary = fault_analysis.problem_summary,
        possible_causes = md_list(&fault_analysis.possible_causes),
        inspection_items = md_list(&fault_analysis.inspection_items),
        inspection_methods = md_list(&fault_analysis.inspection_methods),
        risk_level = fault_analysis.risk_level,
        shutdown_eval = yes_no(fault_analysis.shutdown_evaluation_required),
        human_confirm = yes_no(fault_analysis.human_confirmation_required),
        escalation = yes_no(risk.escalation_required),
        maintenance_actions = md_list(&fault_analysis.maintenance_actions),
        safety_notes = md_list(&fault_analysis.safety_notes),
        hit_documents = md_list(&fault_analysis.evidence_summary.hit_documents),
        graph_nodes = md_list(&fault_analysis.evidence_summary.graph_nodes),
        evidence_risk_level = fault_analysis.evidence_summary.risk_level,
        missing_data = md_list(&fault_analysis.missing_data),
        recommended_interval = fault_analysis.recommended_interval,
        confidence = fault_analysis.confidence,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn md_list(items: &[String]) -> String {
    if items.is_empty() {
        return "- 不确定，需要补充数据。".to_string();
    }
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn md_list_or_fallback(items: &[String], fallback: &str) -> String {
    if items.is_empty() {
        format!("- {fallback}")
    } else {
        md_list(items)
    }
}

fn md_list_graph(suggestions: &[GraphSuggestion]) -> String {
    if suggestions.is_empty() {
        return "- 未命中故障图谱条目".to_string();
    }
    suggestions
        .iter()
        .map(|g| format!("- {}: {}（风险 {}）", g.component, g.symptom, g.risk_level))
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

/// Create a simple timestamp string (YYYYMMDD_HHmmss) from SystemTime.
fn format_timestamp(time: std::time::SystemTime) -> String {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs() as i64;

    let days = total_secs.div_euclid(86_400);
    let day_seconds = total_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

/// Convert days since Unix epoch to civil (year, month, day).
/// Uses the algorithm from http://howardhinnant.github.io/date_algorithms.html
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvidenceSummary;

    fn blade_graph() -> GraphSuggestion {
        GraphSuggestion {
            entry_id: "blade_crack".to_string(),
            component: "Blade".to_string(),
            symptom: "叶片裂纹".to_string(),
            risk_level: "Medium".to_string(),
            fault_mode: None,
            accompanying_symptoms: Vec::new(),
            escalates_to: None,
            mitigated_by: Vec::new(),
            inspection_items: vec!["裂纹长度".to_string(), "裂纹宽度".to_string()],
            inspection_methods: vec!["无人机复检".to_string(), "高清摄影".to_string()],
            recommended_interval: "30 天内复检".to_string(),
            maintenance_actions: vec!["建立跟踪工单".to_string()],
            shutdown_evaluation_conditions: vec!["裂纹进入主承载区域".to_string()],
            safety_notes: vec!["登塔需工作票".to_string()],
            evidence_sources: vec!["knowledge_base/fault_cases/blade_crack_case.md".to_string()],
        }
    }

    fn gearbox_graph() -> GraphSuggestion {
        GraphSuggestion {
            entry_id: "gearbox_temp".to_string(),
            component: "Gearbox".to_string(),
            symptom: "齿轮箱油温升高".to_string(),
            risk_level: "High".to_string(),
            fault_mode: None,
            accompanying_symptoms: Vec::new(),
            escalates_to: None,
            mitigated_by: Vec::new(),
            inspection_items: vec!["油温趋势".to_string(), "冷却系统状态".to_string()],
            inspection_methods: vec!["SCADA 趋势对比".to_string(), "振动频谱分析".to_string()],
            recommended_interval: "当天复核".to_string(),
            maintenance_actions: vec!["检查润滑和冷却系统".to_string()],
            shutdown_evaluation_conditions: vec!["油温持续上升且接近保护阈值".to_string()],
            safety_notes: vec!["不得绕过保护".to_string()],
            evidence_sources: vec!["knowledge_base/manuals/gearbox_temp_manual.md".to_string()],
        }
    }

    fn scada_graph() -> GraphSuggestion {
        GraphSuggestion {
            entry_id: "scada_power".to_string(),
            component: "SCADA".to_string(),
            symptom: "功率曲线异常".to_string(),
            risk_level: "Medium".to_string(),
            fault_mode: None,
            accompanying_symptoms: Vec::new(),
            escalates_to: None,
            mitigated_by: Vec::new(),
            inspection_items: vec!["风速计精度".to_string()],
            inspection_methods: vec!["功率曲线对比".to_string()],
            recommended_interval: "7 天".to_string(),
            maintenance_actions: vec!["校准风速计".to_string()],
            shutdown_evaluation_conditions: vec![],
            safety_notes: vec!["不要远程复位".to_string()],
            evidence_sources: vec!["knowledge_base/scada/power_curve.csv".to_string()],
        }
    }

    fn fault_result(risk_level: &str, component: &str) -> FaultAnalysisResult {
        FaultAnalysisResult {
            problem_summary: format!("{component}故障分析摘要"),
            possible_causes: vec!["可能原因1".to_string()],
            inspection_items: vec!["检查项1".to_string()],
            inspection_methods: vec!["检测方式1".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["维护动作1".to_string()],
            additional_context: Vec::new(),
            history_summary: None,
            risk_level: risk_level.to_string(),
            shutdown_evaluation_required: risk_level == "High",
            human_confirmation_required: true,
            safety_notes: vec!["安全注意1".to_string()],
            evidence_summary: EvidenceSummary {
                hit_documents: vec!["doc1.md".to_string()],
                graph_nodes: vec![format!("{component}:symptom")],
                risk_level: risk_level.to_string(),
            },
            missing_data: vec!["缺失数据1".to_string()],
            confidence: 0.72,
        }
    }

    fn advice_for(component: &str) -> WindInspectionAdvice {
        WindInspectionAdvice {
            problem_summary: format!("{component}建议摘要"),
            should_inspect: true,
            risk_level: "Medium".to_string(),
            additional_context: Vec::new(),
            inspection_items: vec!["检查项A".to_string()],
            inspection_methods: vec!["无人机复检".to_string(), "高清摄影".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["维护动作A".to_string()],
            shutdown_evaluation_conditions: vec![],
            safety_notes: vec!["安全注意A".to_string()],
            evidence_sources: vec!["evidence.md".to_string()],
            missing_data: vec![],
            confidence: 0.7,
        }
    }

    fn risk_for() -> WindRiskAssessment {
        WindRiskAssessment {
            risk_level: "Medium".to_string(),
            risk_reasons: vec!["风险原因1".to_string()],
            escalation_required: false,
            human_confirmation_required: true,
            shutdown_evaluation_required: false,
            immediate_actions: vec!["记录当前判断".to_string()],
            forbidden_actions: vec!["不得未经授权远程停机".to_string()],
            missing_data_impact: "缺失数据较少".to_string(),
            confidence: 0.65,
        }
    }

    // --- dispatch_skill tests ---

    #[test]
    fn dispatch_blade_by_component() {
        assert_eq!(
            dispatch_skill("发现问题", Some("Blade")),
            SkillType::BladeInspection
        );
    }

    #[test]
    fn dispatch_blade_by_keyword() {
        assert_eq!(
            dispatch_skill("叶片裂纹需要检查", None),
            SkillType::BladeInspection
        );
    }

    #[test]
    fn dispatch_gearbox_by_component() {
        assert_eq!(
            dispatch_skill("异常", Some("Gearbox")),
            SkillType::GearboxDiagnosis
        );
    }

    #[test]
    fn dispatch_gearbox_by_keyword() {
        assert_eq!(
            dispatch_skill("齿轮箱油温升高", None),
            SkillType::GearboxDiagnosis
        );
    }

    #[test]
    fn dispatch_scada_by_component() {
        assert_eq!(
            dispatch_skill("数据异常", Some("SCADA")),
            SkillType::ScadaAnalysis
        );
    }

    #[test]
    fn dispatch_scada_by_keyword() {
        assert_eq!(
            dispatch_skill("功率曲线异常报警", None),
            SkillType::ScadaAnalysis
        );
    }

    #[test]
    fn dispatch_report_by_keyword() {
        assert_eq!(
            dispatch_skill("生成报告", None),
            SkillType::ReportGeneration
        );
    }

    #[test]
    fn dispatch_report_keyword_overrides_component() {
        // component is None, keyword "报告" triggers ReportGeneration
        assert_eq!(
            dispatch_skill("帮我生成报告", None),
            SkillType::ReportGeneration
        );
    }

    #[test]
    fn dispatch_default_fallback() {
        assert_eq!(
            dispatch_skill("未知问题", None),
            SkillType::WindFaultAnalysis
        );
    }

    #[test]
    fn dispatch_component_takes_priority_over_keywords() {
        // Even though "振动" is a Gearbox keyword, component=Blade wins
        assert_eq!(
            dispatch_skill("振动异常", Some("Blade")),
            SkillType::BladeInspection
        );
    }

    // --- format_skill_output tests ---

    #[test]
    fn blade_inspection_output_contains_uav_section() {
        let fault = fault_result("Medium", "Blade");
        let advice = advice_for("Blade");
        let risk = risk_for();
        let graph = vec![blade_graph()];
        let output =
            format_skill_output(&SkillType::BladeInspection, &fault, &advice, &risk, &graph);
        assert!(output.contains("# 叶片巡检报告"));
        assert!(output.contains("无人机"));
        assert!(output.contains("复检周期"));
    }

    #[test]
    fn gearbox_diagnosis_output_contains_troubleshooting_steps() {
        let fault = fault_result("High", "Gearbox");
        let advice = advice_for("Gearbox");
        let risk = risk_for();
        let graph = vec![gearbox_graph()];
        let output =
            format_skill_output(&SkillType::GearboxDiagnosis, &fault, &advice, &risk, &graph);
        assert!(output.contains("# 齿轮箱诊断报告"));
        assert!(output.contains("油温排查"));
        assert!(output.contains("油样检查"));
        assert!(output.contains("振动分析"));
        assert!(output.contains("润滑状态"));
    }

    #[test]
    fn scada_analysis_output_contains_power_curve_causes() {
        let fault = fault_result("Medium", "SCADA");
        let advice = advice_for("SCADA");
        let risk = risk_for();
        let graph = vec![scada_graph()];
        let output = format_skill_output(&SkillType::ScadaAnalysis, &fault, &advice, &risk, &graph);
        assert!(output.contains("# SCADA 数据分析报告"));
        assert!(output.contains("测风误差"));
        assert!(output.contains("限功率运行"));
        assert!(output.contains("偏航误差"));
        assert!(output.contains("需要补充的数据"));
    }

    #[test]
    fn report_generation_output_contains_disclaimer() {
        let fault = fault_result("Medium", "Blade");
        let advice = advice_for("Blade");
        let risk = risk_for();
        let graph = vec![blade_graph()];
        let output =
            format_skill_output(&SkillType::ReportGeneration, &fault, &advice, &risk, &graph);
        assert!(output.contains("# 风力发电运维报告"));
        assert!(output.contains("免责声明"));
    }

    #[test]
    fn wind_fault_analysis_output_format() {
        let fault = fault_result("Medium", "Blade");
        let advice = advice_for("Blade");
        let risk = risk_for();
        let graph = vec![blade_graph()];
        let output = format_skill_output(
            &SkillType::WindFaultAnalysis,
            &fault,
            &advice,
            &risk,
            &graph,
        );
        assert!(output.contains("# 风电故障分析"));
        assert!(output.contains("故障概述"));
        assert!(output.contains("可能原因"));
        assert!(output.contains("风险评估"));
        assert!(output.contains("证据来源"));
    }

    #[test]
    fn builtin_skills_has_five_entries() {
        assert_eq!(builtin_skills().len(), 5);
    }

    #[test]
    fn skill_type_display() {
        assert_eq!(
            SkillType::WindFaultAnalysis.to_string(),
            "wind_fault_analysis"
        );
        assert_eq!(SkillType::BladeInspection.to_string(), "blade_inspection");
        assert_eq!(SkillType::GearboxDiagnosis.to_string(), "gearbox_diagnosis");
        assert_eq!(SkillType::ScadaAnalysis.to_string(), "scada_analysis");
        assert_eq!(SkillType::ReportGeneration.to_string(), "report_generation");
    }
}
