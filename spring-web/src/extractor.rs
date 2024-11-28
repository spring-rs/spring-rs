pub use axum::extract::*;

use crate::error::{Result, WebError};
use crate::AppState;
use axum::{async_trait, http::request::Parts};
use spring::config::{ConfigRegistry, Configurable};
use spring::plugin::ComponentRegistry;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;

/// Extending the functionality of RequestParts
pub trait RequestPartsExt {
    /// get AppState
    fn get_app_state(&self) -> &AppState;

    /// get Component
    fn get_component<T: Clone + Send + Sync + 'static>(&self) -> Result<T>;

    /// get Config
    fn get_config<T: serde::de::DeserializeOwned + Configurable>(&self) -> Result<T>;
}

impl RequestPartsExt for Parts {
    fn get_app_state(&self) -> &AppState {
        self.extensions
            .get::<AppState>()
            .expect("extract app state from extension failed")
    }

    fn get_component<T: Clone + Send + Sync + 'static>(&self) -> Result<T> {
        match self.get_app_state().app.get_component_ref::<T>() {
            Some(component) => Ok(T::clone(&component)),
            None => Err(WebError::ComponentNotExists(std::any::type_name::<T>())),
        }
    }

    fn get_config<T: serde::de::DeserializeOwned + Configurable>(&self) -> Result<T> {
        self.get_app_state()
            .app
            .get_config::<T>()
            .map_err(|e| WebError::ConfigDeserializeErr(std::any::type_name::<T>(), e))
    }
}

/// Extract the components registered by the plugin from AppState
pub struct Component<T: Clone>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Component<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, _s: &S) -> StdResult<Self, Self::Rejection> {
        parts.get_component::<T>().map(|c| Component(c))
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
impl<T, S> FromRequestParts<S> for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, _s: &S) -> StdResult<Self, Self::Rejection> {
        parts.get_config().map(|c| Config(c))
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
