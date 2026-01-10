use aide::{Error, generate::GenContext, openapi::{MediaType, Operation, ReferenceOr, Response, SchemaObject, StatusCode}};

pub trait ProblemDetailsVariantInfo {
    fn get_variant_info(variant_name: &str) -> Option<(u16, String, Option<schemars::Schema>)>;
}

/// Generate Problem Details schema for OpenAPI documentation
pub fn problem_details_schema() -> schemars::Schema {
    use schemars::JsonSchema;
    crate::problem_details::ProblemDetails::json_schema(&mut schemars::SchemaGenerator::default())
}

pub fn register_error_response_by_variant<T>(
    _ctx: &mut GenContext,
    operation: &mut Operation,
    variant_path: &str,
) where
    T: ProblemDetailsVariantInfo,
{
    let variant_name = variant_path.split("::").last().unwrap_or(variant_path);
    
    let Some((status_code, description, _schema_opt)) = T::get_variant_info(variant_name) else {
        tracing::warn!("Variant '{}' not found in error type '{}' when registering OpenAPI responses", 
                      variant_name, std::any::type_name::<T>());
        return;
    };
    
    // Create Problem Details response
    let problem_type = format!("about:blank/{}", variant_name.to_lowercase().replace("::", "-"));
    let example = serde_json::json!({
        "type": problem_type,
        "title": format!("{} Error", variant_name),
        "status": status_code,
        "detail": format!("{} occurred", variant_name)
    });
    
    let response = Response {
        description,
        content: {
            let mut content = indexmap::IndexMap::new();
            let media_type = MediaType {
                schema: Some(SchemaObject {
                    json_schema: problem_details_schema(),
                    example: Some(example),
                    external_docs: None,
                }),
                ..Default::default()
            };
            
            content.insert("application/problem+json".to_string(), media_type.clone());
            content.insert("application/json".to_string(), media_type); // backward compatibility
            content
        },
        ..Default::default()
    };
    
    // Add response to operation
    if operation.responses.is_none() {
        operation.responses = Some(Default::default());
    }
    
    let responses = operation.responses.as_mut().unwrap();
    let status_code_key = StatusCode::Code(status_code);
    
    if let Some(existing) = responses.responses.get_mut(&status_code_key) {
        // Merge descriptions if response already exists
        if let ReferenceOr::Item(existing_response) = existing {
            if existing_response.description != response.description {
                existing_response.description = format!("{}\n- {}", existing_response.description, response.description);
            }
        }
    } else {
        responses.responses.insert(status_code_key, ReferenceOr::Item(response));
    }
}

pub fn set_inferred_response(
    ctx: &mut GenContext,
    operation: &mut Operation,
    status: Option<StatusCode>,
    res: Response,
) {
    if operation.responses.is_none() {
        operation.responses = Some(Default::default());
    }

    let responses = operation.responses.as_mut().unwrap();

    match status {
        Some(status_code) => {
            let Some(existing) = responses.responses.get_mut(&status_code) else {
                responses
                    .responses
                    .insert(status_code, ReferenceOr::Item(res));
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
