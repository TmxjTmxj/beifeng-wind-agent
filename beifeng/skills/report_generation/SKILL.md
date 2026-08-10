# Report Generation

## 适用场景

适用于基于故障分析结果、巡检建议、风险评估或维护建议生成 Markdown 运维报告。

## 输入

```json
{
  "report_type": "fault_report",
  "fault_analysis_result": {}
}
```

## 工作流程

1. 接收 FaultAnalysisResult、WindInspectionAdvice 或 WindRiskAssessment。
2. 选择报告模板：巡检报告、故障报告、风险评估报告或维护建议报告。
3. 生成结构化 Markdown。
4. 标注证据来源、缺失数据和人工确认事项。

## 必须调用的工具

- 可选：`wind_knowledge_query`
- 可选：`wind_fault_analysis`

当输入尚未包含完整故障分析结果时，应先调用 `wind_fault_analysis`。

## 输出格式

```markdown
# 风电运维报告

## 基本信息
...

## 问题概述
...

## 分析结果
...

## 风险评估
...

## 建议措施
...

## 安全提示
...

## 证据来源
...

## 缺失数据
...
```

## 安全边界

- 报告不得替代现场工程师判断。
- 高风险项目必须保留人工确认提示。
- 不得编造标准编号、故障代码或设备参数。

## 示例

用户：请根据叶片裂纹分析结果生成一份 Markdown 故障报告。

