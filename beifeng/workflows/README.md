# Workflows

This directory documents the standard Wind O&M engineering workflows supported by the Agent.

Each workflow maps to the runtime tool chain (`wind_fault_analysis` → `wind_knowledge_query` →
advice / risk → `wind_report_generate`) and describes the operational flow used in the field.

| Workflow | File | Core chain |
| --- | --- | --- |
| Fault analysis | `fault_analysis_workflow.md` | 故障描述 → 组件推断 → 知识检索 → 图谱匹配 → 诊断建议 |
| Inspection | `inspection_workflow.md` | 巡检任务 → 数据采集（SCADA/热成像/无人机）→ 检索/匹配 → 风险分级 → 巡检报告 |
| Risk assessment | `risk_assessment_workflow.md` | 图谱命中 → 规则化风险等级 → 停机评估 / 人工确认判定 |
| Report generation | `report_generation_workflow.md` | 分析结果 → 13 段报告模板 → 证据来源 → 置信度 |

See also: [docs/engineering-guide.md](../../docs/engineering-guide.md) for the end-to-end
guide with a real diagnosis walkthrough, and [docs/sample-report-gearbox-overtemp.md](../../docs/sample-report-gearbox-overtemp.md)
for a real generated report.
