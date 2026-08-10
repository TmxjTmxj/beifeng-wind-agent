mod cmms_connector;
mod connector_registry;
mod connector_result;
mod connector_trait;
mod scada_connector;
mod weather_connector;

pub use cmms_connector::{
    CmmsConnector, CmmsSource, MaintenanceRecord, SparePartHistory, WorkOrder,
};
pub use connector_registry::ConnectorRegistry;
pub use connector_result::{ConnectorHealth, ConnectorRecord, ConnectorResult, ConnectorStatus};
pub use connector_trait::{Connector, ConnectorRequest};
pub use scada_connector::{
    derive_scada_metrics, ScadaAlarm, ScadaConnector, ScadaDerivedMetrics, ScadaSource, ScadaTrend,
};
pub use weather_connector::{WeatherConnector, WeatherContext, WeatherEvent, WeatherSource};
