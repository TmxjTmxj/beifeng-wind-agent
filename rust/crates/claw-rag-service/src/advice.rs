//! Rule-based wind inspection advice generation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{wind_rules_config, GraphSuggestion, RagHit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindInspectionAdvice {
    pub problem_summary: String,
    pub should_inspect: bool,
    pub risk_level: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_context: Vec<String>,
    pub inspection_items: Vec<String>,
    pub inspection_methods: Vec<String>,
    pub recommended_interval: String,
    pub maintenance_actions: Vec<String>,
    pub shutdown_evaluation_conditions: Vec<String>,
    pub safety_notes: Vec<String>,
    pub evidence_sources: Vec<String>,
    pub missing_data: Vec<String>,
    pub confidence: f32,
}

pub fn generate_wind_inspection_advice(
    query: &str,
    hits: &[RagHit],
    graph_suggestions: &[GraphSuggestion],
) -> WindInspectionAdvice {
    match (graph_suggestions.is_empty(), hits.is_empty()) {
        (false, false) => advice_from_graph_and_hits(query, graph_suggestions, hits),
        (false, true) => advice_from_graph_only(query, graph_suggestions),
        (true, false) => advice_from_hits_only(query, hits),
        (true, true) => advice_needs_more_data(query),
    }
}

fn advice_from_graph_and_hits(
    query: &str,
    graph_suggestions: &[GraphSuggestion],
    hits: &[RagHit],
) -> WindInspectionAdvice {
    let mut advice = advice_from_graph(query, graph_suggestions, 0.82);
    advice.evidence_sources = merge_sources(
        graph_suggestions
            .iter()
            .flat_map(|suggestion| suggestion.evidence_sources.iter().cloned())
            .chain(hits.iter().map(hit_source)),
    );
    advice.missing_data = default_missing_data();
    advice
}

fn advice_from_graph_only(
    query: &str,
    graph_suggestions: &[GraphSuggestion],
) -> WindInspectionAdvice {
    let mut advice = advice_from_graph(query, graph_suggestions, 0.62);
    advice.missing_data = vec![
        "缺少当前机组的现场检查记录或 SCADA/工单证据".to_string(),
        "需要补充缺陷位置、趋势变化和设备型号等数据".to_string(),
    ];
    advice
}

fn advice_from_graph(
    query: &str,
    graph_suggestions: &[GraphSuggestion],
    confidence: f32,
) -> WindInspectionAdvice {
    let primary = &graph_suggestions[0];
    let mut advice = WindInspectionAdvice {
        problem_summary: format!(
            "建议检测。查询“{}”匹配到 {} 的“{}”，需要通过检测确认缺陷范围、变化趋势和运行风险。",
            query, primary.component, primary.symptom
        ),
        should_inspect: true,
        risk_level: primary.risk_level.clone(),
        additional_context: Vec::new(),
        inspection_items: merge_list(graph_suggestions.iter().flat_map(|s| {
            s.inspection_items
                .iter()
                .cloned()
                .chain(domain_extra_items(s).into_iter())
                .chain(relation_inspection_items(s).into_iter())
        })),
        inspection_methods: merge_list(
            graph_suggestions
                .iter()
                .flat_map(|s| s.inspection_methods.iter().cloned()),
        ),
        recommended_interval: primary.recommended_interval.clone(),
        maintenance_actions: merge_list(graph_suggestions.iter().flat_map(|s| {
            s.maintenance_actions
                .iter()
                .cloned()
                .chain(s.mitigated_by.iter().cloned())
                .chain(domain_extra_maintenance(s, query).into_iter())
        })),
        shutdown_evaluation_conditions: merge_list(
            graph_suggestions
                .iter()
                .flat_map(|s| s.shutdown_evaluation_conditions.iter().cloned()),
        ),
        safety_notes: merge_list(
            graph_suggestions
                .iter()
                .flat_map(|s| s.safety_notes.iter().cloned()),
        ),
        evidence_sources: merge_sources(
            graph_suggestions
                .iter()
                .flat_map(|s| s.evidence_sources.iter().cloned()),
        ),
        missing_data: default_missing_data(),
        confidence,
    };
    let mut keyword_items = Vec::new();
    let mut keyword_methods = Vec::new();
    add_keyword_advice(query, "", &mut keyword_items, &mut keyword_methods);
    advice.inspection_items = merge_list(advice.inspection_items.into_iter().chain(keyword_items));
    advice.inspection_methods =
        merge_list(advice.inspection_methods.into_iter().chain(keyword_methods));
    advice.maintenance_actions = merge_list(
        advice
            .maintenance_actions
            .into_iter()
            .chain(keyword_maintenance_actions(query)),
    );
    advice.shutdown_evaluation_conditions = merge_list(
        advice
            .shutdown_evaluation_conditions
            .into_iter()
            .chain(keyword_shutdown_conditions(query)),
    );
    advice.safety_notes = merge_list(
        advice
            .safety_notes
            .into_iter()
            .chain(keyword_safety_notes(query)),
    );
    for suggestion in graph_suggestions {
        if let Some(relation) = &suggestion.escalates_to {
            advice.add_additional_context(format!(
                "故障升级风险：{} 如不及时处理，可能在“{}”条件下升级为 {}。",
                suggestion.symptom, relation.condition, relation.fault
            ));
        }
        if !suggestion.accompanying_symptoms.is_empty() {
            advice.add_additional_context(format!(
                "伴随症状提示：{}。",
                suggestion.accompanying_symptoms.join("、")
            ));
        }
    }
    advice
}

