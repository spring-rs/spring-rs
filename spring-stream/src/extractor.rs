use spring_boot::async_trait;

#[async_trait]
pub trait FromMsg {
    async fn from_msg() -> Self;
}
