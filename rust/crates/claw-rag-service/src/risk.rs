//! Rule-based wind O&M risk assessment.

use serde::{Deserialize, Serialize};

use crate::{wind_rules_config, RagHit, WindInspectionAdvice};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindRiskAssessment {
    pub risk_level: String,
    pub risk_reasons: Vec<String>,
    pub escalation_required: bool,
    pub human_confirmation_required: bool,
    pub shutdown_evaluation_required: bool,
    pub immediate_actions: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub missing_data_impact: String,
    pub confidence: f32,
}

pub fn generate_wind_risk_assessment(
    advice: &WindInspectionAdvice,
    hits: &[RagHit],
) -> WindRiskAssessment {
    let mut risk_level = normalize_risk_level(&advice.risk_level);
    let graph_critical = advice.risk_level == "Critical";
    let mut risk_reasons = Vec::new();

    if risk_level == "High" {
        risk_reasons.push("故障图谱或建议层风险等级为 High，风险评估至少保持 High。".to_string());
    } else if risk_level == "Unknown" {
        risk_reasons.push("未匹配到明确风险等级，风险等级保持 Unknown。".to_string());
    } else {
        risk_reasons.push(format!("建议层风险等级为 {risk_level}。"));
    }

    if advice.evidence_sources.is_empty() {
        risk_reasons.push("证据来源为空，存在证据不足。".to_string());
    }

    let shutdown_evaluation_required = !advice.shutdown_evaluation_conditions.is_empty();
    if shutdown_evaluation_required {
        risk_reasons.push("存在停机评估条件，需要进行停机评估而不是直接远程停机。".to_string());
        risk_level = max_risk(&risk_level, "Medium").to_string();
    }

    let safety_text = joined_safety_text(advice);
    let human_confirmation_required = contains_safety_keyword(&safety_text);
    if human_confirmation_required {
        risk_reasons.push("内容涉及高风险作业或控制动作关键词，需要人工确认。".to_string());
    }

    let trigger_text = joined_trigger_text(advice);
    if contains_critical_work_context(&trigger_text) {
        risk_reasons.push(
            "建议内容涉及高压、吊装、受限空间或绕过安全联锁等Critical级作业场景。".to_string(),
        );
        risk_level = max_risk(&risk_level, "Critical").to_string();
    } else if contains_grid_quality_context(&trigger_text) {
        risk_reasons.push("建议内容涉及并网电能质量确认，风险等级至少为High。".to_string());
        risk_level = max_risk(&risk_level, "High").to_string();
    }

    apply_threshold_risk_rules(&mut risk_level, &mut risk_reasons, &trigger_text);

    let evidence_score = average_hit_score(hits);
    if let Some(score) = evidence_score {
        risk_reasons.push(format!("RAG evidence 平均分约为 {:.2}。", score));
    }

    let missing_count = advice.missing_data.len();
    let missing_data_impact = if missing_count >= 3 {
        "缺失数据较多，风险等级可参考但置信度明显降低；应补充现场和趋势数据。".to_string()
    } else if missing_count > 0 {
        "存在少量缺失数据，建议补充后复核风险判断。".to_string()
    } else {
        "缺失数据较少，风险判断受缺失数据影响较低。".to_string()
    };
    if missing_count > 0 {
        risk_reasons.push(format!(
            "missing_data 条目数为 {missing_count}，会降低置信度。"
        ));
    }

    let escalation_required = matches!(risk_level.as_str(), "High" | "Critical")
        || shutdown_evaluation_required
        || human_confirmation_required;

    // 如果 graph entry 标记为 Critical，最终风险等级不低于 Critical
    if graph_critical {
        risk_level = max_risk(&risk_level, "Critical").to_string();
    }

    WindRiskAssessment {
        risk_level,
        risk_reasons,
        escalation_required,
        human_confirmation_required,
        shutdown_evaluation_required,
        immediate_actions: immediate_actions(advice, escalation_required),
        forbidden_actions: forbidden_actions(),
        missing_data_impact,
        confidence: confidence(advice, hits),
    }
}

fn normalize_risk_level(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "low" => "Low".to_string(),
        "medium" => "Medium".to_string(),
        "high" => "High".to_string(),
        "critical" => "Critical".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn max_risk(current: &str, minimum: &str) -> &'static str {
    if risk_rank(current) >= risk_rank(minimum) {
        canonical_risk(current)
    } else {
        canonical_risk(minimum)
    }
}

fn risk_rank(value: &str) -> u8 {
    match value {
        "Low" => 1,
        "Medium" => 2,
        "High" => 3,
        "Critical" => 4,
        _ => 0,
    }
}

fn canonical_risk(value: &str) -> &'static str {
    match value {
        "Low" => "Low",
        "Medium" => "Medium",
        "High" => "High",
        "Critical" => "Critical",
        _ => "Unknown",
    }
}