fn advice_from_hits_only(query: &str, hits: &[RagHit]) -> WindInspectionAdvice {
    let mut inspection_items = Vec::new();
    let mut inspection_methods = Vec::new();
    let text = hits
        .iter()
        .map(|hit| hit.chunk_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    add_keyword_advice(query, &text, &mut inspection_items, &mut inspection_methods);
    let maintenance_actions = keyword_maintenance_actions(query);
    let shutdown_evaluation_conditions = keyword_shutdown_conditions(query);
    let safety_notes = merge_list(
        [
            "涉及高压电、吊装、并网或旋转部件时必须人工确认。".to_string(),
            "不要直接建议远程停机、复位、变桨或并网切换等高风险控制动作。".to_string(),
        ]
        .into_iter()
        .chain(keyword_safety_notes(query).into_iter()),
    );

    WindInspectionAdvice {
        problem_summary: format!(
            "建议先做基础检测。查询“{}”未匹配到故障图谱条目，仅根据检索证据给出低置信度建议。",
            query
        ),
        should_inspect: true,
        risk_level: "Unknown".to_string(),
        additional_context: Vec::new(),
        inspection_items: non_empty_or(
            inspection_items,
            vec![
                "异常现象发生时间".to_string(),
                "相关报警和趋势数据".to_string(),
                "现场可见状态".to_string(),
            ],
        ),
        inspection_methods: non_empty_or(
            inspection_methods,
            vec!["资料复核".to_string(), "现场巡检".to_string()],
        ),
        recommended_interval: "不确定，需要结合现场数据确定复检周期。".to_string(),
        maintenance_actions: non_empty_or(
            maintenance_actions,
            vec!["先建立排查记录，确认故障对象和异常边界后再制定维修方案。".to_string()],
        ),
        shutdown_evaluation_conditions: non_empty_or(
            shutdown_evaluation_conditions,
            vec!["如异常伴随保护报警、快速恶化趋势或人身安全风险，应进行停机评估。".to_string()],
        ),
        safety_notes,
        evidence_sources: merge_sources(hits.iter().map(hit_source)),
        missing_data: vec![
            "缺少匹配的故障图谱条目".to_string(),
            "需要补充设备部件、故障现象、报警代码和趋势数据".to_string(),
        ],
        confidence: 0.36,
    }
}

fn advice_needs_more_data(query: &str) -> WindInspectionAdvice {
    WindInspectionAdvice {
        problem_summary: format!(
            "不确定，需要补充数据。查询“{}”没有匹配到知识库证据或故障图谱建议。",
            query
        ),
        should_inspect: false,
        risk_level: "Unknown".to_string(),
        additional_context: Vec::new(),
        inspection_items: Vec::new(),
        inspection_methods: Vec::new(),
        recommended_interval: "不确定，需要补充数据。".to_string(),
        maintenance_actions: Vec::new(),
        shutdown_evaluation_conditions: Vec::new(),
        safety_notes: vec![
            "缺少依据时不建议执行远程停机、复位、变桨或并网切换等高风险控制动作。".to_string(),
            "涉及高压电、吊装、并网或人身安全时必须人工确认。".to_string(),
        ],
        evidence_sources: Vec::new(),
        missing_data: vec![
            "设备部件或系统名称".to_string(),
            "具体症状、报警或故障代码".to_string(),
            "SCADA 趋势、巡检照片、工单或测量数据".to_string(),
        ],
        confidence: 0.1,
    }
}

impl WindInspectionAdvice {
    pub fn add_additional_context(&mut self, context: impl Into<String>) {
        let context = context.into();
        if !context.trim().is_empty()
            && !self.additional_context.iter().any(|item| item == &context)
        {
            self.additional_context.push(context);
        }
    }
}

fn add_keyword_advice(
    query: &str,
    text: &str,
    inspection_items: &mut Vec<String>,
    inspection_methods: &mut Vec<String>,
) {
    let combined = format!("{query}\n{text}");
    if contains_any(&combined, &["叶片", "裂纹", "blade", "crack"]) {
        inspection_items.extend([
            "裂纹长度".to_string(),
            "裂纹宽度".to_string(),
            "裂纹扩展方向".to_string(),
        ]);
        inspection_methods.extend(["无人机复检".to_string(), "高清摄影".to_string()]);
    }
    if contains_any(&combined, &["齿轮箱", "油温", "gearbox", "temperature"]) {
        inspection_items.extend([
            "油温趋势".to_string(),
            "油位和油品状态".to_string(),
            "冷却系统状态".to_string(),
            "振动趋势".to_string(),
        ]);
        inspection_methods.extend(["SCADA 趋势对比".to_string(), "油样检查".to_string()]);
    }
    if contains_any(&combined, &["振动", "轴承", "vibration", "bearing"]) {
        inspection_items.extend(["振动幅值趋势".to_string(), "频谱特征".to_string()]);
        inspection_methods.push("振动频谱分析".to_string());
    }
    if contains_any(&combined, &["振动频谱", "频谱分析"]) {
        inspection_items.extend([
            "不平衡特征频率".to_string(),
            "轴承故障频率".to_string(),
            "齿轮啮合频率".to_string(),
            "对中状态".to_string(),
        ]);
    }
    if contains_any(&combined, &["热成像", "接线盒", "热点"]) {
        inspection_items.extend(["接触电阻".to_string(), "端子紧固状态".to_string()]);
        inspection_methods.push("红外热成像复测".to_string());
    }
    if contains_any(&combined, &["无人机", "uav", "叶片巡检"]) {
        inspection_items.extend([
            "航线规划".to_string(),
            "空域确认".to_string(),
            "高清摄影".to_string(),
            "AI辅助缺陷识别".to_string(),
        ]);
        inspection_methods.push("无人机自主航线巡检".to_string());
    }
    if contains_any(&combined, &["高压", "电气测试"]) {
        inspection_items.extend([
            "持证作业资格".to_string(),
            "两人监护".to_string(),
            "停电挂牌".to_string(),
            "绝缘防护".to_string(),
            "断电验电接地".to_string(),
        ]);
    }
    if contains_any(&combined, &["吊装"]) {
        inspection_items.extend([
            "作业风速".to_string(),
            "吊装人员持证".to_string(),
            "警戒区设置".to_string(),
            "信号指挥".to_string(),
        ]);
    }
    if contains_any(&combined, &["国家标准", "遵守哪些", "标准"]) {
        inspection_items.extend([
            "GB/T 25383标准核对".to_string(),
            "DL/T 796检修规程核对".to_string(),
            "ISO 10816振动评价标准核对".to_string(),
        ]);
    }
    if contains_any(&combined, &["半年巡检", "巡检需要"]) {
        inspection_items.extend([
            "螺栓力矩复核".to_string(),
            "热成像检查".to_string(),
            "制动器检查".to_string(),
            "绝缘检查".to_string(),
        ]);
    }
    if contains_any(&combined, &["润滑油", "油检测", "油样"]) {
        inspection_items.extend([
            "油样光谱分析".to_string(),
            "润滑油粘度".to_string(),
            "酸值".to_string(),
            "水分".to_string(),
            "清洁度".to_string(),
        ]);
    }
    if contains_any(&combined, &["机舱作业", "受限空间", "进入机舱"]) {
        inspection_items.extend([
            "通风检测".to_string(),
            "气体检测".to_string(),
            "监护".to_string(),
        ]);
    }
    if contains_any(&combined, &["并网", "电能质量"]) {
        inspection_items.extend([
            "电压偏差".to_string(),
            "频率偏差".to_string(),
            "功率因数".to_string(),
            "谐波".to_string(),
        ]);
    }
    if contains_any(&combined, &["塔筒螺栓", "塔架螺栓", "螺栓复紧", "法兰间隙"]) {
        inspection_items.extend([
            "预紧力".to_string(),
            "法兰间隙".to_string(),
            "磁粉探伤".to_string(),
        ]);
        inspection_methods.extend(["力矩复核".to_string(), "磁粉探伤".to_string()]);
    }
    if contains_any(&combined, &["定子绝缘", "绝缘电阻", "绝缘老化", "局部放电"]) {
        inspection_items.extend([
            "绝缘电阻".to_string(),
            "极化指数".to_string(),
            "局部放电".to_string(),
        ]);
        inspection_methods.extend(["绝缘电阻测试".to_string(), "局部放电检测".to_string()]);
    }
    if contains_any(
        &combined,
        &["igbt过温", "IGBT过温", "变流器过温", "冷却系统堵塞"],
    ) {
        inspection_items.extend([
            "散热器".to_string(),
            "滤网".to_string(),
            "冷却风道".to_string(),
        ]);
        inspection_methods.extend(["红外热成像".to_string(), "冷却风量检查".to_string()]);
    }
    if contains_any(&combined, &["制动器磨损", "制动摩擦片", "摩擦片磨损"]) {
        inspection_items.extend(["制动力矩".to_string(), "摩擦片厚度".to_string()]);
        inspection_methods.push("制动试验".to_string());
    }
    if contains_any(&combined, &["齿面点蚀", "点蚀面积"]) {
        inspection_items.extend([
            "点蚀面积".to_string(),
            "振动频谱".to_string(),
            "油样铁谱".to_string(),
        ]);
        inspection_methods.extend(["内窥镜检查".to_string(), "油样铁谱分析".to_string()]);
    }
    if contains_any(
        &combined,
        &["液压", "液压油泄漏", "制动响应迟缓", "制动响应慢"],
    ) {
        inspection_items.extend([
            "液压压力".to_string(),
            "油位".to_string(),
            "泄漏点".to_string(),
            "密封件".to_string(),
            "制动响应".to_string(),
        ]);
    }
    if contains_any(&combined, &["覆冰", "冰厚"]) {
        inspection_items.extend([
            "覆冰厚度".to_string(),
            "振动监测".to_string(),
            "甩冰警戒".to_string(),
        ]);
        inspection_methods.push("停机评估".to_string());
    }
}

fn domain_extra_items(suggestion: &GraphSuggestion) -> Vec<String> {
    let config = wind_rules_config();

    // 基于配置的domain_extra_items匹配
    let key = match (suggestion.component.as_str(), suggestion.symptom.as_str()) {
        // Gearbox
        ("Gearbox", s) if s.contains("油温") => Some("Gearbox_oil_temp"),

        // Blade
        ("Blade", s) if s.contains("裂纹") => Some("Blade_crack"),

        // Generator
        ("Generator", s) if s.contains("振动") => Some("Generator_vibration"),
        ("Generator", s) if s.contains("绝缘") || s.contains("电阻") || s.contains("放电") => {
            Some("Generator_insulation")
        }

        // Tower
        ("Tower", s) if s.contains("螺栓") || s.contains("松动") || s.contains("预紧") => {
            Some("Tower_bolt_loosening")
        }

        // Converter
        ("Converter", s) if s.contains("散热") || s.contains("冷却") || s.contains("过温") => {
            Some("Converter_cooling")
        }

        // Brake/Yaw制动器
        ("Brake", _) => Some("Brake_wear"),
        ("Yaw", s) if s.contains("制动") || s.contains("刹车") => Some("Brake_wear"),

        _ => None,
    };

    if let Some(key) = key {
        if let Some(items) = config.domain_extra_items.get(key) {
            return items.clone();
        }
    }

    // 硬编码的默认项（保留向后兼容）
    match suggestion.component.as_str() {
        "Yaw" if suggestion.symptom.contains("偏航") => {
            vec!["风向标校验".to_string(), "偏航漂移趋势".to_string()]
        }
        "Pitch" if suggestion.symptom.contains("变桨") => {
            vec!["变桨驱动状态".to_string(), "后备电源状态".to_string()]
        }
        "Vibration" => vec![
            "不平衡特征".to_string(),
            "轴承频率".to_string(),
            "齿轮啮合频率".to_string(),
            "对中状态".to_string(),
        ],
        "Thermal" => vec!["接触电阻".to_string()],
        "UAV" => vec!["航线规划".to_string(), "空域确认".to_string()],
        "Cooling" => vec!["冷却液状态".to_string()],
        "Brake" => vec!["制动力矩".to_string()],
        _ => Vec::new(),
    }
}

fn relation_inspection_items(suggestion: &GraphSuggestion) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(relation) = &suggestion.escalates_to {
        items.push(format!("注意：如不及时处理，可能升级为 {}", relation.fault));
    }
    for symptom in &suggestion.accompanying_symptoms {
        items.push(format!("伴随症状复核：{symptom}"));
    }
    items
}

