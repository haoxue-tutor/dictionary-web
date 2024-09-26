#[cfg(feature = "ssr")]
use async_openai_wasm::types::*;
use leptos::*;

#[server]
pub async fn query_openai(
    system: String,
    history: Vec<(String, String)>,
    prompt: String,
) -> Result<String, ServerFnError> {
    use send_wrapper::SendWrapper;

    let mut messages: Vec<ChatCompletionRequestMessage> = vec![];
    messages.push(ChatCompletionRequestSystemMessage::from(system).into());
    for (req, resp) in history {
        messages.push(ChatCompletionRequestUserMessage::from(req).into());
        messages.push(ChatCompletionRequestAssistantMessage::from(resp).into());
    }
    messages.push(ChatCompletionRequestUserMessage::from(prompt).into());

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(128_u16)
        .temperature(0.0)
        .model("openai/gpt-4o-mini")
        .messages(messages)
        .build()
        .unwrap();

    SendWrapper::new(cached_query_openai(request)).await
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

#[cfg(feature = "ssr")]
pub async fn cached_query_openai(request: CreateChatCompletionRequest) -> Result<String, ServerFnError> {
    const CACHE_TTL: u64 = 24 * 60 * 60; // 1 day in seconds
    use async_openai_wasm::{config::OpenAIConfig, Client};

    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;

    let kv = env.kv("LLM_CACHE").expect("LLM_CACHE must be set");

    let cache_key = {
        use ahash::AHasher;
        use std::hash::{Hash, Hasher};
        let json = serde_json::to_string(&request)?;
        let mut hasher = AHasher::default();
        json.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };

    // Check if the response is cached
    let start_time = instant::Instant::now();
    let cached_result = kv.get(&cache_key).cache_ttl(CACHE_TTL).text().await?;
    let duration = start_time.elapsed();
    log::info!("Time taken to query cache: {:?}", duration);

    if let Some(cached) = cached_result {
        return Ok(cached);
    }

    // If not cached, make the API call
    let api_key = env
        .secret("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set")
        .to_string();

    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base("https://openrouter.ai/api/v1");
    let client = Client::with_config(config);

    let response = client.chat().create(request.clone()).await?;
    let choice = response.choices.first().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Received empty set of choices",
    ))?;
    let content = choice.message.content.clone().unwrap_or_default();

    // Cache the response
    let start_time = instant::Instant::now();
    kv.put(&cache_key, content.clone())?
        .expiration_ttl(CACHE_TTL)
        .execute()
        .await?;
    let duration = start_time.elapsed();
    log::info!("Time taken to cache response: {:?}", duration);

    Ok(content)
}
