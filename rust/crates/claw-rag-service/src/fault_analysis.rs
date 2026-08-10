//! Rule-based Wind Knowledge Hub fault analysis workflow.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{wind_rules_config, GraphSuggestion, QueryResponse, RagHit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultAnalysisInput {
    pub problem: String,
    pub component: Option<String>,
    pub symptom: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub hit_documents: Vec<String>,
    pub graph_nodes: Vec<String>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultAnalysisResult {
    pub problem_summary: String,
    pub possible_causes: Vec<String>,
    pub inspection_items: Vec<String>,
    pub inspection_methods: Vec<String>,
    pub recommended_interval: String,
    pub maintenance_actions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_context: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_summary: Option<String>,
    pub risk_level: String,
    pub shutdown_evaluation_required: bool,
    pub human_confirmation_required: bool,
    pub safety_notes: Vec<String>,
    pub evidence_summary: EvidenceSummary,
    pub missing_data: Vec<String>,
    pub confidence: f32,
}

#[must_use]
pub fn generate_fault_analysis_result(
    input: &FaultAnalysisInput,
    response: &QueryResponse,
) -> FaultAnalysisResult {
    let graph = &response.graph_suggestions;
    let advice = &response.advice;
    let risk = &response.risk_assessment;

    FaultAnalysisResult {
        problem_summary: problem_summary(input, response),
        possible_causes: possible_causes(graph),
        inspection_items: advice.inspection_items.clone(),
        inspection_methods: advice.inspection_methods.clone(),
        recommended_interval: advice.recommended_interval.clone(),
        maintenance_actions: advice.maintenance_actions.clone(),
        additional_context: advice.additional_context.clone(),
        history_summary: history_summary(&advice.additional_context),
        risk_level: risk.risk_level.clone(),
        shutdown_evaluation_required: risk.shutdown_evaluation_required,
        human_confirmation_required: risk.human_confirmation_required,
        safety_notes: advice.safety_notes.clone(),
        evidence_summary: evidence_summary(&response.hits, graph, &risk.risk_level),
        missing_data: advice.missing_data.clone(),
        confidence: ((advice.confidence + risk.confidence) / 2.0).clamp(0.0, 1.0),
    }
}

fn history_summary(items: &[String]) -> Option<String> {
    items.iter().find(|item| item.contains("历史记忆")).cloned()
}

fn problem_summary(input: &FaultAnalysisInput, response: &QueryResponse) -> String {
    if let Some(primary) = response.graph_suggestions.first() {
        return format!(
            "故障现象“{}”匹配到 {} 的“{}”，按风电故障分析流程生成检测、维修和风险评估建议。",
            input.problem, primary.component, primary.symptom
        );
    }
    if !response.hits.is_empty() {
        return format!(
            "故障现象“{}”未命中明确图谱节点，已基于检索证据生成低置信度分析。",
            input.problem
        );
    }
    format!(
        "故障现象“{}”缺少图谱和文档证据，需要补充数据后再诊断。",
        input.problem
    )
}

