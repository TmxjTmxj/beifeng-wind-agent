use serde::{Deserialize, Serialize};

use crate::QueryResponse;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryContext {
    pub turbine_id: Option<String>,
    pub component: Option<String>,
    pub symptom: Option<String>,
    pub historical_fault_count: usize,
    pub last_fault_time: Option<String>,
    pub last_risk_level: Option<String>,
    pub maintenance_count: usize,
    pub previous_reports: Vec<String>,
    pub short_term_recurrence: bool,
}

impl MemoryContext {
    #[must_use]
    pub fn has_history(&self) -> bool {
        self.historical_fault_count > 0
            || self.maintenance_count > 0
            || !self.previous_reports.is_empty()
    }

    #[must_use]
    pub fn history_summary(&self) -> Option<String> {
        if !self.has_history() {
            return None;
        }
        let turbine = self.turbine_id.as_deref().unwrap_or("UNKNOWN");
        let component = self.component.as_deref().unwrap_or("Unknown");
        let symptom = self.symptom.as_deref().unwrap_or("未指定症状");
        let last_fault = self.last_fault_time.as_deref().unwrap_or("暂无记录");
        let last_risk = self.last_risk_level.as_deref().unwrap_or("Unknown");
        let current_ordinal = self.historical_fault_count + 1;
        Some(format!(
            "历史记忆：{turbine} {component}/{symptom} 历史同类故障 {faults} 次，最近一次 {last_fault}，上次风险 {last_risk}；历史维修 {maintenance} 次；当前为第 {current_ordinal} 次出现。",
            faults = self.historical_fault_count,
            maintenance = self.maintenance_count
        ))
    }

    #[must_use]
    pub fn risk_upgrade_reason(&self, current_risk: &str) -> Option<String> {
        if !self.has_concrete_turbine() {
            return None;
        }
        if self.historical_fault_count + 1 >= 3 && matches!(current_risk, "Medium" | "High") {
            return Some("同一风机同类故障累计达到3次，按历史复发规则升级风险。".to_string());
        }
        if self.short_term_recurrence && matches!(current_risk, "Medium" | "High") {
            return Some("维修后短期复发，按历史记忆规则升级风险。".to_string());
        }
        None
    }

    fn has_concrete_turbine(&self) -> bool {
        self.turbine_id
            .as_deref()
            .map(|value| {
                let value = value.trim();
                !value.is_empty() && !value.eq_ignore_ascii_case("UNKNOWN")
            })
            .unwrap_or(false)
    }
}

pub fn apply_memory_context(response: &mut QueryResponse, context: &MemoryContext) {
    if let Some(summary) = context.history_summary() {
        response.advice.add_additional_context(summary);
    }
    let current = response.risk_assessment.risk_level.clone();
    if let Some(reason) = context.risk_upgrade_reason(&current) {
        let upgraded = match current.as_str() {
            "Medium" => "High",
            "High" => "Critical",
            other => other,
        };
        if upgraded != current {
            response.risk_assessment.risk_level = upgraded.to_string();
            response.advice.risk_level = upgraded.to_string();
            response
                .risk_assessment
                .risk_reasons
                .push(format!("{reason} {current} → {upgraded}。"));
            response.risk_assessment.escalation_required = true;
            response
                .risk_assessment
                .immediate_actions
                .push("结合历史复发记录升级给现场负责人复核。".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        generate_wind_inspection_advice, generate_wind_risk_assessment, GraphSuggestion, SearchMode,
    };

    fn response(risk_level: &str) -> QueryResponse {
        let graph = vec![GraphSuggestion {
            entry_id: "blade_crack".to_string(),
            component: "Blade".to_string(),
            symptom: "叶片裂纹".to_string(),
            fault_mode: None,
            accompanying_symptoms: Vec::new(),
            escalates_to: None,
            mitigated_by: Vec::new(),
            inspection_items: vec!["裂纹长度".to_string()],
            inspection_methods: vec!["无人机复检".to_string()],
            recommended_interval: "30 天".to_string(),
            maintenance_actions: vec!["建立跟踪工单".to_string()],
            risk_level: risk_level.to_string(),
            shutdown_evaluation_conditions: Vec::new(),
            safety_notes: Vec::new(),
            evidence_sources: Vec::new(),
        }];
        let advice = generate_wind_inspection_advice("叶片裂纹", &[], &graph);
        let risk_assessment = generate_wind_risk_assessment(&advice, &[]);
        QueryResponse {
            hits: Vec::new(),
            graph_suggestions: graph,
            advice,
            risk_assessment,
            phase: "test".to_string(),
            search_mode: SearchMode::Hybrid,
            fts5_enabled: false,
        }
    }

    #[test]
    fn repeated_faults_upgrade_medium_to_high() {
        let mut response = response("Medium");
        let context = MemoryContext {
            turbine_id: Some("A001".to_string()),
            component: Some("Blade".to_string()),
            symptom: Some("叶片裂纹".to_string()),
            historical_fault_count: 2,
            ..MemoryContext::default()
        };
        apply_memory_context(&mut response, &context);
        assert_eq!(response.risk_assessment.risk_level, "High");
        assert!(response
            .advice
            .additional_context
            .iter()
            .any(|item| item.contains("历史记忆")));
    }
}
