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
            .temperature(0.0) // Set temperature to zero
            // .model("qwen/qwen-2-7b-instruct:free")
            // .model("qwen/qwen-2-7b-instruct")
            .model("openai/gpt-4o-mini")
            // .model("qwen/qwen-2-72b-instruct")
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
pub async fn chinese_to_english(chinese: String) -> Result<String, ServerFnError> {
    query_openai(
        "You are a Chinese to English translation system. You will respond only with translations.".to_string(),
        vec![
            ("你需要哪本书？".into(), "Which book do you need?".into()),
            ("这只苹果有半公斤。".into(), "This apple weighs half a kilogram.".into()),
            ("".into(), "".into()),
            ("她正在打电话。".into(), "She is making a phone call.".into()),
        ],
        chinese,
    )
    .await
}

// English to Chinese translation
pub async fn english_to_chinese(english: String) -> Result<String, ServerFnError> {
    query_openai(
        "You are an English to Chinese translation system. You will respond only with translations.".to_string(),
        vec![
            ("Which book do you need?".into(), "你需要哪本书？".into()),
            ("This apple weighs half a kilogram.".into(), "这只苹果有半公斤。".into()),
            ("".into(), "".into()),
            ("She is making a phone call.".into(), "她正在打电话。".into()),
        ],
        english,
    )
    .await
}

// Chinese to Pinyin translation
pub async fn chinese_to_pinyin(chinese: String) -> Result<String, ServerFnError> {
    query_openai(
        "You are a Chinese to Pinyin translation system. You will respond only with Pinyin translations.".to_string(),
        vec![
            ("你需要哪本书？".into(), "Nǐ xūyào nǎ běn shū?".into()),
            ("这只苹果有半公斤。".into(), "Zhè zhī píngguǒ yǒu bàn gōngjīn.".into()),
            ("".into(), "".into()),
            ("她正在打电话。".into(), "Tā zhèngzài dǎ diànhuà.".into()),
        ],
        chinese,
    )
    .await
}

// Pinyin to Chinese translation
pub async fn pinyin_to_chinese(pinyin: String) -> Result<String, ServerFnError> {
    query_openai(
        "You are a Pinyin to Chinese translation system. You will respond only with Chinese translations.".to_string(),
        vec![
            ("Nǐ xūyào nǎ běn shū?".into(), "你需要哪本书？".into()),
            ("Zhè zhī píngguǒ yǒu bàn gōngjīn.".into(), "这只苹果有半公斤。".into()),
            ("".into(), "".into()),
            ("Tā zhèngzài dǎ diànhuà.".into(), "她正在打电话。".into()),
        ],
        pinyin,
    )
    .await
}
