use config::builder::DefaultState;
use config::{ConfigBuilder, File};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    ConfigBuilder::<DefaultState>::default()
        .set_default("default", "1")?
        .add_source(File::with_name("configuration"))
        .set_override("override", "1")?
        .build()?
        .try_deserialize()
}
