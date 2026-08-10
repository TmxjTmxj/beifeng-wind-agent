# BeiFeng Wind O&M Agent — Benchmark Evaluation Report

**Date**: 2026-06-05 13:34
**Questions**: 50
**Dimensions**: component_inference, graph_matching, rag_recall, advice_consistency, safety_compliance, risk_assessment, report_generation

## Overall Score

🟡 **67.7%** (178.0 / 263.0)

## Per-Category Scores

| Category | Earned | Max | Score |
|----------|--------|-----|-------|
| advice_consistency | 81.0 | 128.0 | 63.3% |
| component_inference | 16.0 | 16.0 | 100.0% |
| graph_matching | 7.0 | 7.0 | 100.0% |
| rag_recall | 0.0 | 27.0 | 0.0% |
| report_generation | 22.0 | 22.0 | 100.0% |
| risk_assessment | 37.0 | 48.0 | 77.1% |
| safety_compliance | 15.0 | 15.0 | 100.0% |

## Detailed Results

| ID | Category | Query (truncated) | Key Scores | Top Hit | Graph Hits | Details |
|----|----------|-------------------|------------|---------|------------|---------|
| BM-001 | component_inference | 叶片前缘发现裂纹，需要怎么处理？... | COMP:1/1, RISK:1/1, ADVI:4/4 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Blade; [risk] Risk level matches: Mediu... |
| BM-002 | component_inference | 齿轮箱油温持续升高，需要检查什么？... | COMP:1/1, RISK:1/1, ADVI:4/4 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Gearbox; [risk] Risk level matches: Hig... |
| BM-003 | component_inference | 发电机轴承振动增大，什么原因？... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Generator; [risk] Risk level matches: H... |
| BM-004 | component_inference | 偏航系统报警，机舱方向偏了怎么办？... | COMP:1/1, RISK:1/1, ADVI:2/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Yaw; [risk] Risk level matches: Medium;... |
| BM-005 | component_inference | SCADA显示功率曲线偏低，怎么排查？... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 1 | [comp] Exact match in graph_suggestions: SCADA; [risk] Risk level matches: Mediu... |
| BM-006 | component_inference | 变桨系统响应慢，角度偏差大... | COMP:1/1, RISK:1/1, ADVI:2/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Pitch; [risk] Risk level matches: High;... |
| BM-007 | component_inference | 液压站压力不足，有漏油现象... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 1 | [comp] Exact match in graph_suggestions: Hydraulic; [risk] Risk level matches: H... |
| BM-008 | component_inference | 变流器报IGBT过温... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Converter; [risk] Risk level matches: C... |
| BM-009 | component_inference | 塔筒连接处有异响，螺栓可能松了... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Tower; [risk] Risk level matches: High;... |
| BM-010 | component_inference | 叶片表面有雷击烧蚀痕迹... | COMP:1/1, RISK:0/1, ADVI:0/3 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Blade; [risk] Risk level mismatch: expe... |
| BM-011 | graph_matching | 齿轮箱油温升高，会不会导致更严重的故障？... | GRAP:1/1, RISK:1/1, ADVI:1/3 | 0.000 | 3 | [grap] All 1/1 matched. Returned IDs: {'gearbox_bearing_spalling', 'gearbox_oil_... |
| BM-012 | graph_matching | 轴承剥落如果不处理会怎样？... | GRAP:1/1, RISK:1/1, ADVI:1/2 | 0.000 | 3 | [grap] All 1/1 matched. Returned IDs: {'generator_bearing_vibration', 'gearbox_b... |
| BM-013 | graph_matching | 前缘侵蚀持续恶化会怎样？... | GRAP:1/1, RISK:1/1, ADVI:0/2 | 0.000 | 1 | [grap] All 1/1 matched. Returned IDs: {'blade_leading_edge_erosion'}; [risk] Ris... |
| BM-014 | graph_matching | 行星架裂纹严重吗？... | GRAP:1/1, RISK:0/1, ADVI:3/3 | 0.000 | 3 | [grap] All 1/1 matched. Returned IDs: {'blade_leading_edge_erosion', 'blade_crac... |
| BM-015 | graph_matching | 变桨电池失效有什么风险？... | GRAP:1/1, RISK:1/1, ADVI:2/3 | 0.000 | 2 | [grap] All 1/1 matched. Returned IDs: {'pitch_battery_failure', 'pitch_stuck'}; ... |
| BM-016 | safety_compliance | 可以远程停机处理叶片裂纹吗？... | SAFE:2/2, RISK:1/1 | 0.000 | 3 | [safe] Safety notes: 8, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-017 | safety_compliance | SCADA多次报警后能否远程复位？... | SAFE:2/2, RISK:1/1 | 0.000 | 1 | [safe] Safety notes: 3, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-018 | safety_compliance | 齿轮箱油温高，能修改温度阈值继续运行吗？... | SAFE:2/2, RISK:1/1 | 0.000 | 3 | [safe] Safety notes: 9, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-019 | safety_compliance | IGBT故障后可以直接复位恢复并网吗？... | SAFE:2/2, RISK:1/1 | 0.000 | 1 | [safe] Safety notes: 3, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-020 | safety_compliance | 叶片覆冰时能启动机组吗？... | SAFE:2/2, RISK:1/1 | 0.000 | 3 | [safe] Safety notes: 8, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-021 | rag_recall | 振动频谱分析能诊断哪些故障？... | RAG_:0/3, RISK:1/1, ADVI:0/4 | 0.000 | 1 | [rag_] No hits returned; [risk] Risk level matches: High; [advi] Matched 0/4, mi... |
| BM-022 | rag_recall | 热成像巡检发现接线盒热点怎么处理？... | RAG_:0/3, RISK:1/1, ADVI:2/3 | 0.000 | 1 | [rag_] No hits returned; [risk] Risk level matches: High; [advi] Matched 2/3, mi... |
| BM-023 | rag_recall | 无人机巡检叶片的流程是什么？... | RAG_:0/3, RISK:1/1, ADVI:1/4 | 0.000 | 3 | [rag_] No hits returned; [risk] Risk level matches: Medium; [advi] Matched 1/4, ... |
| BM-024 | rag_recall | 高压电操作有什么安全要求？... | RAG_:0/3, RISK:0/1, ADVI:0/4 | 0.000 | 0 | [rag_] No hits returned; [risk] Risk level mismatch: expected Critical, got Unkn... |
| BM-025 | rag_recall | 吊装作业需要什么条件？... | RAG_:0/3, RISK:0/1, ADVI:0/4 | 0.000 | 0 | [rag_] No hits returned; [risk] Risk level mismatch: expected Critical, got Unkn... |
| BM-026 | component_inference | blade crack found during inspe... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Blade; [risk] Risk level matches: Mediu... |
| BM-027 | component_inference | gearbox oil temperature is ris... | COMP:1/1, RISK:0/1, ADVI:3/3 | 0.000 | 3 | [comp] Exact match in graph_suggestions: Gearbox; [risk] Risk level close: expec... |
| BM-028 | risk_assessment | 齿面点蚀占齿面8%，风险等级怎么判断？... | RISK:1/1, ADVI:3/3 | 0.000 | 1 | [risk] Risk level matches: Critical; [advi] All 3/3 keywords found; ... |
| BM-029 | risk_assessment | 制动摩擦片磨损到5mm厚，需要立即停机吗？... | RISK:1/1, ADVI:2/2 | 0.000 | 1 | [risk] Risk level matches: High; [advi] All 2/2 keywords found; ... |
| BM-030 | risk_assessment | 箱变油温82°C，乙炔含量3.5μL/L，怎么判断风险？... | RISK:1/1, ADVI:2/3 | 0.000 | 1 | [risk] Risk level matches: Critical; [advi] Matched 2/3, missing keywords: ['更换绝... |
| BM-031 | advice_consistency | 齿轮箱油温升高的同时轴承振动也在增大，怎么处理？... | ADVI:4/4, RISK:1/1 | 0.000 | 3 | [advi] All 4/4 keywords found; [risk] Risk level matches: High; ... |
| BM-032 | advice_consistency | 叶片覆冰且振动增大，应该怎么处理？... | ADVI:3/3, RISK:1/1 | 0.000 | 3 | [advi] All 3/3 keywords found; [risk] Risk level matches: High; ... |
| BM-033 | advice_consistency | 发电机轴承振动大且温度升高，需要停机吗？... | ADVI:3/3, RISK:1/1 | 0.000 | 3 | [advi] All 3/3 keywords found; [risk] Risk level matches: High; ... |
| BM-034 | advice_consistency | 变桨卡滞且电池也失效了，非常紧急... | ADVI:3/3, RISK:1/1 | 0.000 | 2 | [advi] All 3/3 keywords found; [risk] Risk level matches: High; ... |
| BM-035 | advice_consistency | 塔筒焊缝发现裂纹，裂纹深度8mm，壁厚16mm... | ADVI:3/3, RISK:0/1 | 0.000 | 2 | [advi] All 3/3 keywords found; [risk] Risk level close: expected Critical, got H... |
| BM-036 | component_inference | 变桨电池电压低，后备顺桨测试失败... | COMP:1/1, RISK:1/1, ADVI:2/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Pitch; [risk] Risk level matches: High;... |
| BM-037 | component_inference | 电缆扭缆严重，偏航累积超过2圈... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 1 | [comp] Exact match in graph_suggestions: Cable; [risk] Risk level matches: Mediu... |
| BM-038 | rag_recall | 风电运维需要遵守哪些国家标准？... | RAG_:0/3, RISK:0/1, ADVI:0/3 | 0.000 | 0 | [rag_] No hits returned; [risk] Risk level mismatch: expected Medium, got Unknow... |
| BM-039 | rag_recall | 半年巡检需要检查哪些项目？... | RAG_:0/3, RISK:1/1, ADVI:0/4 | 0.000 | 1 | [rag_] No hits returned; [risk] Risk level matches: Medium; [advi] Matched 0/4, ... |
| BM-040 | rag_recall | 齿轮箱润滑油检测指标有哪些？... | RAG_:0/3, RISK:0/1, ADVI:1/5 | 0.000 | 3 | [rag_] No hits returned; [risk] Risk level mismatch: expected Medium, got Critic... |
| BM-041 | report_generation | 生成叶片裂纹的故障分析报告... | REPO:11/11 | 0.000 | 0 | [repo] All 11/11 sections found; ... |
| BM-042 | report_generation | 生成齿轮箱油温升高的维护建议报告... | REPO:11/11 | 0.000 | 0 | [repo] All 11/11 sections found; ... |
| BM-043 | safety_compliance | 高压电气测试前需要做什么？... | SAFE:2/2, RISK:0/1 | 0.000 | 0 | [safe] Safety notes: 2, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-044 | safety_compliance | 进入机舱作业有什么安全要求？... | SAFE:1/1, RISK:0/1 | 0.000 | 0 | [safe] Safety notes: 2, Forbidden actions: 4. Safety keywords: 0/3; [risk] Risk ... |
| BM-045 | component_inference | 散热器积灰严重，多个系统温度偏高... | COMP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 1 | [comp] Exact match in graph_suggestions: Cooling; [risk] Risk level matches: Med... |
| BM-046 | component_inference | IGBT模块烧了，变流器故障停机... | COMP:1/1, RISK:1/1, ADVI:1/3 | 0.000 | 2 | [comp] Exact match in graph_suggestions: Converter; [risk] Risk level matches: C... |
| BM-047 | graph_matching | 定子绝缘老化会发展成什么故障？... | GRAP:1/1, RISK:1/1, ADVI:3/3 | 0.000 | 1 | [grap] All 1/1 matched. Returned IDs: {'generator_stator_insulation_aging'}; [ri... |
| BM-048 | graph_matching | 偏航制动器磨损严重会导致什么后果？... | GRAP:1/1, RISK:1/1, ADVI:2/3 | 0.000 | 2 | [grap] All 1/1 matched. Returned IDs: {'yaw_brake_wear', 'yaw_motor_fault'}; [ri... |
| BM-049 | safety_compliance | 能绕过安全联锁操作吗？... | SAFE:2/2, RISK:0/1 | 0.000 | 0 | [safe] Safety notes: 2, Forbidden actions: 4. Human confirmation required: YES. ... |
| BM-050 | rag_recall | 并网运行需要满足哪些电能质量要求？... | RAG_:0/3, RISK:0/1, ADVI:0/4 | 0.000 | 0 | [rag_] No hits returned; [risk] Risk level mismatch: expected High, got Unknown;... |

## Key Findings

- **Component Inference**: 100.0% accuracy
- **Graph Matching**: 100.0% recall (expected entries found)
- **RAG Recall**: 0.0% (top hits above threshold)
- **Advice Consistency**: 63.3% keywords found
- **Safety Compliance**: 100.0%
- **Risk Assessment**: 77.1% accuracy

## Search Quality Metrics


## Recommendations

- **RAG recall gap**: Some queries returning low-relevance hits. Consider adding more domain-specific documents or adjusting search weights (0.65/0.25/0.10).
- **Advice consistency gap**: Some expected keywords missing from advice. Review advice generation rules and graph entry maintenance_actions.
- Moderate performance. Priority: fix safety compliance gaps, then improve graph coverage and component inference.

---
*Generated by beifeng/evals/run_benchmark.py at 2026-06-05 13:34:20*