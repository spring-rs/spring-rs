use anyhow::Context;
use spring::{auto_config, App};
use spring_openai::config::OpenAIConfig;
use spring_openai::v1::chat_completion;
use spring_openai::v1::chat_completion::ChatCompletionRequest;
use spring_web::extractor::Config;
use spring_web::get;
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    WebConfigurator, WebPlugin,
};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new().add_plugin(WebPlugin).run().await
}

#[get("/chat")]
async fn send_mail(Config(config): Config<OpenAIConfig>) -> Result<impl IntoResponse> {
    let mut openai = config.build()?;
    let req = ChatCompletionRequest::new(
        "deepseek/deepseek-r1-0528-qwen3-8b:free".to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("What is bitcoin?")),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
    );
    let resp = openai
        .chat_completion(req)
        .await
        .context("chat_completion 调用失败")?;
    Ok(Json(resp))
}