fn joined_safety_text(advice: &WindInspectionAdvice) -> String {
    advice
        .safety_notes
        .iter()
        .chain(advice.shutdown_evaluation_conditions.iter())
        .chain(advice.maintenance_actions.iter())
        .chain(advice.missing_data.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

fn joined_trigger_text(advice: &WindInspectionAdvice) -> String {
    advice.problem_summary.clone()
}

fn apply_threshold_risk_rules(
    risk_level: &mut String,
    risk_reasons: &mut Vec<String>,
    risk_text: &str,
) {
    if contains_any(risk_text, &["裂纹深度", "裂纹深度超过"])
        && (contains_any(risk_text, &["壁厚50", "壁厚 50", "壁厚的50", "壁厚超过50"])
            || (contains_any(risk_text, &["8mm", "8 mm"])
                && contains_any(risk_text, &["16mm", "16 mm"])))
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "Critical",
            "裂纹深度超过壁厚50%，按结构失效阈值升级为Critical。",
        );
    }

    if contains_any(risk_text, &["齿面点蚀", "点蚀面积", "点蚀"])
        && contains_any(risk_text, &["齿面10", "齿面 10", "超过齿面10", "面积超过"])
        && contains_any(risk_text, &["振动上升", "振动持续上升", "振动增大"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "Critical",
            "齿面点蚀超过10%且振动上升，按复合传动链风险升级为Critical。",
        );
    }

    if contains_any(risk_text, &["igbt", "IGBT", "变流器"])
        && contains_any(risk_text, &["过温", "最高允许结温", "结温", "烧味"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "Critical",
            "变流器IGBT过温并出现结温或异味风险，升级为Critical。",
        );
    }

    if contains_any(risk_text, &["液压", "液压站"])
        && contains_any(
            risk_text,
            &[
                "最低运行液位",
                "油位低",
                "压力不足",
                "压力持续下降",
                "液压油泄漏",
            ],
        )
        && contains_any(risk_text, &["制动响应慢", "制动响应迟缓", "制动响应"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "High",
            "液压油位或压力不足叠加制动响应变慢，风险至少为High。",
        );
    }

    if contains_any(risk_text, &["覆冰", "冰厚"])
        && contains_any(risk_text, &["50mm", "50 mm", "超过50"])
        && contains_any(risk_text, &["继续运行", "运行"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "High",
            "覆冰厚度超过50mm且存在继续运行场景，风险至少为High。",
        );
    }

    if contains_any(risk_text, &["绝缘电阻", "定子绝缘"])
        && contains_any(risk_text, &["最低安全值", "低于", "下降"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "High",
            "发电机定子绝缘电阻低于安全阈值，风险至少为High。",
        );
    }

    if contains_any(risk_text, &["热点", "热成像"])
        && contains_any(risk_text, &["材料允许值", "持续升高", "接近"])
    {
        raise_risk(
            risk_level,
            risk_reasons,
            "High",
            "热成像热点接近材料允许值或持续升高，风险至少为High。",
        );
    }
}

fn raise_risk(
    risk_level: &mut String,
    risk_reasons: &mut Vec<String>,
    minimum: &str,
    reason: &str,
) {
    let raised = max_risk(risk_level, minimum).to_string();
    if raised != *risk_level {
        *risk_level = raised;
        risk_reasons.push(reason.to_string());
    }
}

fn contains_safety_keyword(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    wind_rules_config()
        .safety_keywords
        .trigger_human_confirmation
        .iter()
        .any(|keyword| lower.contains(&keyword.to_ascii_lowercase()))
}

