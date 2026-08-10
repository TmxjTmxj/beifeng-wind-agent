use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::knowledge::execute_wind_knowledge_query;
use super::{WindFaultAnalysisInput, WindKnowledgeQueryInput};

pub(super) fn execute_wind_fault_analysis(input: &WindFaultAnalysisInput) -> Result<Value, String> {
    let knowledge = execute_wind_knowledge_query(&WindKnowledgeQueryInput {
        query: input.problem.clone(),
        component: input.component.clone(),
        symptom: input.symptom.clone(),
        domain: input.component.clone(),
        equipment: None,
        top_k: Some(8),
        debug: None,
    })?;
    Ok(wind_fault_analysis_result(input, &knowledge))
}

fn wind_fault_analysis_result(input: &WindFaultAnalysisInput, knowledge: &Value) -> Value {
    let advice = &knowledge["advice"];
    let risk = &knowledge["risk_assessment"];
    let graph = knowledge["graph_suggestions"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let hits = knowledge["hits"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let risk_level = risk
        .get("risk_level")
        .and_then(Value::as_str)
        .or_else(|| advice.get("risk_level").and_then(Value::as_str))
        .unwrap_or("Unknown");

    json!({
        "problem_summary": fault_problem_summary(&input.problem, graph, hits),
        "possible_causes": fault_possible_causes(graph),
        "inspection_items": value_string_array(advice.get("inspection_items")),
        "inspection_methods": value_string_array(advice.get("inspection_methods")),
        "recommended_interval": advice.get("recommended_interval").and_then(Value::as_str).unwrap_or("不确定，需要补充数据。"),
        "maintenance_actions": value_string_array(advice.get("maintenance_actions")),
        "risk_level": risk_level,
        "shutdown_evaluation_required": risk.get("shutdown_evaluation_required").and_then(Value::as_bool).unwrap_or(false),
        "human_confirmation_required": risk.get("human_confirmation_required").and_then(Value::as_bool).unwrap_or(false),
        "safety_notes": value_string_array(advice.get("safety_notes")),
        "evidence_summary": {
            "hit_documents": hit_documents(hits),
            "graph_nodes": graph_nodes(graph),
            "risk_level": risk_level
        },
        "missing_data": value_string_array(advice.get("missing_data")),
        "confidence": average_confidence(
            advice.get("confidence").and_then(Value::as_f64),
            risk.get("confidence").and_then(Value::as_f64)
        )
    })
}

fn fault_problem_summary(problem: &str, graph: &[Value], hits: &[Value]) -> String {
    if let Some(primary) = graph.first() {
        let component = primary
            .get("component")
            .and_then(Value::as_str)
            .unwrap_or("Unknown");
        let symptom = primary
            .get("symptom")
            .and_then(Value::as_str)
            .unwrap_or(problem);
        return format!(
            "故障现象“{problem}”匹配到 {component} 的“{symptom}”，已按风电故障分析流程生成建议。"
        );
    }
    if !hits.is_empty() {
        return format!("故障现象“{problem}”未命中明确图谱节点，已基于检索证据生成低置信度分析。");
    }
    format!("故障现象“{problem}”缺少图谱和文档证据，需要补充数据后再诊断。")
}

fn fault_possible_causes(graph: &[Value]) -> Vec<String> {
    let mut causes = BTreeSet::new();
    for item in graph {
        match item
            .get("component")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "Blade" => {
                causes.insert("前缘冲蚀、雷击、外物冲击或表面缺陷。".to_string());
                causes.insert("裂纹位置和扩展趋势尚需现场复核。".to_string());
            }
            "Gearbox" => {
                causes.insert("润滑油状态、油位、冷却效率或轴承/齿轮啮合异常。".to_string());
            }
            "Generator" => {
                causes.insert("轴承润滑、对中、转子不平衡或冷却异常。".to_string());
            }
            "Yaw" => {
                causes.insert("偏航驱动、制动、风向测量或齿圈润滑异常。".to_string());
            }
            "SCADA" => {
                causes.insert("测量数据质量、限功率策略、偏航误差或设备输出受限。".to_string());
            }
            "Hydraulic" => {
                causes.insert("液压密封、管路连接或液压油品质异常。".to_string());
            }
            "Tower" => {
                causes.insert("塔筒连接螺栓预紧力衰减、焊缝缺陷或腐蚀减薄。".to_string());
            }
            "Cable" => {
                causes.insert("偏航累积导致电缆扭转超限或解缆功能失效。".to_string());
            }
            "Cooling" => {
                causes.insert("散热器堵塞、冷却液不足或冷却风扇故障。".to_string());
            }
            "Converter" => {
                causes.insert("IGBT过温/过流、散热系统故障或驱动电路异常。".to_string());
            }
            "Brake" => {
                causes.insert("制动衬片磨损超限、制动间隙过大或液压压力不足。".to_string());
            }
            "Transformer" => {
                causes.insert("负荷过大、冷却系统故障或绝缘油劣化。".to_string());
            }
            "Pitch" => {
                causes.insert("变桨轴承润滑不良、驱动系统故障或备用电源失效。".to_string());
            }
            _ => {}
        }
    }
    if causes.is_empty() {
        causes.insert("不确定，需要补充数据。".to_string());
    }
    causes.into_iter().collect()
}

pub(super) fn value_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn hit_documents(hits: &[Value]) -> Vec<String> {
    hits.iter()
        .filter_map(|hit| {
            hit.get("source_path")
                .or_else(|| hit.get("path"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn graph_nodes(graph: &[Value]) -> Vec<String> {
    graph
        .iter()
        .map(|item| {
            let component = item
                .get("component")
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            let symptom = item
                .get("symptom")
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            format!("{component}:{symptom}")
        })
        .collect()
}

fn average_confidence(advice: Option<f64>, risk: Option<f64>) -> f64 {
    match (advice, risk) {
        (Some(a), Some(r)) => ((a + r) / 2.0).clamp(0.0, 1.0),
        (Some(a), None) => a.clamp(0.0, 1.0),
        (None, Some(r)) => r.clamp(0.0, 1.0),
        (None, None) => 0.0,
    }
}