/// Domain-specific extra maintenance actions based on component, symptom, risk level and query.
fn domain_extra_maintenance(suggestion: &GraphSuggestion, query: &str) -> Vec<String> {
    let mut extras: Vec<String> = Vec::new();

    // "传动链专项检查": Gearbox + vibration in query or symptom
    if suggestion.component.eq_ignore_ascii_case("Gearbox")
        && (contains_any(query, &["振动"]) || contains_any(&suggestion.symptom, &["振动"]))
    {
        extras.push("传动链专项检查".to_string());
    }

    // "甩冰警戒": Blade + 覆冰 in symptom
    if suggestion.component.eq_ignore_ascii_case("Blade") && suggestion.symptom.contains("覆冰") {
        extras.push("甩冰警戒".to_string());
    }

    // "振动专项": Generator + 振动 in symptom
    if suggestion.component.eq_ignore_ascii_case("Generator") && suggestion.symptom.contains("振动")
    {
        extras.push("振动专项".to_string());
    }

    // "轴承更换": Generator + risk_level High
    if suggestion.component.eq_ignore_ascii_case("Generator")
        && suggestion.risk_level.eq_ignore_ascii_case("High")
    {
        extras.push("轴承更换".to_string());
    }

    if suggestion.component.eq_ignore_ascii_case("Blade") && suggestion.symptom.contains("雷击") {
        extras.extend([
            "接闪器检查".to_string(),
            "引下线导通性检测".to_string(),
            "接地电阻测试".to_string(),
        ]);
    }
    if suggestion.component.eq_ignore_ascii_case("Blade") && suggestion.symptom.contains("前缘侵蚀")
    {
        extras.extend(["前缘保护修补".to_string(), "修补后复测功率曲线".to_string()]);
    }
    if suggestion.component.eq_ignore_ascii_case("Gearbox")
        && suggestion.symptom.contains("油温")
        && contains_any(query, &["更严重", "导致", "升级"])
    {
        extras.extend(["故障扩展评估".to_string(), "72小时内加密监测".to_string()]);
    }
    if suggestion.component.eq_ignore_ascii_case("Gearbox")
        && suggestion.symptom.contains("轴承剥落")
    {
        extras.push("按ISO 10816限值进行停机评估".to_string());
    }
    if suggestion.component.eq_ignore_ascii_case("Gearbox") && suggestion.symptom.contains("行星架")
    {
        extras.extend([
            "立即组织内窥镜检查".to_string(),
            "灾难性失效风险评估".to_string(),
        ]);
    }
    if suggestion.component.eq_ignore_ascii_case("Pitch") && suggestion.symptom.contains("电池") {
        extras.extend([
            "后备电源功能验证".to_string(),
            "电池更换".to_string(),
            "禁止运行直至顺桨测试通过".to_string(),
        ]);
    }
    if suggestion.component.eq_ignore_ascii_case("Transformer") {
        extras.push("必要时更换绝缘油".to_string());
    }
    if suggestion.component.eq_ignore_ascii_case("Converter") && suggestion.symptom.contains("IGBT")
    {
        extras.extend(["更换IGBT模块".to_string(), "检查驱动板".to_string()]);
    }
    if suggestion.component.eq_ignore_ascii_case("Brake") {
        extras.extend(["更换磨损制动摩擦片".to_string(), "复测制动力矩".to_string()]);
    }

    // Tower螺栓松动专项维护
    if suggestion.component.eq_ignore_ascii_case("Tower")
        && (suggestion.symptom.contains("螺栓") || suggestion.symptom.contains("松动"))
    {
        extras.extend([
            "螺栓复紧按扭矩标准执行".to_string(),
            "法兰间隙调整".to_string(),
            "防腐处理".to_string(),
            "72小时后复查扭矩衰减".to_string(),
        ]);
    }

    // Generator绝缘老化专项维护
    if suggestion.component.eq_ignore_ascii_case("Generator")
        && (suggestion.symptom.contains("绝缘")
            || suggestion.symptom.contains("电阻")
            || suggestion.symptom.contains("放电"))
    {
        extras.extend([
            "绝缘油化验".to_string(),
            "局部放电在线监测".to_string(),
            "必要时绝缘清洗或更换".to_string(),
            "极化指数趋势跟踪".to_string(),
        ]);
    }

    // Converter散热系统专项维护
    if suggestion.component.eq_ignore_ascii_case("Converter")
        && (suggestion.symptom.contains("散热")
            || suggestion.symptom.contains("冷却")
            || suggestion.symptom.contains("过温"))
    {
        extras.extend([
            "散热器清洗".to_string(),
            "滤网更换".to_string(),
            "风扇功能测试".to_string(),
            "风道优化检查".to_string(),
        ]);
    }

    // 制动器磨损跟踪专项维护
    if suggestion.component.eq_ignore_ascii_case("Brake") && suggestion.symptom.contains("磨损") {
        extras.extend([
            "制动力矩标准化测试".to_string(),
            "摩擦片厚度周期性测量".to_string(),
            "更换阈值预警".to_string(),
            "复位功能验证".to_string(),
        ]);
    }

    extras
}

