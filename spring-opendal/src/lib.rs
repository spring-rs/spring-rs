mod config;

use std::str::FromStr;
use opendal::Operator;
use spring_boot::config::Configurable;
use crate::config::OpenDALConfig;
use anyhow::Result;
use spring_boot::app::AppBuilder;
use spring_boot::async_trait;
use spring_boot::plugin::Plugin;

pub type Op = opendal::Operator;

#[derive(Configurable)]
#[config_prefix = "opendal"]
pub struct OpenDALPlugin;

#[async_trait]
impl Plugin for OpenDALPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenDALConfig>(self)
            .expect("OpenDAL plugin config load failed");

        let connect: Operator = Self::operator(config).expect("OpenDAL operator construct failed");
        app.add_component(connect);
    }
}

impl OpenDALPlugin {
    pub fn operator(config: OpenDALConfig) -> Result<Operator> {
        
        let scheme = opendal::Scheme::from_str(&config.scheme)
            .map_err(|err| {
                opendal::Error::new(opendal::ErrorKind::Unexpected, "not supported scheme")
                    .set_source(err)
            })?;
        
        let options = config.options.unwrap_or_default();

        Ok(Operator::via_iter(scheme, options)?)
    }
}