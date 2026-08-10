# Wind Fault Analysis

## 适用场景

适用于风机故障诊断、巡检异常分析、检测建议生成、复检周期判断、停机评估和安全风险提示。

优先用于 Blade、Gearbox、Generator、Yaw、Pitch、SCADA、Safety 相关问题。

## 输入

```json
{
  "problem": "叶片裂纹",
  "component": "Blade",
  "symptom": "裂纹"
}
```

`component` 和 `symptom` 可选。仅提供 `problem` 时，应从问题文本中识别部件和故障现象。

## 工作流程

1. 解析用户故障描述，识别 component、symptom、domain、equipment。
2. 调用 `wind_knowledge_query` 获取 hits、graph_suggestions、advice、risk_assessment。
3. 汇总证据，形成 Evidence Summary。
4. 输出标准 FaultAnalysisResult。

## 必须调用的工具

- `wind_knowledge_query`

## 输出格式

```markdown
## 故障概述
...

## 可能原因
...

## 建议检查项目
...

## 建议检测方式
...

## 建议复检周期
...

## 维修建议
...

## 风险等级
...

## 是否需要停机评估
...

## 是否需要人工确认
...

## 安全提示
...

## 证据来源
...

## 缺失数据
...

## 置信度
...
```

## 安全边界

- 不直接建议远程停机、远程复位、变桨、并网切换等高风险控制动作。
- 涉及高压、吊装、并网、变桨、远程复位、停机、人身安全时，必须提示人工确认。
- 不得替代现场工程师判断。
- 缺少依据时明确说明“不确定，需要补充数据”。

## 示例

用户：叶片裂纹是否需要停机？多久复检？

期望：调用 `wind_knowledge_query`，输出故障概述、检测建议、复检周期、停机评估条件、安全提示和证据来源。

