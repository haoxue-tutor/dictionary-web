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
pub async fn query_openai(
    system: String,
    history: Vec<(String, String)>,
    prompt: String,
) -> Result<String, ServerFnError> {
    use async_openai_wasm::{config::OpenAIConfig, types::*, Client};

    use send_wrapper::SendWrapper;

    let mut messages: Vec<ChatCompletionRequestMessage> = vec![];
    messages.push(ChatCompletionRequestSystemMessage::from(system).into());
    for (req, resp) in history {
        messages.push(ChatCompletionRequestUserMessage::from(req).into());
        messages.push(ChatCompletionRequestAssistantMessage::from(resp).into());
    }
    messages.push(ChatCompletionRequestUserMessage::from(prompt).into());

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
            .messages(messages)
            .build()
            .unwrap();

        let response = client.chat().create(request).await?;
        let choice = response.choices.first().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Received empty set of choices",
        ))?;
        Ok(choice.message.content.clone().unwrap_or_default())
    })
    .await
}

// Chinese to English translation
pub async fn translate(chinese: String) -> Result<String, ServerFnError> {
    query_openai(
        "You are a Chinese to English translation system. You will respond only with translations."
            .to_string(),
        vec![
            ("你需要哪本书？".into(), "Which book do you need?".into()),
            (
                "这只苹果有半公斤。".into(),
                "Zhè zhī píngguǒ yǒu bàn gōngjīn.".into(),
            ),
            ("".into(), "".into()),
            (
                "她正在打电话。".into(),
                "She is making a phone call.".into(),
            ),
        ],
        chinese,
    )
    .await
}
