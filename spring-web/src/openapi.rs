use aide::{
    generate::GenContext,
    openapi::{Operation, ReferenceOr, Response, StatusCode},
    Error,
};

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
        Some(status) => {
            if responses.responses.contains_key(&status) {
                ctx.error(Error::InferredResponseConflict(status.to_string()));
            } else {
                responses.responses.insert(status, ReferenceOr::Item(res));
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
