use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use jsonwebtoken::{errors::Result, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spring_web::async_trait;
use spring_web::axum::http::request::Parts;
use spring_web::axum::http::StatusCode;
use spring_web::axum::response::IntoResponse;
use spring_web::axum::response::Response;
use spring_web::axum::Json;
use spring_web::axum::RequestPartsExt;
use spring_web::extractor::FromRequestParts;

lazy_static! {
    static ref DECODE_KEY: DecodingKey =
        DecodingKey::from_rsa_pem(include_bytes!("../keys/public.key"))
            .expect("public key parse failed");
    static ref ENCODE_KEY: EncodingKey =
        EncodingKey::from_rsa_pem(include_bytes!("../keys/private.key"))
            .expect("private key parse failed");
}

const ISSUER: &'static str = "Spring-RS web-example";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub uid: i64,
    iss: String,
}

impl Claims {
    pub fn new(uid: i64) -> Self {
        Self {
            uid,
            iss: String::from(ISSUER),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let claims = decode(bearer.token()).map_err(|_| AuthError::InvalidToken)?;

        Ok(claims)
    }
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

/// JWT encode
pub fn encode(claims: Claims) -> Result<String> {
    jsonwebtoken::encode::<Claims>(&Header::new(Algorithm::RS256), &claims, &ENCODE_KEY)
}

/// JWT decode
pub fn decode(token: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    jsonwebtoken::decode::<Claims>(&token, &DECODE_KEY, &validation)
        .map(|token_data| token_data.claims)
}