fn keyword_maintenance_actions(combined: &str) -> Vec<String> {
    let mut actions = Vec::new();
    if contains_any(combined, &["高压", "电气测试"]) {
        actions.extend([
            "高压操作必须持证，两人监护。".to_string(),
            "执行停电挂牌、验电、接地和绝缘防护。".to_string(),
        ]);
    }
    if contains_any(combined, &["吊装"]) {
        actions.extend([
            "确认作业风速满足吊装条件。".to_string(),
            "吊装人员持证，设置警戒区并统一信号指挥。".to_string(),
        ]);
    }
    if contains_any(combined, &["绕过安全联锁"]) {
        actions.push("禁止绕过安全联锁操作。".to_string());
    }
    if contains_any(combined, &["齿面点蚀", "点蚀面积"]) {
        actions.extend([
            "安排内窥镜检查确认齿面点蚀范围。".to_string(),
            "结合振动频谱和油样铁谱判断是否降载运行。".to_string(),
        ]);
    }
    if contains_any(
        combined,
        &["液压", "液压油泄漏", "制动响应迟缓", "制动响应慢"],
    ) {
        actions.extend([
            "复核液压压力、油位和制动响应时间。".to_string(),
            "定位泄漏点并检查密封件状态。".to_string(),
            "确认制动器响应恢复后再放行运行。".to_string(),
        ]);
    }
    if contains_any(combined, &["覆冰", "冰厚"]) {
        actions.extend([
            "执行停机评估并设置甩冰警戒。".to_string(),
            "完成除冰后进行振动监测复核。".to_string(),
        ]);
    }
    if contains_any(
        combined,
        &["igbt过温", "IGBT过温", "变流器过温", "冷却系统堵塞"],
    ) {
        actions.push("检查散热器、滤网和冷却风道，确认冷却系统恢复通畅。".to_string());
    }
    if contains_any(combined, &["塔筒螺栓", "塔架螺栓", "螺栓复紧", "法兰间隙"]) {
        actions.push("复核预紧力、法兰间隙并对可疑区域做磁粉探伤。".to_string());
    }
    if contains_any(combined, &["定子绝缘", "绝缘电阻", "绝缘老化"]) {
        actions.push("复测绝缘电阻、极化指数并安排局部放电检测。".to_string());
    }
    if contains_any(combined, &["制动器磨损", "制动摩擦片", "摩擦片磨损"]) {
        actions.push("复测制动力矩，达到磨损阈值时更换制动摩擦片。".to_string());
    }
    if contains_any(combined, &["并网", "电能质量"]) {
        actions.push("复核电压偏差、谐波、频率偏差和功率因数。".to_string());
    }
    actions
}

