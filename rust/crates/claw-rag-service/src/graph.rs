//! Lightweight Wind Knowledge Hub fault graph.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultGraph {
    pub schema_version: String,
    pub entries: Vec<FaultGraphEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultGraphEntry {
    pub id: String,
    pub component: String,
    pub symptom: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_mode: Option<String>,
    pub possible_causes: Vec<String>,
    #[serde(default)]
    pub accompanying_symptoms: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalates_to: Option<FaultEscalationRelation>,
    #[serde(default)]
    pub mitigated_by: Vec<String>,
    pub inspection_items: Vec<String>,
    pub inspection_methods: Vec<String>,
    pub recommended_interval: String,
    pub maintenance_actions: Vec<String>,
    pub risk_level: String,
    pub shutdown_evaluation_conditions: Vec<String>,
    pub safety_notes: Vec<String>,
    pub evidence_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FaultEscalationRelation {
    pub fault: String,
    pub condition: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphQuery {
    pub component: Option<String>,
    pub symptom: Option<String>,
    pub risk_level: Option<String>,
    pub inspection_method: Option<String>,
    #[serde(default = "default_graph_limit")]
    pub limit: usize,
}

fn default_graph_limit() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSuggestion {
    pub entry_id: String,
    pub component: String,
    pub symptom: String,
    pub risk_level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_mode: Option<String>,
    #[serde(default)]
    pub accompanying_symptoms: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalates_to: Option<FaultEscalationRelation>,
    #[serde(default)]
    pub mitigated_by: Vec<String>,
    pub inspection_items: Vec<String>,
    pub inspection_methods: Vec<String>,
    pub recommended_interval: String,
    pub maintenance_actions: Vec<String>,
    pub shutdown_evaluation_conditions: Vec<String>,
    pub safety_notes: Vec<String>,
    pub evidence_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultEscalation {
    pub from_id: String,
    pub to_id: String,
    pub condition: String,
    pub to_entry: FaultGraphEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    pub graph_path: String,
    pub matches: Vec<GraphSuggestion>,
}

pub fn load_fault_graph(path: &Path) -> Result<FaultGraph, String> {
    let raw =
        std::fs::read_to_string(path).map_err(|e| format!("read graph {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse graph {}: {e}", path.display()))
}

pub fn query_fault_graph(graph: &FaultGraph, query: &GraphQuery) -> Vec<GraphSuggestion> {
    let mut scored = graph
        .entries
        .iter()
        .filter_map(|entry| {
            let score = graph_match_score(entry, query);
            (score > 0).then_some((score, entry))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    scored
        .into_iter()
        .take(query.limit.max(1))
        .map(|(_, entry)| suggestion_from_entry(entry))
        .collect()
}

pub fn query_fault_graph_file(
    graph_path: &Path,
    query: &GraphQuery,
) -> Result<GraphQueryResponse, String> {
    let graph = load_fault_graph(graph_path)?;
    let matches = query_fault_graph(&graph, query);
    Ok(GraphQueryResponse {
        graph_path: graph_path.to_string_lossy().replace('\\', "/"),
        matches,
    })
}

#[must_use]
pub fn default_graph_path() -> PathBuf {
    let relative = PathBuf::from("beifeng")
        .join("knowledge")
        .join("knowledge_graph")
        .join("wind_fault_graph.json");
    if relative.is_file() {
        return relative;
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors().take(4) {
            let candidate = ancestor.join(&relative);
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_root.ancestors().take(6) {
        let candidate = ancestor.join(&relative);
        if candidate.is_file() {
            return candidate;
        }
    }
    let legacy = PathBuf::from("knowledge_graph").join("wind_fault_graph.json");
    if legacy.is_file() {
        return legacy;
    }
    legacy
}

pub fn suggestions_for_query(
    graph_path: &Path,
    query_text: &str,
    domain: Option<&str>,
    equipment: Option<&str>,
) -> Result<Vec<GraphSuggestion>, String> {
    if !graph_path.is_file() {
        return Ok(Vec::new());
    }
    let normalized = normalize_graph_query(query_text, domain, equipment);
    let query = GraphQuery {
        component: normalized.component,
        symptom: normalized.symptom,
        risk_level: None,
        inspection_method: None,
        limit: 3,
    };
    Ok(query_fault_graph(&load_fault_graph(graph_path)?, &query))
}

/// 支持多组件查询的版本，从 query 文本中识别多个组件并分别查询
pub fn suggestions_for_multi_component_query(
    graph_path: &Path,
    query_text: &str,
) -> Result<Vec<GraphSuggestion>, String> {
    if !graph_path.is_file() {
        return Ok(Vec::new());
    }

    // 从 query 文本中提取所有可能的组件
    let mut queries = Vec::new();

    // 检查是否包含多个组件关键词
    // 使用 Vec 以便支持不同长度的关键词列表
    // 改进点：为 Generator、Converter、Yaw、Brake 添加更多上下文关键词
    // 优化逻辑：优先匹配更具体的关键词组合
    let components_to_check: Vec<(&str, Vec<&str>)> = vec![
        // Blade - 基础关键词 + 伴随症状
        (
            "Blade",
            vec![
                "叶片",
                "叶轮",
                "桨叶",
                "雷击",
                "前缘",
                "裂纹",
                "侵蚀",
                "覆冰",
                "叶片前缘",
                "叶片雷击",
                "叶片裂纹",
                "前缘侵蚀",
            ],
        ),
        // Gearbox - 基础关键词 + 关联故障
        (
            "Gearbox",
            vec![
                "齿轮箱",
                "行星架",
                "油温",
                "剥落",
                "齿轮箱油温",
                "齿轮箱油温升高",
                "齿轮箱轴承",
            ],
        ),
        // Generator - 增强关键词：增加"发电机"前缀和"定子/转子"关键词
        (
            "Generator",
            vec![
                "发电机",
                "定子",
                "转子",
                "绝缘",
                "轴承",
                "发电机轴承",
                "发电机定子",
                "发电机绝缘",
                "发电机温度",
                "定子绝缘",
                "定子绕组",
                "转子BALANCE",
            ],
        ),
        // Yaw - 增强关键词：增加"偏航制动"组合词
        (
            "Yaw",
            vec![
                "偏航",
                "偏航电机",
                "偏航制动",
                "偏航制动器",
                "偏航刹车",
                "偏航累积",
                "偏航计数器",
                "解缆",
            ],
        ),
        // Cable - 扭缆与偏航常伴随出现，但电缆本体需要独立返回
        (
            "Cable",
            vec!["电缆", "扭缆", "电缆扭缆", "扭缆超限", "电缆扭转"],
        ),
        // Pitch - 保持不变
        (
            "Pitch",
            vec![
                "变桨",
                "卡滞",
                "电池",
                "变桨电机",
                "变桨轴承",
                "变桨驱动",
                "后备顺桨",
                "变桨电池",
            ],
        ),
        // Converter - 增强关键词：增加"冷却/散热"关联
        (
            "Converter",
            vec![
                "变流器",
                "变频器",
                "IGBT",
                "过温",
                "变流器过温",
                "IGBT过温",
                "变流器冷却",
                "IGBT模块",
                "驱动板",
                "直流母线",
            ],
        ),
        // Hydraulic - 保持不变
        (
            "Hydraulic",
            vec![
                "液压",
                "压力",
                "泄漏",
                "液压站",
                "液压油",
                "液压泵",
                "液压压力",
                "液压泄漏",
            ],
        ),
        // Tower - 增强关键词：增加"螺栓松动"组合
        (
            "Tower",
            vec![
                "塔筒",
                "塔架",
                "塔筒螺栓",
                "塔筒连接处",
                "塔筒裂纹",
                "塔架裂纹",
                "焊缝裂纹",
                "法兰间隙",
                "塔筒垂直度",
                "基础沉降",
            ],
        ),
        // Cooling - 增强关键词：增加"散热器/滤网"具体部件
        (
            "Cooling",
            vec![
                "冷却",
                "散热",
                "滤网",
                "堵塞",
                "散热器",
                "冷却液",
                "散热器堵塞",
                "散热器清洁",
                "滤网更换",
                "冷却风扇",
                "进出风温差",
                "冷却系统堵塞",
            ],
        ),
        // Transformer - 增强关键词：增加"电能质量"关联
        (
            "Transformer",
            vec![
                "变压器",
                "箱变",
                "绝缘油",
                "绕组温度",
                "绝缘击穿",
                "负荷电流",
            ],
        ),
        // SCADA - 增强关键词：增加"功率曲线"和"报警"组合
        (
            "SCADA",
            vec![
                "scada",
                "功率曲线",
                "报警",
                "功率曲线异常",
                "功率偏低",
                "功率偏差",
                "风速仪",
                "偏航误差",
                "限功率",
                "并网",
                "电能质量",
                "谐波",
                "电压偏差",
            ],
        ),
        // Brake - 增强：独立制动器条目
        (
            "Brake",
            vec![
                "制动",
                "刹车",
                "磨损",
                "制动器",
                "制动摩擦片",
                "制动盘",
                "制动力矩",
                "制动衬片",
                "制动距离",
            ],
        ),
        // Vibration - 保持不变但调整关键词优先级
        (
            "Vibration",
            vec![
                "振动",
                "频谱",
                "bearing",
                "振动频谱",
                "振动趋势",
                "特征频率",
                "包络分析",
                "振动值",
            ],
        ),
        // Thermal - 增强关键词：增加"热点"相关
        (
            "Thermal",
            vec![
                "热成像",
                "红外",
                "热点",
                "红外热成像",
                "接线盒热点",
                "温度分布",
                "温差",
            ],
        ),
    ];

    for (component, keywords) in components_to_check {
        for keyword in keywords {
            if query_text.contains(keyword) {
                let query = GraphQuery {
                    component: Some(component.to_string()),
                    symptom: component_query_symptom(component, query_text),
                    risk_level: None,
                    inspection_method: None,
                    limit: 3,
                };
                queries.push(query);
                break; // 找到一个关键词就跳过该组件的其他关键词
            }
        }
    }

    // 如果没有识别到任何组件，使用默认查询
    if queries.is_empty() {
        let normalized = normalize_graph_query(query_text, None, None);
        let query = GraphQuery {
            component: normalized.component,
            symptom: normalized.symptom,
            risk_level: None,
            inspection_method: None,
            limit: 5,
        };
        return Ok(query_fault_graph(&load_fault_graph(graph_path)?, &query));
    }

    // 对每个组件查询并合并结果
    let graph = load_fault_graph(graph_path)?;
    let mut all_suggestions = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for query in queries {
        let suggestions = query_fault_graph(&graph, &query);
        for suggestion in suggestions {
            // 避免重复的条目
            if seen_ids.insert(suggestion.entry_id.clone()) {
                all_suggestions.push(suggestion);
            }
        }
    }

    // 对每个组件查询并合并结果（保留原有逻辑）
    // 按评分排序
    all_suggestions.sort_by(|a, b| {
        // 简单排序：先按 entry_id（保证稳定性），再按 component 重要性
        // 组件重要性排序（根据常见故障频率和复合故障风险）
        let component_importance = |c: &str| {
            match c {
                "Gearbox" | "Generator" | "Tower" | "Converter" => 0, // 最高优先级
                "Blade" | "Pitch" | "Hydraulic" | "Yaw" | "Cable" => 1, // 高优先级
                "SCADA" | "Transformer" | "Cooling" => 2,             // 中优先级
                "Brake" | "Vibration" | "Thermal" => 3,               // 低优先级
                _ => 4,
            }
        };
        component_importance(&a.component)
            .cmp(&component_importance(&b.component))
            .then_with(|| a.entry_id.cmp(&b.entry_id))
    });

    Ok(all_suggestions)
}

fn component_query_symptom(component: &str, query_text: &str) -> Option<String> {
    match component {
        "Blade" if contains_any_text(query_text, &["覆冰", "冰厚", "结冰"]) => {
            Some("叶片覆冰".to_string())
        }
        "Blade" if contains_any_text(query_text, &["前缘", "侵蚀"]) => {
            Some("前缘侵蚀".to_string())
        }
        "Blade" if contains_any_text(query_text, &["雷击", "接闪器", "引下线"]) => {
            Some("雷击损伤".to_string())
        }
        "Blade" if contains_any_text(query_text, &["裂纹", "裂痕", "开裂"]) => {
            Some("叶片裂纹".to_string())
        }
        "Gearbox" if contains_any_text(query_text, &["点蚀", "齿面", "啮合"]) => {
            Some("齿面点蚀".to_string())
        }
        "Gearbox" if contains_any_text(query_text, &["润滑油", "油检测", "油样"]) => {
            Some("油温升高".to_string())
        }
        "Gearbox" if contains_any_text(query_text, &["油温", "温度"]) => {
            Some("油温升高".to_string())
        }
        "Generator"
            if contains_any_text(
                query_text,
                &["定子绝缘", "绝缘电阻", "绝缘老化", "局部放电", "极化指数"],
            ) =>
        {
            Some("定子绝缘老化".to_string())
        }
        "Generator" if contains_any_text(query_text, &["轴承振动", "发电机振动"]) => {
            Some("轴承振动异常".to_string())
        }
        "Converter"
            if contains_any_text(
                query_text,
                &["变流器过温", "igbt过温", "IGBT过温", "结温", "烧味"],
            ) =>
        {
            Some("变流器过温".to_string())
        }
        "Converter" if contains_any_text(query_text, &["igbt", "IGBT"]) => {
            Some("IGBT故障".to_string())
        }
        "Hydraulic"
            if contains_any_text(
                query_text,
                &[
                    "液压油泄漏",
                    "液压泄漏",
                    "油位",
                    "压力不足",
                    "压力持续下降",
                    "制动响应",
                ],
            ) =>
        {
            Some("液压油泄漏".to_string())
        }
        "Tower" if contains_any_text(query_text, &["螺栓", "复紧", "预紧", "法兰间隙"]) => {
            Some("塔筒螺栓松动".to_string())
        }
        "Tower" if contains_any_text(query_text, &["塔筒裂纹", "塔架裂纹", "焊缝裂纹"]) => {
            Some("塔筒裂纹".to_string())
        }
        "Cooling" if contains_any_text(query_text, &["冷却", "散热", "滤网", "堵塞"]) => {
            Some("散热器堵塞".to_string())
        }
        "Transformer" if contains_any_text(query_text, &["箱变", "变压器", "绝缘油", "乙炔"]) => {
            Some("箱变过温".to_string())
        }
        "SCADA"
            if contains_any_text(
                query_text,
                &[
                    "功率曲线",
                    "功率低",
                    "功率偏低",
                    "风速正常",
                    "偏航误差",
                    "电能质量",
                    "并网",
                    "谐波",
                    "电压偏差",
                ],
            ) =>
        {
            Some("功率曲线异常".to_string())
        }
        "Brake"
            if contains_any_text(
                query_text,
                &["制动器磨损", "机械制动", "摩擦片", "制动力矩", "制动衬片"],
            ) =>
        {
            Some("机械制动器磨损".to_string())
        }
        "Yaw" if contains_any_text(query_text, &["偏航制动", "偏航刹车"]) => {
            Some("偏航制动器磨损".to_string())
        }
        "Yaw" if contains_any_text(query_text, &["偏航误差", "偏航异常"]) => {
            Some("偏航异常".to_string())
        }
        "Cable" if contains_any_text(query_text, &["电缆", "扭缆", "解缆"]) => {
            Some("扭缆".to_string())
        }
        "Thermal" if contains_any_text(query_text, &["热成像", "红外", "热点"]) => {
            Some("热成像异常热点".to_string())
        }
        "Vibration" if contains_any_text(query_text, &["振动", "频谱"]) => {
            Some("振动异常".to_string())
        }
        _ => None,
    }
}

#[must_use]
pub fn walk_escalation_path(
    entries: &[FaultGraphEntry],
    hit_ids: &[&str],
    max_depth: u8,
) -> Vec<FaultEscalation> {
    let by_id = entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut out = Vec::new();
    let mut frontier = hit_ids.to_vec();
    let mut seen = std::collections::BTreeSet::<String>::new();

    for _ in 0..max_depth {
        let mut next = Vec::new();
        for id in frontier.drain(..) {
            let Some(entry) = by_id.get(id).copied() else {
                continue;
            };
            let Some(relation) = &entry.escalates_to else {
                continue;
            };
            let Some(to_entry) = entries.iter().find(|candidate| {
                candidate.id == relation.fault || candidate.symptom == relation.fault
            }) else {
                continue;
            };
            let key = format!("{}->{}", entry.id, to_entry.id);
            if seen.insert(key) {
                out.push(FaultEscalation {
                    from_id: entry.id.clone(),
                    to_id: to_entry.id.clone(),
                    condition: relation.condition.clone(),
                    to_entry: to_entry.clone(),
                });
                next.push(to_entry.id.as_str());
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }
    out
}

#[must_use]
pub fn get_accompanying_symptoms(entries: &[FaultGraphEntry], hit_ids: &[&str]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| hit_ids.iter().any(|id| *id == entry.id))
        .flat_map(|entry| entry.accompanying_symptoms.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedGraphQuery {
    component: Option<String>,
    symptom: Option<String>,
}

fn normalize_graph_query(
    query_text: &str,
    domain: Option<&str>,
    equipment: Option<&str>,
) -> NormalizedGraphQuery {
    let combined = format!(
        "{} {} {}",
        query_text,
        domain.unwrap_or_default(),
        equipment.unwrap_or_default()
    );
    NormalizedGraphQuery {
        component: normalize_component(domain)
            .or_else(|| normalize_component(equipment))
            .or_else(|| normalize_component(Some(&combined))),
        symptom: normalize_symptom(Some(&combined)).or_else(|| Some(query_text.to_string())),
    }
}

fn normalize_component(value: Option<&str>) -> Option<String> {
    let value = value?;
    if contains_any_text(value, &["叶片", "叶轮叶片", "桨叶", "blade"]) {
        return Some("Blade".to_string());
    }
    if contains_any_text(value, &["齿轮箱", "行星架", "gearbox"]) {
        return Some("Gearbox".to_string());
    }
    if contains_any_text(value, &["发电机", "generator"]) {
        return Some("Generator".to_string());
    }
    if contains_any_text(value, &["电缆", "扭缆", "cable"]) {
        return Some("Cable".to_string());
    }
    if contains_any_text(value, &["偏航", "yaw"]) {
        return Some("Yaw".to_string());
    }
    if contains_any_text(value, &["变桨", "pitch"]) {
        return Some("Pitch".to_string());
    }
    if contains_any_text(value, &["scada", "功率曲线", "报警"]) {
        return Some("SCADA".to_string());
    }
    if contains_any_text(value, &["液压", "hydraulic"]) {
        return Some("Hydraulic".to_string());
    }
    if contains_any_text(value, &["塔筒", "塔架", "tower"]) {
        return Some("Tower".to_string());
    }
    if contains_any_text(value, &["冷却", "散热", "cooling"]) {
        return Some("Cooling".to_string());
    }
    if contains_any_text(value, &["变流器", "变频器", "converter"]) {
        return Some("Converter".to_string());
    }
    if contains_any_text(value, &["制动", "刹车", "brake"]) {
        return Some("Brake".to_string());
    }
    if contains_any_text(value, &["变压器", "箱变", "transformer"]) {
        return Some("Transformer".to_string());
    }
    if contains_any_text(value, &["振动", "频谱", "vibration"]) {
        return Some("Vibration".to_string());
    }
    if contains_any_text(value, &["热成像", "红外", "thermal"]) {
        return Some("Thermal".to_string());
    }
    if contains_any_text(value, &["无人机", "UAV", "巡检"]) {
        return Some("UAV".to_string());
    }
    None
}

fn normalize_symptom(value: Option<&str>) -> Option<String> {
    let value = value?;
    if contains_any_text(value, &["雷击", "烧蚀", "接闪器", "引下线"]) {
        return Some("雷击损伤".to_string());
    }
    if contains_any_text(value, &["焊缝裂纹", "塔筒裂纹"]) {
        return Some("塔筒裂纹".to_string());
    }
    if contains_any_text(value, &["行星架"]) {
        return Some("行星架裂纹".to_string());
    }
    if contains_any_text(value, &["轴承剥落", "剥落"]) {
        return Some("轴承剥落".to_string());
    }
    if contains_any_text(value, &["前缘侵蚀"]) {
        return Some("前缘侵蚀".to_string());
    }
    if contains_any_text(value, &["变桨电池", "后备顺桨", "电池电压"]) {
        return Some("变桨电池失效".to_string());
    }
    if contains_any_text(value, &["偏航制动器", "偏航制动", "偏航刹车"]) {
        return Some("偏航制动器磨损".to_string());
    }
    if contains_any_text(
        value,
        &["igbt过温", "变流器过温", "最高允许结温", "结温", "烧味"],
    ) {
        return Some("变流器过温".to_string());
    }
    if contains_any_text(value, &["igbt"]) {
        return Some("IGBT故障".to_string());
    }
    if contains_any_text(value, &["响应慢", "角度偏差", "变桨卡滞"]) {
        return Some("变桨卡滞".to_string());
    }
    if contains_any_text(value, &["液压站压力不足", "漏油", "液压油泄漏"]) {
        return Some("液压油泄漏".to_string());
    }
    if contains_any_text(value, &["液压压力", "压力不足", "油位低", "制动响应迟缓"])
    {
        return Some("液压油泄漏".to_string());
    }
    if contains_any_text(value, &["螺栓可能松", "螺栓松动", "螺栓复紧", "法兰间隙"])
    {
        return Some("塔筒螺栓松动".to_string());
    }
    if contains_any_text(
        value,
        &["散热器积灰", "散热器堵塞", "冷却系统堵塞", "滤网堵塞"],
    ) {
        return Some("散热器堵塞".to_string());
    }
    if contains_any_text(
        value,
        &["制动摩擦片", "摩擦片磨损", "机械制动器磨损", "制动器磨损"],
    ) {
        return Some("机械制动器磨损".to_string());
    }
    if contains_any_text(value, &["定子绝缘", "绝缘电阻", "极化指数", "局部放电"]) {
        return Some("定子绝缘老化".to_string());
    }
    if contains_any_text(value, &["齿面点蚀", "点蚀面积", "点蚀"]) {
        return Some("齿面点蚀".to_string());
    }
    if contains_any_text(value, &["覆冰", "冰厚"]) {
        return Some("叶片覆冰".to_string());
    }
    if contains_any_text(value, &["箱变过温", "变压器过温", "乙炔升高"]) {
        return Some("箱变过温".to_string());
    }
    if contains_any_text(value, &["电缆扭缆", "扭缆"]) {
        return Some("扭缆".to_string());
    }
    if contains_any_text(
        value,
        &["疑似裂纹", "表面裂纹", "裂纹", "裂痕", "开裂", "crack"],
    ) {
        return Some("裂纹".to_string());
    }
    if contains_any_text(value, &["热成像", "接线盒热点", "热点"]) {
        return Some("热成像异常热点".to_string());
    }
    if contains_any_text(value, &["无人机巡检", "叶片巡检"]) {
        return Some("无人机巡检发现叶片异常".to_string());
    }
    if contains_any_text(value, &["油温", "oil temperature", "temperature is rising"]) {
        return Some("油温升高".to_string());
    }
    if contains_any_text(value, &["功率曲线", "功率低", "功率偏低", "风速正常"]) {
        return Some("功率曲线异常".to_string());
    }
    if contains_any_text(value, &["轴承振动"]) {
        return Some("轴承振动异常".to_string());
    }
    if contains_any_text(value, &["偏航"]) {
        return Some("偏航异常".to_string());
    }
    None
}

fn contains_any_text(value: &str, needles: &[&str]) -> bool {
    let lower = value.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| lower.contains(&needle.to_ascii_lowercase()))
}

fn graph_match_score(entry: &FaultGraphEntry, query: &GraphQuery) -> u32 {
    let mut score = 0;
    let mut constrained = false;
    let mut component_only_score = 0;
    let mut symptom_checked = false;
    let mut symptom_matched = false;

    if let Some(component) = query
        .component
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        constrained = true;
        if text_matches(&entry.component, component) {
            component_only_score = 4;
        } else {
            return 0;
        }
    }
    if let Some(symptom) = query
        .symptom
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        constrained = true;
        symptom_checked = true;
        if text_matches(&entry.symptom, symptom)
            || entry
                .inspection_items
                .iter()
                .chain(entry.possible_causes.iter())
                .chain(entry.maintenance_actions.iter())
                .chain(entry.evidence_sources.iter())
                .any(|value| text_matches(value, symptom) || text_matches(symptom, value))
        {
            symptom_matched = true;
            score += 3;
        }
    }
    if let Some(risk) = query
        .risk_level
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        constrained = true;
        if entry.risk_level.eq_ignore_ascii_case(risk) {
            score += 4;
        } else {
            return 0;
        }
    }
    if let Some(method) = query
        .inspection_method
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        constrained = true;
        if entry
            .inspection_methods
            .iter()
            .any(|candidate| text_matches(candidate, method))
        {
            score += 3;
        }
    }

    if constrained && (!symptom_checked || symptom_matched) {
        score += component_only_score;
        score
    } else if constrained {
        0
    } else {
        1
    }
}

fn suggestion_from_entry(entry: &FaultGraphEntry) -> GraphSuggestion {
    GraphSuggestion {
        entry_id: entry.id.clone(),
        component: entry.component.clone(),
        symptom: entry.symptom.clone(),
        risk_level: entry.risk_level.clone(),
        fault_mode: entry.fault_mode.clone(),
        accompanying_symptoms: entry.accompanying_symptoms.clone(),
        escalates_to: entry.escalates_to.clone(),
        mitigated_by: entry.mitigated_by.clone(),
        inspection_items: entry.inspection_items.clone(),
        inspection_methods: entry.inspection_methods.clone(),
        recommended_interval: entry.recommended_interval.clone(),
        maintenance_actions: entry.maintenance_actions.clone(),
        shutdown_evaluation_conditions: entry.shutdown_evaluation_conditions.clone(),
        safety_notes: entry.safety_notes.clone(),
        evidence_sources: entry.evidence_sources.clone(),
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    let value_lower = value.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    value_lower.contains(&query_lower)
        || query_lower.contains(&value_lower)
        || cjk_token_overlap(value, query)
}

fn cjk_token_overlap(value: &str, query: &str) -> bool {
    cjk_tokens(query)
        .into_iter()
        .any(|token| token.chars().count() >= 2 && value.contains(&token))
}

fn cjk_tokens(value: &str) -> Vec<String> {
    let known = [
        "裂纹",
        "油温",
        "油温升高",
        "偏航",
        "功率曲线",
        "振动",
        "轴承",
        "点蚀",
        "齿面点蚀",
        "绝缘",
        "绝缘电阻",
        "定子绝缘",
        "局部放电",
        "覆冰",
        "变流器过温",
        "散热器",
        "滤网",
        "液压",
        "液压油泄漏",
        "制动响应",
        "扭缆",
        "箱变过温",
        "电能质量",
        "谐波",
        "电压偏差",
        "热点",
        "无人机",
        "超声",
        "红外",
        "高压",
    ];
    known
        .iter()
        .filter(|token| value.contains(*token))
        .map(|token| (*token).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_fixture() -> FaultGraph {
        FaultGraph {
            schema_version: "1.0".to_string(),
            entries: vec![
                FaultGraphEntry {
                    id: "blade_crack".to_string(),
                    component: "Blade".to_string(),
                    symptom: "叶片裂纹".to_string(),
                    fault_mode: None,
                    possible_causes: vec!["前缘冲蚀".to_string()],
                    accompanying_symptoms: Vec::new(),
                    escalates_to: None,
                    mitigated_by: Vec::new(),
                    inspection_items: vec!["裂纹长度".to_string()],
                    inspection_methods: vec!["无人机复检".to_string(), "超声检测".to_string()],
                    recommended_interval: "30 天".to_string(),
                    maintenance_actions: vec!["建立跟踪工单".to_string()],
                    risk_level: "Medium".to_string(),
                    shutdown_evaluation_conditions: vec!["进入主承载区域".to_string()],
                    safety_notes: vec!["登塔需工作票".to_string()],
                    evidence_sources: vec![
                        "knowledge_base/fault_cases/blade_crack_case.md".to_string()
                    ],
                },
                FaultGraphEntry {
                    id: "gearbox_temp".to_string(),
                    component: "Gearbox".to_string(),
                    symptom: "齿轮箱油温升高".to_string(),
                    fault_mode: None,
                    possible_causes: vec!["冷却系统效率下降".to_string()],
                    accompanying_symptoms: Vec::new(),
                    escalates_to: None,
                    mitigated_by: Vec::new(),
                    inspection_items: vec!["滤芯压差".to_string()],
                    inspection_methods: vec!["SCADA 趋势对比".to_string()],
                    recommended_interval: "当天复核".to_string(),
                    maintenance_actions: vec!["检查冷却系统".to_string()],
                    risk_level: "High".to_string(),
                    shutdown_evaluation_conditions: vec!["接近保护阈值".to_string()],
                    safety_notes: vec!["不得绕过保护".to_string()],
                    evidence_sources: vec![
                        "knowledge_base/manuals/gearbox_temp_manual.md".to_string()
                    ],
                },
            ],
        }
    }

    #[test]
    fn blade_crack_query_returns_inspection_advice() {
        let matches = query_fault_graph(
            &graph_fixture(),
            &GraphQuery {
                component: Some("Blade".to_string()),
                symptom: Some("裂纹".to_string()),
                ..GraphQuery::default()
            },
        );
        assert_eq!(matches[0].entry_id, "blade_crack");
        assert!(matches[0]
            .inspection_items
            .iter()
            .any(|item| item.contains("裂纹")));
    }

    #[test]
    fn gearbox_temperature_query_returns_check_advice() {
        let matches = query_fault_graph(
            &graph_fixture(),
            &GraphQuery {
                component: Some("Gearbox".to_string()),
                symptom: Some("油温升高".to_string()),
                ..GraphQuery::default()
            },
        );
        assert_eq!(matches[0].entry_id, "gearbox_temp");
        assert!(matches[0].maintenance_actions[0].contains("冷却"));
    }

    #[test]
    fn high_risk_query_returns_safety_notes() {
        let matches = query_fault_graph(
            &graph_fixture(),
            &GraphQuery {
                risk_level: Some("High".to_string()),
                ..GraphQuery::default()
            },
        );
        assert_eq!(matches[0].risk_level, "High");
        assert!(matches[0].safety_notes[0].contains("保护"));
    }

    #[test]
    fn chinese_blade_crack_query_normalizes_for_graph_lookup() {
        let normalized = normalize_graph_query(
            "无人机巡检发现叶片疑似裂纹，是否需要停机？多久复检？",
            None,
            None,
        );
        assert_eq!(normalized.component.as_deref(), Some("Blade"));
        assert_eq!(normalized.symptom.as_deref(), Some("裂纹"));

        let matches = query_fault_graph(
            &graph_fixture(),
            &GraphQuery {
                component: normalized.component,
                symptom: normalized.symptom,
                ..GraphQuery::default()
            },
        );
        assert_eq!(matches[0].entry_id, "blade_crack");
    }
}
