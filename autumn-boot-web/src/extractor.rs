use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::AppState;

pub struct Component<T>(pub Arc<T>);

#[async_trait]
impl<T> FromRequestParts<AppState> for Component<T>
where
    T: Send + Sync + 'static,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match state.app.get_component::<T>() {
            Some(component) => Ok(Component(component)),
            None => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "server component not found",
            )),
        }
    }
}
