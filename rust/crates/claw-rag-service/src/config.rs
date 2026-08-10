//! Runtime-loadable wind-domain rule configuration.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WindRulesConfig {
    #[serde(default)]
    pub possible_causes: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub forbidden_actions: ForbiddenActionsConfig,
    #[serde(default)]
    pub safety_keywords: SafetyKeywordsConfig,
    #[serde(default)]
    pub domain_extra_items: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ForbiddenActionsConfig {
    #[serde(default)]
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SafetyKeywordsConfig {
    #[serde(default)]
    pub trigger_human_confirmation: Vec<String>,
}

static WIND_RULES_CONFIG: OnceLock<Arc<WindRulesConfig>> = OnceLock::new();

impl WindRulesConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read wind rules config {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("parse wind rules config {}", path.display()))
    }

    #[must_use]
    pub fn default_embedded() -> Self {
        let mut possible_causes = HashMap::new();
        possible_causes.insert(
            "Blade".to_string(),
            strings([
                "雷击",
                "前缘侵蚀",
                "冰覆",
                "叶根疲劳",
                "制造缺陷",
                "裂纹扩展",
            ]),
        );
        possible_causes.insert(
            "Gearbox".to_string(),
            strings(["润滑不良", "齿面磨损", "轴承失效", "对中不良", "过载运行"]),
        );
        possible_causes.insert(
            "Generator".to_string(),
            strings(["绝缘老化", "转子不平衡", "轴承过热", "冷却失效"]),
        );
        possible_causes.insert(
            "Yaw".to_string(),
            strings([
                "偏航电机故障",
                "制动器磨损",
                "风向传感器失效",
                "偏航累积过大",
            ]),
        );
        possible_causes.insert(
            "Pitch".to_string(),
            strings(["变桨轴承润滑不良", "驱动系统故障", "备用电源失效"]),
        );
        possible_causes.insert(
            "Hydraulic".to_string(),
            strings(["密封件老化", "液压泵磨损", "管路腐蚀", "液压油污染"]),
        );
        possible_causes.insert(
            "Converter".to_string(),
            strings(["IGBT过温", "驱动电路异常", "散热系统故障"]),
        );
        possible_causes.insert(
            "Tower".to_string(),
            strings(["螺栓预紧力衰减", "焊缝缺陷", "腐蚀减薄", "基础沉降"]),
        );
        possible_causes.insert(
            "SCADA".to_string(),
            strings(["测量数据质量", "限功率策略", "偏航误差", "设备输出受限"]),
        );

        let mut domain_extra_items = HashMap::new();
        domain_extra_items.insert(
            "Gearbox_oil_temp".to_string(),
            strings([
                "油样状态",
                "润滑状态",
                "振动趋势",
                "72小时内复查油温扩展趋势",
            ]),
        );
        domain_extra_items.insert(
            "Blade_crack".to_string(),
            strings(["裂纹长度", "裂纹宽度", "裂纹扩展方向"]),
        );
        domain_extra_items.insert(
            "Generator_vibration".to_string(),
            strings(["动平衡校正", "轴承更换", "振动频谱分析"]),
        );

        Self {
            possible_causes,
            forbidden_actions: ForbiddenActionsConfig {
                actions: strings([
                    "不得未经授权远程停机",
                    "不得未经授权远程复位",
                    "不得绕过安全联锁",
                    "不得替代现场工程师判断",
                ]),
            },
            safety_keywords: SafetyKeywordsConfig {
                trigger_human_confirmation: strings([
                    "高压",
                    "吊装",
                    "并网",
                    "变桨",
                    "远程复位",
                    "复位",
                    "停机",
                    "绕过保护",
                    "修改温度阈值",
                    "安全联锁",
                    "高处作业",
                    "受限空间",
                    "大型机械",
                ]),
            },
            domain_extra_items,
        }
    }
}

pub fn set_global_wind_rules_config(config: WindRulesConfig) {
    let _ = WIND_RULES_CONFIG.set(Arc::new(config));
}

#[must_use]
pub fn wind_rules_config() -> Arc<WindRulesConfig> {
    WIND_RULES_CONFIG
        .get_or_init(|| Arc::new(WindRulesConfig::default_embedded()))
        .clone()
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(ToOwned::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_wind_rules_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rules.toml");
        std::fs::write(
            &path,
            r#"
[possible_causes]
Blade = ["雷击"]

[forbidden_actions]
actions = ["不得远程复位"]

[safety_keywords]
trigger_human_confirmation = ["复位"]

[domain_extra_items]
Blade_crack = ["裂纹长度"]
"#,
        )
        .unwrap();
        let cfg = WindRulesConfig::load(&path).unwrap();
        assert_eq!(cfg.possible_causes["Blade"], vec!["雷击"]);
        assert_eq!(cfg.forbidden_actions.actions, vec!["不得远程复位"]);
    }
}