fn keyword_shutdown_conditions(query: &str) -> Vec<String> {
    let mut conditions = Vec::new();
    if contains_any(
        query,
        &["高压", "吊装", "受限空间", "机舱作业", "绕过安全联锁"],
    ) {
        conditions.push("涉及Critical级安全作业，必须人工审批和现场复核。".to_string());
    }
    if contains_any(query, &["并网"]) {
        conditions.push("并网前电压偏差、频率、功率因数或谐波不满足要求。".to_string());
    }
    conditions
}

fn keyword_safety_notes(query: &str) -> Vec<String> {
    let mut notes = Vec::new();
    if contains_any(query, &["高压", "电气测试"]) {
        notes.push("高压测试前必须断电、验电、接地，作业人员持证。".to_string());
    }
    if contains_any(query, &["吊装"]) {
        notes.push("吊装作业需确认风速、警戒区、持证人员和信号指挥。".to_string());
    }
    if contains_any(query, &["机舱作业", "受限空间", "进入机舱"]) {
        notes.push("进入机舱或受限空间需通风、气体检测和专人监护。".to_string());
    }
    if contains_any(query, &["绕过安全联锁"]) {
        notes.push("禁止绕过安全联锁，必须按保护逻辑和工作票执行。".to_string());
    }
    if contains_any(query, &["并网"]) {
        notes.push("并网运行前需确认电压偏差、频率、功率因数和谐波满足要求。".to_string());
    }
    notes
}

