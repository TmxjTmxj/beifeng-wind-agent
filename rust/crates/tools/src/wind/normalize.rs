use super::WindKnowledgeQueryInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NormalizedWindKnowledgeQuery {
    pub(super) query: String,
    pub(super) component: Option<String>,
    pub(super) domain: Option<String>,
    pub(super) equipment: Option<String>,
    pub(super) symptom: Option<String>,
}

#[derive(Debug, Clone)]
struct WindComponentMapping {
    domain: String,
    equipment: String,
}

#[derive(Debug, Clone, Copy)]
enum WindField {
    Domain,
    Equipment,
}

pub(super) fn normalize_wind_knowledge_query(
    input: &WindKnowledgeQueryInput,
) -> NormalizedWindKnowledgeQuery {
    let mut query = input.query.trim().to_string();
    let combined = [
        input.query.as_str(),
        input.component.as_deref().unwrap_or_default(),
        input.domain.as_deref().unwrap_or_default(),
        input.equipment.as_deref().unwrap_or_default(),
        input.symptom.as_deref().unwrap_or_default(),
    ]
    .join(" ");

    let inferred = infer_wind_component(&combined);
    let component = normalize_wind_field(input.component.as_deref(), WindField::Domain)
        .or_else(|| normalize_wind_field(input.domain.as_deref(), WindField::Domain))
        .or_else(|| normalize_wind_field(input.equipment.as_deref(), WindField::Domain))
        .or_else(|| inferred.clone().map(|component| component.domain));
    let domain = component
        .clone()
        .or_else(|| normalize_wind_field(input.domain.as_deref(), WindField::Domain));
    let equipment = normalize_wind_field(input.component.as_deref(), WindField::Equipment)
        .or_else(|| normalize_wind_field(input.domain.as_deref(), WindField::Equipment))
        .or_else(|| normalize_wind_field(input.equipment.as_deref(), WindField::Equipment))
        .or_else(|| inferred.map(|component| component.equipment));
    let symptom =
        normalize_wind_symptom(input.symptom.as_deref()).or_else(|| infer_wind_symptom(&combined));

    if let Some(symptom) = symptom.as_deref() {
        append_query_token(&mut query, symptom);
    }
    if let Some(domain) = domain.as_deref() {
        append_query_token(&mut query, domain);
    }

    NormalizedWindKnowledgeQuery {
        query,
        component,
        domain,
        equipment,
        symptom,
    }
}

fn normalize_wind_field(value: Option<&str>, field: WindField) -> Option<String> {
    let value = value?;
    if is_generic_wind_turbine(value) {
        return None;
    }
    infer_wind_component(value).map(|mapping| match field {
        WindField::Domain => mapping.domain,
        WindField::Equipment => mapping.equipment,
    })
}

fn infer_wind_component(value: &str) -> Option<WindComponentMapping> {
    if contains_any_text(value, &["叶片", "叶轮叶片", "桨叶", "blade"]) {
        return Some(wind_component("Blade", "blade"));
    }
    if contains_any_text(value, &["齿轮箱", "gearbox"]) {
        return Some(wind_component("Gearbox", "gearbox"));
    }
    if contains_any_text(value, &["发电机", "generator"]) {
        return Some(wind_component("Generator", "generator"));
    }
    if contains_any_text(value, &["偏航", "yaw"]) {
        return Some(wind_component("Yaw", "yaw_system"));
    }
    if contains_any_text(value, &["变桨", "pitch"]) {
        return Some(wind_component("Pitch", "pitch_system"));
    }
    if contains_any_text(value, &["scada", "功率曲线", "报警"]) {
        return Some(wind_component("SCADA", "scada"));
    }
    if contains_any_text(value, &["液压", "hydraulic"]) {
        return Some(wind_component("Hydraulic", "hydraulic_system"));
    }
    if contains_any_text(value, &["塔筒", "塔架", "tower"]) {
        return Some(wind_component("Tower", "tower"));
    }
    if contains_any_text(value, &["电缆", "扭缆", "cable"]) {
        return Some(wind_component("Cable", "cable"));
    }
    if contains_any_text(value, &["冷却", "散热", "cooling"]) {
        return Some(wind_component("Cooling", "cooling_system"));
    }
    if contains_any_text(value, &["变流器", "变频器", "converter"]) {
        return Some(wind_component("Converter", "converter"));
    }
    if contains_any_text(value, &["制动", "刹车", "brake"]) {
        return Some(wind_component("Brake", "brake_system"));
    }
    if contains_any_text(value, &["变压器", "箱变", "transformer"]) {
        return Some(wind_component("Transformer", "transformer"));
    }
    if contains_any_text(value, &["振动", "频谱", "vibration"]) {
        return Some(wind_component("Vibration", "vibration_monitoring"));
    }
    if contains_any_text(value, &["热成像", "红外", "thermal"]) {
        return Some(wind_component("Thermal", "thermal_imaging"));
    }
    if contains_any_text(value, &["无人机", "UAV", "巡检"]) {
        return Some(wind_component("UAV", "uav_inspection"));
    }
    None
}

