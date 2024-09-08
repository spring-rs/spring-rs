use sea_orm::{ConnectionTrait, EntityTrait, FromQueryResult, PaginatorTrait, Select};
use serde::{Deserialize, Serialize};
use spring::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
}
fn default_page() -> u64 {
    0
}
fn default_size() -> u64 {
    20
}

#[cfg(feature = "with-web")]
mod web {
    use super::Pagination;
    use crate::config::SeaOrmWebConfig;
    use serde::Deserialize;
    use spring::async_trait;
    use spring::config::ConfigRegistry;
    use spring_web::axum::extract::rejection::QueryRejection;
    use spring_web::axum::extract::{FromRequestParts, Query};
    use spring_web::axum::http::request::Parts;
    use spring_web::axum::http::StatusCode;
    use spring_web::axum::response::IntoResponse;
    use spring_web::AppState;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum SeaOrmWebErr {
        #[error(transparent)]
        QueryRejection(#[from] QueryRejection),

        #[error(transparent)]
        AppConfigError(#[from] spring::error::AppError),
    }

    impl IntoResponse for SeaOrmWebErr {
        fn into_response(self) -> spring_web::axum::response::Response {
            match self {
                Self::QueryRejection(e) => e.into_response(),
                Self::AppConfigError(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("sea orm get pagination config failed: {}", e),
                )
                    .into_response(),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    struct OptionalPagination {
        page: Option<u64>,
        size: Option<u64>,
    }

    #[async_trait]
    impl FromRequestParts<AppState> for Pagination {
        type Rejection = SeaOrmWebErr;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &AppState,
        ) -> std::result::Result<Self, Self::Rejection> {
            let Query(pagination) = Query::<OptionalPagination>::try_from_uri(&parts.uri)?;

            let config = state.app.get_config::<SeaOrmWebConfig>()?;

            let size = match pagination.size {
                Some(size) => {
                    if size > config.max_page_size {
                        config.max_page_size
                    } else {
                        size
                    }
                }
                None => config.default_page_size,
            };

            let page = if config.one_indexed {
                pagination
                    .page
                    .map(|page| if page == 0 { 0 } else { page - 1 })
                    .unwrap_or(0)
            } else {
                pagination.page.unwrap_or(0)
            };

            Ok(Pagination { page, size })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    content: Vec<T>,
    size: u64,
    page: u64,
    total_elements: u64,
    total_pages: u64,
}

impl<T> Page<T> {
    pub fn new(content: Vec<T>, pagination: Pagination, total: u64) -> Self {
        Self {
            content,
            size: pagination.size,
            page: pagination.page,
            total_elements: total,
            total_pages: Self::total_pages(total, pagination.size),
        }
    }

    /// Compute the number of pages for the current page
    fn total_pages(total: u64, size: u64) -> u64 {
        (total / size) + (total % size > 0) as u64
    }
}

#[derive(Debug, Error)]
pub enum OrmError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}

pub type PageResult<T> = std::result::Result<Page<T>, OrmError>;

#[async_trait]
/// A Trait for any type that can paginate results
pub trait PaginationExt<'db, C, M>
where
    C: ConnectionTrait,
{
    /// pagination
    async fn page(self, db: &'db C, pagination: Pagination) -> PageResult<M>;
}

#[async_trait]
impl<'db, C, M, E> PaginationExt<'db, C, M> for Select<E>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
{
    async fn page(self, db: &'db C, pagination: Pagination) -> PageResult<M> {
        let total = self.clone().paginate(db, 1).num_items().await?;
        let content = self
            .paginate(db, pagination.size)
            .fetch_page(pagination.page)
            .await?;

        Ok(Page::new(content, pagination, total))
    }
}