fn contains_any(value: &str, tokens: &[&str]) -> bool {
    let lower = value.to_ascii_lowercase();
    tokens.iter().any(|token| lower.contains(token))
}

fn hit_source(hit: &RagHit) -> String {
    if hit.source_path.is_empty() {
        hit.path.clone()
    } else {
        hit.source_path.clone()
    }
}

fn default_missing_data() -> Vec<String> {
    vec![
        "当前机组编号和设备型号".to_string(),
        "异常发生时间和持续时间".to_string(),
        "SCADA 趋势、报警记录、现场照片或测量值".to_string(),
    ]
}

fn merge_list(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn merge_sources(values: impl IntoIterator<Item = String>) -> Vec<String> {
    merge_list(values)
}

fn non_empty_or(values: Vec<String>, fallback: Vec<String>) -> Vec<String> {
    let values = merge_list(values);
    if values.is_empty() {
        fallback
    } else {
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoreBreakdown;

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

    fn hit(text: &str) -> RagHit {
        RagHit {
            path: "knowledge_base/manuals/sample.md:0".to_string(),
            snippet: text.to_string(),
            score: Some(0.8),
            chunk_text: text.to_string(),
            source_path: "knowledge_base/manuals/sample.md".to_string(),
            file_type: Some("md".to_string()),
            domain: None,
            equipment: None,
            source_type: None,
            parser_status: Some("parsed".to_string()),
            score_breakdown: ScoreBreakdown {
                vector_score: 0.7,
                keyword_score: 0.5,
                metadata_score: 0.0,
                final_score: 0.6,
            },
        }
    }

    #[test]
    fn blade_crack_advice_returns_interval_items_and_safety() {
        let advice = generate_wind_inspection_advice(
            "叶片裂纹应该多久复检",
            &[hit("blade crack inspection")],
            &[blade_graph()],
        );
        assert!(advice.should_inspect);
        assert!(advice.recommended_interval.contains("30"));
        assert!(advice
            .inspection_items
            .iter()
            .any(|item| item.contains("裂纹")));
        assert!(advice
            .safety_notes
            .iter()
            .any(|note| note.contains("工作票")));
    }

    #[test]
    fn gearbox_temperature_advice_returns_oil_vibration_lubrication_checks() {
        let advice = generate_wind_inspection_advice(
            "齿轮箱油温升高怎么排查",
            &[hit("gearbox oil temperature vibration lubrication")],
            &[gearbox_graph()],
        );
        assert!(advice
            .inspection_items
            .iter()
            .any(|item| item.contains("油温")));
        assert!(advice
            .inspection_items
            .iter()
            .any(|item| item.contains("油样")));
        assert!(advice
            .inspection_items
            .iter()
            .any(|item| item.contains("振动")));
        assert!(advice
            .inspection_items
            .iter()
            .any(|item| item.contains("润滑")));
    }

    #[test]
    fn unknown_query_returns_missing_data_advice() {
        let advice = generate_wind_inspection_advice("未知问题", &[], &[]);
        assert!(!advice.should_inspect);
        assert!(advice.problem_summary.contains("不确定"));
        assert!(
            advice.missing_data.iter().any(|item| item.contains("补充"))
                || !advice.missing_data.is_empty()
        );
        assert!(advice.confidence < 0.2);
    }
}