fn wind_component(domain: &str, equipment: &str) -> WindComponentMapping {
    WindComponentMapping {
        domain: domain.to_string(),
        equipment: equipment.to_string(),
    }
}

fn normalize_wind_symptom(value: Option<&str>) -> Option<String> {
    infer_wind_symptom(value?)
}

fn infer_wind_symptom(value: &str) -> Option<String> {
    if contains_any_text(value, &["疑似裂纹", "表面裂纹", "裂纹", "裂痕", "开裂"]) {
        return Some("裂纹".to_string());
    }
    if contains_any_text(value, &["油温"]) {
        return Some("油温升高".to_string());
    }
    if contains_any_text(value, &["功率曲线"]) {
        return Some("功率曲线异常".to_string());
    }
    if contains_any_text(value, &["轴承振动"]) {
        return Some("轴承振动异常".to_string());
    }
    None
}

fn append_query_token(query: &mut String, token: &str) {
    if !query.contains(token) {
        query.push(' ');
        query.push_str(token);
    }
}

fn contains_any_text(value: &str, needles: &[&str]) -> bool {
    let lower = value.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| lower.contains(&needle.to_ascii_lowercase()))
}

fn is_generic_wind_turbine(value: &str) -> bool {
    let trimmed = value.trim();
    matches!(
        trimmed,
        "风力发电机组" | "风电机组" | "风机" | "机组" | "turbine" | "wind turbine"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query_input(query: &str) -> WindKnowledgeQueryInput {
        WindKnowledgeQueryInput {
            query: query.to_string(),
            component: None,
            symptom: None,
            domain: None,
            equipment: None,
            top_k: None,
            debug: None,
        }
    }

    fn query_input_with(
        query: &str,
        component: Option<&str>,
        symptom: Option<&str>,
    ) -> WindKnowledgeQueryInput {
        WindKnowledgeQueryInput {
            query: query.to_string(),
            component: component.map(String::from),
            symptom: symptom.map(String::from),
            domain: None,
            equipment: None,
            top_k: None,
            debug: None,
        }
    }

    // ── infer_wind_component tests ──────────────────────────────────

    #[test]
    fn infer_component_blade_chinese() {
        let mapping = infer_wind_component("叶片裂纹").unwrap();
        assert_eq!(mapping.domain, "Blade");
        assert_eq!(mapping.equipment, "blade");
    }

    #[test]
    fn infer_component_blade_english() {
        let mapping = infer_wind_component("blade crack").unwrap();
        assert_eq!(mapping.domain, "Blade");
    }

    #[test]
    fn infer_component_gearbox_chinese() {
        let mapping = infer_wind_component("齿轮箱油温升高").unwrap();
        assert_eq!(mapping.domain, "Gearbox");
        assert_eq!(mapping.equipment, "gearbox");
    }

    #[test]
    fn infer_component_gearbox_english() {
        let mapping = infer_wind_component("gearbox overheating").unwrap();
        assert_eq!(mapping.domain, "Gearbox");
    }

    #[test]
    fn infer_component_generator() {
        let mapping = infer_wind_component("发电机振动").unwrap();
        assert_eq!(mapping.domain, "Generator");
        assert_eq!(mapping.equipment, "generator");
    }

    #[test]
    fn infer_component_yaw() {
        let mapping = infer_wind_component("偏航系统故障").unwrap();
        assert_eq!(mapping.domain, "Yaw");
        assert_eq!(mapping.equipment, "yaw_system");
    }

    #[test]
    fn infer_component_pitch() {
        let mapping = infer_wind_component("变桨角度异常").unwrap();
        assert_eq!(mapping.domain, "Pitch");
        assert_eq!(mapping.equipment, "pitch_system");
    }

    #[test]
    fn infer_component_scada() {
        let mapping = infer_wind_component("SCADA报警").unwrap();
        assert_eq!(mapping.domain, "SCADA");
    }

    #[test]
    fn infer_component_unknown() {
        assert!(infer_wind_component("其他问题").is_none());
    }

    // ── new component inference tests ──────────────────────────────

    #[test]
    fn infer_component_hydraulic_chinese() {
        let mapping = infer_wind_component("液压油泄漏").unwrap();
        assert_eq!(mapping.domain, "Hydraulic");
        assert_eq!(mapping.equipment, "hydraulic_system");
    }

    #[test]
    fn infer_component_hydraulic_english() {
        let mapping = infer_wind_component("hydraulic pressure drop").unwrap();
        assert_eq!(mapping.domain, "Hydraulic");
    }

    #[test]
    fn infer_component_tower_chinese() {
        let mapping = infer_wind_component("塔筒裂纹").unwrap();
        assert_eq!(mapping.domain, "Tower");
        assert_eq!(mapping.equipment, "tower");
    }

    #[test]
    fn infer_component_tower_english() {
        let mapping = infer_wind_component("tower bolt loosening").unwrap();
        assert_eq!(mapping.domain, "Tower");
    }

    #[test]
    fn infer_component_cable_chinese() {
        let mapping = infer_wind_component("扭缆超限").unwrap();
        assert_eq!(mapping.domain, "Cable");
        assert_eq!(mapping.equipment, "cable");
    }

    #[test]
    fn infer_component_cable_english() {
        let mapping = infer_wind_component("cable twist").unwrap();
        assert_eq!(mapping.domain, "Cable");
    }

    #[test]
    fn infer_component_cooling_chinese() {
        let mapping = infer_wind_component("冷却系统堵塞").unwrap();
        assert_eq!(mapping.domain, "Cooling");
        assert_eq!(mapping.equipment, "cooling_system");
    }

    #[test]
    fn infer_component_cooling_english() {
        let mapping = infer_wind_component("cooling system blockage").unwrap();
        assert_eq!(mapping.domain, "Cooling");
    }

    #[test]
    fn infer_component_converter_chinese() {
        let mapping = infer_wind_component("变流器过温").unwrap();
        assert_eq!(mapping.domain, "Converter");
        assert_eq!(mapping.equipment, "converter");
    }

    #[test]
    fn infer_component_converter_english() {
        let mapping = infer_wind_component("converter IGBT fault").unwrap();
        assert_eq!(mapping.domain, "Converter");
    }

    #[test]
    fn infer_component_brake_chinese() {
        let mapping = infer_wind_component("制动衬片磨损").unwrap();
        assert_eq!(mapping.domain, "Brake");
        assert_eq!(mapping.equipment, "brake_system");
    }

    #[test]
    fn infer_component_brake_english() {
        let mapping = infer_wind_component("brake pad wear").unwrap();
        assert_eq!(mapping.domain, "Brake");
    }

    #[test]
    fn infer_component_transformer_chinese() {
        let mapping = infer_wind_component("变压器过温").unwrap();
        assert_eq!(mapping.domain, "Transformer");
        assert_eq!(mapping.equipment, "transformer");
    }

    #[test]
    fn infer_component_transformer_english() {
        let mapping = infer_wind_component("transformer overtemperature").unwrap();
        assert_eq!(mapping.domain, "Transformer");
    }

    #[test]
    fn infer_component_vibration_chinese() {
        let mapping = infer_wind_component("振动频谱异常").unwrap();
        assert_eq!(mapping.domain, "Vibration");
        assert_eq!(mapping.equipment, "vibration_monitoring");
    }

    #[test]
    fn infer_component_vibration_english() {
        let mapping = infer_wind_component("vibration spectrum abnormal").unwrap();
        assert_eq!(mapping.domain, "Vibration");
    }

    #[test]
    fn infer_component_thermal_chinese() {
        let mapping = infer_wind_component("热成像发现热点").unwrap();
        assert_eq!(mapping.domain, "Thermal");
        assert_eq!(mapping.equipment, "thermal_imaging");
    }

    #[test]
    fn infer_component_thermal_english() {
        let mapping = infer_wind_component("thermal imaging anomaly").unwrap();
        assert_eq!(mapping.domain, "Thermal");
    }

    #[test]
    fn infer_component_uav_chinese() {
        let mapping = infer_wind_component("无人机巡检异常").unwrap();
        assert_eq!(mapping.domain, "UAV");
        assert_eq!(mapping.equipment, "uav_inspection");
    }

    #[test]
    fn infer_component_uav_english() {
        let mapping = infer_wind_component("UAV inspection found crack").unwrap();
        assert_eq!(mapping.domain, "UAV");
    }

    // ── infer_wind_symptom tests ────────────────────────────────────

    #[test]
    fn infer_symptom_crack_chinese() {
        assert_eq!(infer_wind_symptom("疑似裂纹"), Some("裂纹".to_string()));
        assert_eq!(infer_wind_symptom("表面裂纹"), Some("裂纹".to_string()));
    }

    #[test]
    fn infer_symptom_oil_temp() {
        assert_eq!(infer_wind_symptom("油温过高"), Some("油温升高".to_string()));
    }

    #[test]
    fn infer_symptom_power_curve() {
        assert_eq!(
            infer_wind_symptom("功率曲线偏移"),
            Some("功率曲线异常".to_string())
        );
    }

    #[test]
    fn infer_symptom_bearing_vibration() {
        assert_eq!(
            infer_wind_symptom("轴承振动增大"),
            Some("轴承振动异常".to_string())
        );
    }

    #[test]
    fn infer_symptom_unknown() {
        assert!(infer_wind_symptom("其他症状").is_none());
    }

    // ── is_generic_wind_turbine tests ───────────────────────────────

    #[test]
    fn generic_turbine_filtered() {
        assert!(is_generic_wind_turbine("风机"));
        assert!(is_generic_wind_turbine("风力发电机组"));
        assert!(is_generic_wind_turbine("turbine"));
        assert!(is_generic_wind_turbine("wind turbine"));
    }

    #[test]
    fn specific_component_not_filtered() {
        assert!(!is_generic_wind_turbine("叶片"));
        assert!(!is_generic_wind_turbine("齿轮箱"));
        assert!(!is_generic_wind_turbine("generator"));
    }

    // ── normalize_wind_knowledge_query integration tests ────────────

    #[test]
    fn normalize_query_blade_crack() {
        let input = query_input_with("叶片裂纹检查", Some("叶片"), Some("裂纹"));
        let result = normalize_wind_knowledge_query(&input);
        assert_eq!(result.component.as_deref(), Some("Blade"));
        assert_eq!(result.symptom.as_deref(), Some("裂纹"));
    }

    #[test]
    fn normalize_query_gearbox_from_query_only() {
        let input = query_input("齿轮箱油温升高");
        let result = normalize_wind_knowledge_query(&input);
        assert_eq!(result.component.as_deref(), Some("Gearbox"));
        assert_eq!(result.symptom.as_deref(), Some("油温升高"));
    }

    #[test]
    fn normalize_query_generic_turbine_ignored() {
        let input = query_input_with("风机运行情况", Some("风机"), None);
        let result = normalize_wind_knowledge_query(&input);
        // "风机" is a generic term, should be filtered by is_generic_wind_turbine
        assert!(result.component.is_none() || result.component.as_deref() != Some("风机"));
    }

    #[test]
    fn normalize_query_english_component() {
        let input = query_input_with("blade inspection", Some("blade"), None);
        let result = normalize_wind_knowledge_query(&input);
        assert_eq!(result.component.as_deref(), Some("Blade"));
    }

    #[test]
    fn normalize_query_appends_symptom_to_query() {
        let input = query_input_with("检查问题", None, Some("裂纹"));
        let result = normalize_wind_knowledge_query(&input);
        assert!(result.query.contains("裂纹"));
    }

    // ── contains_any_text tests ─────────────────────────────────────

    #[test]
    fn contains_any_chinese_match() {
        assert!(contains_any_text("叶片裂纹检测", &["叶片", "blade"]));
    }

    #[test]
    fn contains_any_case_insensitive() {
        assert!(contains_any_text("Blade Inspection", &["blade"]));
    }
}
