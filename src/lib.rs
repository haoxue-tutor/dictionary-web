use haoxue_dict::DictEntry;
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
                        dict.segment(&words.get()).into_iter().map(|either| either.right_or_else(DictEntry::simplified).to_string()).collect::<Vec<_>>()
                    } else {
                        vec![] //words.get().split_whitespace().map(ToString::to_string).collect::<Vec<String>>()
                    }
                }
                key=|word| word.clone()
                let:word
            >
                <li>{{
                    let dict = DictContext::use_context().get();
                    if let Some(dict) = dict {
                        // view! { {word} }
                        if let Some(entry) = dict.get_entry(&word) {
                            view! { {word} {entry.definitions().next().unwrap_or_default().to_string()} }.into_view()
                        } else {
                            view! { {word} }.into_view()
                        }
                    } else {
                        view! { {word} }.into_view()
                    }
                    }}
                </li>
            </For>
        </ul>
    }
}

#[component]
pub fn Dictionary() -> impl IntoView {
    let (input, set_input) = create_signal(String::from("我忘记带钥匙了。"));

    view! {
        <div>
            <InputField
                value=input
                set_value=set_input
            />
            <WordList
                words=input
            />
        </div>
    }
}

#[component]
pub fn FileDownloader() -> impl IntoView {
    let dict = DictContext::use_context();

    view! {
        <Suspense fallback=move || view! { <p> Please wait, loading dictionary <span class:loader=true></span></p> }.into_view()>
        <div>
            {move || {
                let _ = dict.get();
                view! { }
            }}
        </div>
        </Suspense>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    DictContext::provide_context();

    view! {
        <Stylesheet href="/pkg/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/pkg/favicon.ico" />
        <h1 class="text-4xl font-bold text-center my-6">"Chinese to English Dictionary"</h1>
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
