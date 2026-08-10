use runtime::ConfigLoader;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConfigWarningMode {
    EmitStderr,
    SuppressStderr,
}

pub(crate) fn load_config_with_warning_mode(
    loader: &ConfigLoader,
    mode: ConfigWarningMode,
) -> Result<runtime::RuntimeConfig, runtime::ConfigError> {
    match mode {
        ConfigWarningMode::EmitStderr => loader.load(),
        ConfigWarningMode::SuppressStderr => loader
            .load_collecting_warnings()
            .map(|(runtime_config, _warnings)| runtime_config),
    }
}