fn possible_causes(graph: &[GraphSuggestion]) -> Vec<String> {
    if graph.is_empty() {
        return vec!["不确定，需要补充数据。".to_string()];
    }
    graph
        .iter()
        .flat_map(|suggestion| {
            suggestion
                .evidence_sources
                .iter()
                .filter_map(|source| source_hint_as_cause(source))
                .chain(component_default_causes(suggestion).into_iter())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn source_hint_as_cause(source: &str) -> Option<String> {
    if source.contains("blade_crack") {
        Some("叶片表面或结构损伤扩展。".to_string())
    } else if source.contains("gearbox_temp") {
        Some("润滑、冷却或传动链状态异常。".to_string())
    } else if source.contains("power_curve") {
        Some("测风、偏航、限功率或输出链路异常。".to_string())
    } else {
        None
    }
}

fn component_default_causes(suggestion: &GraphSuggestion) -> Vec<String> {
    let config = wind_rules_config();
    if let Some(causes) = config.possible_causes.get(&suggestion.component) {
        return causes
            .iter()
            .map(|cause| format!("{cause}。"))
            .collect::<Vec<_>>();
    }
    match suggestion.component.as_str() {
        "Blade" => vec![
            "前缘冲蚀、雷击、外物冲击或表面缺陷。".to_string(),
            "裂纹位置和扩展趋势尚需现场复核。".to_string(),
        ],
        "Gearbox" => vec!["润滑油状态、油位、冷却效率或轴承/齿轮啮合异常。".to_string()],
        "Generator" => vec!["轴承润滑、对中、转子不平衡或冷却异常。".to_string()],
        "Yaw" => vec!["偏航驱动、制动、风向测量或齿圈润滑异常。".to_string()],
        "SCADA" => vec!["测量数据质量、限功率策略、偏航误差或设备输出受限。".to_string()],
        _ => Vec::new(),
    }
}

fn evidence_summary(
    hits: &[RagHit],
    graph: &[GraphSuggestion],
    risk_level: &str,
) -> EvidenceSummary {
    EvidenceSummary {
        hit_documents: hits
            .iter()
            .map(|hit| hit.source_path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        graph_nodes: graph
            .iter()
            .map(|suggestion| format!("{}:{}", suggestion.component, suggestion.symptom))
            .collect(),
        risk_level: risk_level.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        generate_wind_inspection_advice, generate_wind_risk_assessment, GraphSuggestion,
        QueryResponse, SearchMode, WindInspectionAdvice,
    };

    fn response(component: &str, symptom: &str, risk_level: &str) -> QueryResponse {
        let graph = vec![GraphSuggestion {
            entry_id: format!("{}_{}", component.to_ascii_lowercase(), symptom),
            component: component.to_string(),
            symptom: symptom.to_string(),
            risk_level: risk_level.to_string(),
            fault_mode: None,
            accompanying_symptoms: Vec::new(),
            escalates_to: None,
            mitigated_by: Vec::new(),
            inspection_items: vec!["检查项".to_string()],
            inspection_methods: vec!["检测方式".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["建立工单".to_string()],
            shutdown_evaluation_conditions: vec!["进入停机评估条件".to_string()],
            safety_notes: vec!["需要人工确认".to_string()],
            evidence_sources: vec!["knowledge_base/fault_cases/sample.md".to_string()],
        }];
        let advice: WindInspectionAdvice = generate_wind_inspection_advice(symptom, &[], &graph);
        let risk = generate_wind_risk_assessment(&advice, &[]);
        QueryResponse {
            hits: Vec::new(),
            graph_suggestions: graph,
            advice,
            risk_assessment: risk,
            phase: "test".to_string(),
            search_mode: SearchMode::Hybrid,
            fts5_enabled: false,
        }
    }

    #[test]
    fn blade_fault_analysis_uses_graph_advice_and_risk() {
        let result = generate_fault_analysis_result(
            &FaultAnalysisInput {
                problem: "叶片裂纹".to_string(),
                component: Some("Blade".to_string()),
                symptom: Some("裂纹".to_string()),
            },
            &response("Blade", "叶片裂纹", "Medium"),
        );
        assert_eq!(result.risk_level, "Medium");
        assert!(result.shutdown_evaluation_required);
        assert!(result
            .evidence_summary
            .graph_nodes
            .iter()
            .any(|node| node.contains("Blade")));
    }

    #[test]
    fn five_common_faults_generate_fault_analysis_results() {
        for (problem, component, symptom, risk) in [
            ("叶片裂纹", "Blade", "叶片裂纹", "Medium"),
            ("齿轮箱油温升高", "Gearbox", "齿轮箱油温升高", "High"),
            ("功率曲线异常", "SCADA", "功率曲线异常", "Medium"),
            (
                "发电机轴承振动异常",
                "Generator",
                "发电机轴承振动异常",
                "High",
            ),
            ("偏航异常", "Yaw", "偏航异常", "Medium"),
        ] {
            let result = generate_fault_analysis_result(
                &FaultAnalysisInput {
                    problem: problem.to_string(),
                    component: Some(component.to_string()),
                    symptom: Some(symptom.to_string()),
                },
                &response(component, symptom, risk),
            );
            assert_eq!(result.risk_level, risk);
            assert!(!result.problem_summary.is_empty());
            assert!(!result.inspection_items.is_empty());
            assert!(!result.evidence_summary.graph_nodes.is_empty());
        }
    }
}
