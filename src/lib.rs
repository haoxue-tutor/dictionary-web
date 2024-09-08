use haoxue_dict::{DictEntry, Dictionary};
use leptos::{component, create_signal, view, IntoView, SignalGet, SignalSet};
use leptos_meta::*;

use leptos::*;

mod dict_context;
use dict_context::DictContext;

#[component]
pub fn InputField(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <input
            type="text"
            value=value
            on:input=move |ev| {
                set_value.set(event_target_value(&ev));
            }
            class="w-full mx-2 my-1 px-2 py-1 border border-gray-300"
        />
    }
}

#[component]
pub fn WordList(#[prop(into)] words: Signal<String>) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || {
                    let dict = DictContext::use_context().get();
                    if let Some(dict) = dict {
                        dict.segment(&words.get()).into_iter().map(|x| x.right_or_else(DictEntry::simplified).to_string()).collect::<Vec<_>>()
                    } else {
                        words.get().split_whitespace().map(ToString::to_string).collect::<Vec<String>>()
                    }
                }
                key=|word| word.to_string()
                let:word
            >
                <li>{word}</li>
            </For>
        </ul>
    }
}

#[component]
pub fn Dictionary() -> impl IntoView {
    let (input, set_input) = create_signal(String::from("我忘记带钥匙了。"));

    let dict = DictContext::use_context();

    view! {
        <div>
            <h2>"Dictionary"</h2>
            <InputField
                value=input
                set_value=set_input
            />
            <WordList
                words=input
            />
            <p>
                {move || {
                    let dict = dict.get();
                    if dict.is_none() {
                        "The dictionary is empty."
                    } else {
                        "Dictionary available"
                    }
                }}
            </p>
        </div>
    }
}

#[component]
pub fn FileDownloader() -> impl IntoView {
    use futures_util::StreamExt;
    let (progress, set_progress) = create_signal(0.0);
    let (is_downloading, set_is_downloading) = create_signal(false);

    let dict = DictContext::use_context();

    let download = move |_| {
        set_is_downloading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            // let url = "https://github.com/haoxue-tutor/haoxue-dict/raw/main/data/cedict-2024-06-07.txt"; // Replace with your file URL
            let url = "https://assets.lemmih.org/cedict-2024-06-07.txt";
            let client = reqwest::Client::new();
            let response = client.get(url).send().await.unwrap();

            let total_size = response
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|cl| cl.to_str().ok())
                .and_then(|cl| cl.parse::<f64>().ok())
                .unwrap_or(0.0);

            let mut received = 0.0;
            let mut buffer = Vec::with_capacity(total_size as usize);

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.unwrap();
                received += chunk.len() as f64;
                buffer.extend_from_slice(&chunk);
                set_progress.set((received / total_size) * 100.0);
            }

            set_is_downloading.set(false);
            log::info!(
                "File download completed successfully! Total size: {:.2} MB",
                received / (1024.0 * 1024.0)
            );

            // Convert the received data to a string
            let file_content = String::from_utf8(buffer).unwrap_or_else(|_| {
                log::error!("Failed to convert file content to UTF-8");
                String::new()
            });

            let url = "https://assets.lemmih.org/SUBTLEX-CH-WF.utf8.txt";
            let subtlex_content = reqwest::get(url).await.unwrap().text().await.unwrap();

            use instant::Instant;

            let start_time = Instant::now();
            let new_dict = Dictionary::new_from_reader(
                std::io::Cursor::new(file_content),
                std::io::Cursor::new(subtlex_content),
            );
            let duration = start_time.elapsed();
            log::info!("Time taken to create dictionary: {:.2?}", duration);
            dict.set(new_dict);
        });
    };

    view! {
        <div>
            <button
                on:click=download
                disabled=is_downloading
            >
                "Download Large File"
            </button>
            {move || if is_downloading.get() {
                view! {
                    <div>
                        <progress value=progress max="100"></progress>
                    </div>
                    <div>{move || format!("{:.1}%", progress.get())}</div>
                }.into_view()
            } else {
                view! { <p>"Click to start download"</p> }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    DictContext::provide_context();

    view! {
        <Stylesheet href="/pkg/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/pkg/favicon.ico" />
        <p>"Dictionary app"</p>
        <FileDownloader />
        <Dictionary />
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[cfg(feature = "ssr")]
mod ssr_imports {
    use crate::App;
    use axum::http::{HeaderValue, StatusCode};
    use axum::{
        extract::Path,
        response::IntoResponse,
        routing::{get, post},
        Router,
    };
    use include_dir::{include_dir, Dir};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use worker::{event, Context, Env, HttpRequest, Result};

    static PKG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/pkg/");

    async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
        let mime_type = mime_guess::from_path(&path).first_or_text_plain();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        );
        match PKG_DIR.get_file(path) {
            None => (StatusCode::NOT_FOUND, headers, "File not found.".as_bytes()),
            Some(file) => (StatusCode::OK, headers, file.contents()),
        }
    }

    fn router() -> Router {
        let leptos_options = LeptosOptions::builder()
            .output_name("client")
            .site_pkg_dir("pkg")
            .build();
        let routes = generate_route_list(App);

        // build our application with a route
        let app: axum::Router<()> = Router::new()
            .leptos_routes(&leptos_options, routes, App)
            .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
            .route("/pkg/*file_name", get(serve_static))
            .with_state(leptos_options);
        app
    }

    #[event(start)]
    fn register() {
        // No server functions to register
    }

    #[event(fetch)]
    async fn fetch(
        req: HttpRequest,
        _env: Env,
        _ctx: Context,
    ) -> Result<axum::http::Response<axum::body::Body>> {
        _ = console_log::init_with_level(log::Level::Debug);
        use tower_service::Service;

        console_error_panic_hook::set_once();

        Ok(router().call(req).await?)
    }
}
