use haoxue_dict::Dictionary;
use leptos::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct DictContext {
    signal: Signal<Option<Arc<Dictionary>>>,
}

impl DictContext {
    fn new() -> Self {
        let (signal, set_signal) = create_signal(None);
        if !cfg!(feature = "ssr") {
            spawn_local(async move {
                let url = "https://assets.erudify.org/cedict-2024-06-07.txt";
                let client = reqwest::Client::new();
                let response = client.get(url).send().await.unwrap();

                let file_content = response.text().await.unwrap();

                log::info!(
                    "File download completed successfully! Total size: {:.2} MB",
                    file_content.len() as f64 / (1024.0 * 1024.0)
                );

                let url = "https://assets.erudify.org/SUBTLEX-CH-WF.utf8.txt";
                let subtlex_content = reqwest::get(url).await.unwrap().text().await.unwrap();

                use instant::Instant;

                let start_time = Instant::now();
                let new_dict = Dictionary::new_from_reader(
                    std::io::Cursor::new(file_content),
                    std::io::Cursor::new(subtlex_content),
                );
                let duration = start_time.elapsed();
                log::info!("Time taken to create dictionary: {:.2?}", duration);

                set_signal.set(Some(Arc::new(new_dict)));
            });
        }
        Self { signal: signal.into() }
    }
    pub fn get(&self) -> Option<Arc<Dictionary>> {
        self.signal.get()
    }

    pub fn loading(&self) -> bool {
        self.signal.try_get().flatten().is_none()
    }

    pub fn provide_context() {
        provide_context(Self::new());
    }

    pub fn use_context() -> Self {
        use_context::<Self>().expect("DictContext should be provided")
    }
}
