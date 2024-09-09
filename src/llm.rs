use async_openai_wasm::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    },
    Client,
};
use leptos::*;
#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
static API_KEY: OnceLock<String> = OnceLock::new();

#[cfg(feature = "ssr")]
pub fn set_api_key(api_key: String) {
    let _ = API_KEY.set(api_key);
}

#[server]
pub async fn query_openai(prompt: String) -> Result<CreateChatCompletionResponse, ServerFnError> {
    use send_wrapper::SendWrapper;

    SendWrapper::new(async move {
        let api_key = API_KEY.get().expect("OPENAI_API_KEY must be set");
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://openrouter.ai/api/v1");
        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(128_u16)
            // .model("qwen/qwen-2-7b-instruct:free")
            .model("qwen/qwen-2-7b-instruct")
            // .model("nousresearch/hermes-3-llama-3.1-405b")
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let response = client.chat().create(request).await?;
        Ok(response)
    })
    .await
}
