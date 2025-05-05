use spring::{app::AppBuilder, plugin::Plugin};
use tonic::async_trait;

/// Grpc Plugin Definition
pub struct GrpcPlugin;

#[async_trait]
impl Plugin for GrpcPlugin {
    async fn build(&self, _app: &mut AppBuilder) {
        // TODO: Implement the integration logic for the GrpcPlugin
    }
}
