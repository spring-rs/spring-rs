pub use axum::extract::*;

use crate::AppState;
use axum::{
    async_trait,
    http::{request::Parts, StatusCode},
};
use spring::config::{ConfigRegistry, Configurable};
use std::ops::{Deref, DerefMut};

/// Extract the components registered by the plugin from AppState
pub struct Component<T: Clone>(pub T);

#[async_trait]
impl<T> FromRequestParts<AppState> for Component<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match state.app.get_component::<T>() {
            Some(component) => Ok(Component(T::clone(&component))),
            None => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "server component not found",
            )),
        }
    }
}

impl<T: Clone> Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Config<T>(pub T)
where
    T: serde::de::DeserializeOwned + Configurable;

#[async_trait]
impl<T> FromRequestParts<AppState> for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match state.app.get_config::<T>() {
            Ok(config) => Ok(Config(config)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "get server config failed for typeof {}: {}",
                    std::any::type_name::<T>(),
                    e
                ),
            )),
        }
    }
}

impl<T> Deref for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
