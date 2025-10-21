use aide::{generate::GenContext, openapi::{MediaType, Operation, ReferenceOr, Response, SchemaObject, StatusCode}, Error};

/// Helper trait to optionally get JsonSchema
/// This is implemented for all types, but only returns Some for types that implement JsonSchema
pub trait MaybeJsonSchema {
    fn maybe_schema() -> Option<schemars::Schema> {
        None
    }
}

pub trait HttpStatusCodeVariantInfo {
    fn get_variant_info(variant_name: &str) -> Option<(u16, String, Option<schemars::Schema>)>;
}

pub fn set_inferred_response(
    ctx: &mut GenContext,
    operation: &mut Operation,
    status: Option<u16>,
    res: Response,
) {
    if operation.responses.is_none() {
        operation.responses = Some(Default::default());
    }

    let responses = operation.responses.as_mut().unwrap();

    match status {
        Some(status) => {
            let status_code_key = StatusCode::Code(status);
            let Some(existing) = responses.responses.get_mut(&status_code_key) else {
                responses
                    .responses
                    .insert(status_code_key, ReferenceOr::Item(res));
                return;
            };

            let ReferenceOr::Item(existing_response) = existing else {
                return;
            };

            if existing_response.description != res.description {
                existing_response.description = format!("- {}\n- {}", existing_response.description, res.description);
            }
        }
        None => {
            if responses.default.is_some() {
                ctx.error(Error::InferredDefaultResponseConflict);
            } else {
                responses.default = Some(ReferenceOr::Item(res));
            }
        }
    }
}

pub fn register_error_response_by_variant<T>(
    ctx: &mut GenContext,
    operation: &mut Operation,
    variant_path: &str,
) where
    T: crate::HttpStatusCode + HttpStatusCodeVariantInfo,
{
    let variant_name = variant_path.split("::").last().unwrap_or(variant_path);
    
    let Some((status_code, description, schema_opt)) = T::get_variant_info(variant_name) else {
        tracing::warn!("Variant '{}' not found in error type when registering OpenAPI responses", variant_name);
        return;
    };
    
    let mut response = Response {
        description,
        ..Default::default()
    };
    
    if let Some(schema) = schema_opt {
        response.content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Some(SchemaObject {
                    json_schema: schema,
                    example: None,
                    external_docs: None,
                }),
                ..Default::default()
            }
        );
    }
    
    set_inferred_response(ctx, operation, Some(status_code), response);
}