fn contains_critical_work_context(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "高压测试前",
        "高压电气测试",
        "高压电操作",
        "高压电",
        "电气测试前",
        "高压操作必须持证",
        "停电挂牌",
        "断电、验电、接地",
        "吊装作业需确认",
        "吊装作业",
        "吊装更换",
        "吊装人员持证",
        "受限空间",
        "进入机舱",
        "绕过安全联锁",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

fn contains_grid_quality_context(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["并网运行前", "电压偏差", "功率因数", "谐波"]
        .iter()
        .any(|keyword| lower.contains(keyword))
}

fn contains_any(value: &str, tokens: &[&str]) -> bool {
    let lower = value.to_ascii_lowercase();
    tokens
        .iter()
        .any(|token| lower.contains(&token.to_ascii_lowercase()))
}

fn average_hit_score(hits: &[RagHit]) -> Option<f32> {
    let scores = hits
        .iter()
        .filter_map(|hit| hit.score.or(Some(hit.score_breakdown.final_score)))
        .collect::<Vec<_>>();
    if scores.is_empty() {
        None
    } else {
        Some(scores.iter().sum::<f32>() / scores.len() as f32)
    }
}

fn immediate_actions(advice: &WindInspectionAdvice, escalation_required: bool) -> Vec<String> {
    let mut actions = Vec::new();
    if escalation_required {
        actions.push("升级给现场工程师或值班负责人确认。".to_string());
    }
    if !advice.shutdown_evaluation_conditions.is_empty() {
        actions.push("按停机评估条件进行人工复核，禁止直接远程停机替代评估。".to_string());
    }
    if advice.evidence_sources.is_empty() {
        actions.push("补充 SCADA 趋势、巡检记录、工单或现场测量数据。".to_string());
    }
    if actions.is_empty() {
        actions.push("记录当前判断，并按建议周期复核。".to_string());
    }
    actions
}

fn forbidden_actions() -> Vec<String> {
    let actions = &wind_rules_config().forbidden_actions.actions;
    if actions.is_empty() {
        WindRulesFallback::forbidden_actions()
    } else {
        actions.clone()
    }
}

struct WindRulesFallback;

impl WindRulesFallback {
    fn forbidden_actions() -> Vec<String> {
        vec![
            "不得未经授权远程停机".to_string(),
            "不得未经授权远程复位".to_string(),
            "不得绕过安全联锁".to_string(),
            "不得替代现场工程师判断".to_string(),
        ]
    }
}

fn confidence(advice: &WindInspectionAdvice, hits: &[RagHit]) -> f32 {
    let mut value = advice.confidence;
    if advice.evidence_sources.is_empty() {
        value -= 0.15;
    }
    value -= (advice.missing_data.len() as f32 * 0.06).min(0.24);
    if hits.is_empty() {
        value -= 0.05;
    } else if average_hit_score(hits).unwrap_or(0.0) >= 0.65 {
        value += 0.05;
    }
    value.clamp(0.05, 0.95)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoreBreakdown;

    fn advice(risk_level: &str) -> WindInspectionAdvice {
        WindInspectionAdvice {
            problem_summary: "test".to_string(),
            should_inspect: true,
            risk_level: risk_level.to_string(),
            additional_context: Vec::new(),
            inspection_items: vec!["裂纹长度".to_string()],
            inspection_methods: vec!["无人机复检".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["建立工单".to_string()],
            shutdown_evaluation_conditions: vec!["裂纹进入主承载区域".to_string()],
            safety_notes: vec!["登塔需工作票".to_string()],
            evidence_sources: vec!["knowledge_base/fault_cases/blade_crack_case.md".to_string()],
            missing_data: vec!["SCADA 趋势".to_string()],
            confidence: 0.7,
        }
    }

    fn hit(score: f32) -> RagHit {
        RagHit {
            path: "a.md:0".to_string(),
            snippet: "sample".to_string(),
            score: Some(score),
            chunk_text: "sample".to_string(),
            source_path: "a.md".to_string(),
            file_type: Some("md".to_string()),
            domain: None,
            equipment: None,
            source_type: None,
            parser_status: Some("parsed".to_string()),
            score_breakdown: ScoreBreakdown {
                vector_score: score,
                keyword_score: 0.0,
                metadata_score: 0.0,
                final_score: score,
            },
        }
    }

    #[test]
    fn blade_crack_requires_shutdown_evaluation() {
        let assessment = generate_wind_risk_assessment(&advice("Medium"), &[hit(0.8)]);
        assert!(assessment.shutdown_evaluation_required);
        assert!(assessment.escalation_required);
    }

    #[test]
    fn gearbox_high_stays_high_or_above() {
        let mut a = advice("High");
        a.shutdown_evaluation_conditions = vec!["油温持续上升且接近保护阈值".to_string()];
        let assessment = generate_wind_risk_assessment(&a, &[hit(0.7)]);
        assert!(matches!(
            assessment.risk_level.as_str(),
            "High" | "Critical"
        ));
    }

    #[test]
    fn unknown_query_returns_unknown_and_missing_data() {
        let mut a = advice("Unknown");
        a.shutdown_evaluation_conditions.clear();
        a.evidence_sources.clear();
        a.missing_data = vec![
            "设备部件".to_string(),
            "故障现象".to_string(),
            "趋势数据".to_string(),
        ];
        let assessment = generate_wind_risk_assessment(&a, &[]);
        assert_eq!(assessment.risk_level, "Unknown");
        assert!(assessment.missing_data_impact.contains("缺失数据较多"));
        assert!(assessment
            .risk_reasons
            .iter()
            .any(|reason| reason.contains("证据不足")));
    }

    #[test]
    fn high_voltage_lifting_grid_keywords_require_human_confirmation() {
        let mut a = advice("Low");
        a.shutdown_evaluation_conditions.clear();
        a.safety_notes = vec!["涉及高压、吊装、并网操作时必须人工确认。".to_string()];
        let assessment = generate_wind_risk_assessment(&a, &[hit(0.6)]);
        assert!(assessment.human_confirmation_required);
    }

    #[test]
    fn critical_graph_entry_stays_critical() {
        let mut a = advice("Critical");
        a.shutdown_evaluation_conditions.clear();
        let assessment = generate_wind_risk_assessment(&a, &[hit(0.8)]);
        assert_eq!(assessment.risk_level, "Critical");
    }

    #[test]
    fn critical_graph_entry_stays_critical_with_shutdown_conditions() {
        let a = advice("Critical");
        let assessment = generate_wind_risk_assessment(&a, &[hit(0.7)]);
        assert_eq!(assessment.risk_level, "Critical");
        assert!(assessment.escalation_required);
    }
}
