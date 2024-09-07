// use either_of::Either;
use leptos::{
    component, create_signal, event_target_value, server, spawn_local, view, For, IntoView,
    ServerFnError, SignalSet as _,
};
use leptos_meta::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Exercise {
    pub segments: Vec<Segment>,
    pub english: String,
}

fn ex_1() -> Exercise {
    Exercise {
        segments: vec![
            Segment {
                chinese: String::from("我"),
                pinyin: String::from("wǒ"),
            },
            Segment {
                chinese: String::from("是"),
                pinyin: String::from("shì"),
            },
            Segment {
                chinese: String::from("学生"),
                pinyin: String::from("xué sheng"),
            },
            Segment {
                chinese: String::from("。"),
                pinyin: String::from(""),
            },
        ],
        english: String::from("I am a student."),
    }
}
fn ex_2() -> Exercise {
    Exercise {
        segments: vec![
            Segment {
                chinese: String::from("明天"),
                pinyin: String::from("míng tiān"),
            },
            Segment {
                chinese: String::from("我"),
                pinyin: String::from("wǒ"),
            },
            Segment {
                chinese: String::from("会"),
                pinyin: String::from("huì"),
            },
            Segment {
                chinese: String::from("去"),
                pinyin: String::from("qù"),
            },
            Segment {
                chinese: String::from("图书馆"),
                pinyin: String::from("tú shū guǎn"),
            },
            Segment {
                chinese: String::from("。"),
                pinyin: String::from(""),
            },
        ],
        english: String::from("I will go to the library tomorrow."),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub chinese: String,
    pub pinyin: String,
}

#[server]
pub async fn shouting_text(input: String) -> Result<String, ServerFnError> {
    log::info!("Server-side fn: {}", &input);
    // insert a simulated wait
    Ok(input.to_ascii_uppercase())
}

#[component]
pub fn SizedInput() -> impl IntoView {
    let (value, set_value) = create_signal(String::new());
    view! {
        <input type="text" style:width="5em" autofocus on:input=move |ev| {
            ev.prevent_default();
            log::info!("Input: {}", event_target_value(&ev));
            set_value.set(event_target_value(&ev).to_uppercase());
        }
            prop:value=value/>
    }
    // view! { "Empty" }
}

#[component]
pub fn AudioExercise(exercise: Exercise, index: usize) -> impl IntoView {
    log::info!("index: {index}");
    view! {
        <form>
            <For
                each=move || exercise.segments.clone().into_iter().enumerate()
                key=|(idx, segment)| segment.chinese.clone()
                let:segment
            >
                {move || {
                    let width = segment.1.chinese.chars().count();
                    let x = if segment.0 == index {
                        view! {
                            <SizedInput/>
                        }.into_view()
                    } else {
                        view! { {segment.1.chinese.to_string()} }.into_view()
                    };
                    view! { {x} }
                }}
                // {move || if segment.0 == 0 {
                //     view! { <p>"Input"</p> }
                // } else {
                //     view! { <p>{segment.1.chinese}</p> }
                // }}
                // {segment.1.chinese}
            </For>
        // for (nth , segment) in exercise.segments.iter().enumerate() {
        // if nth < index || segment.pinyin.is_empty() {
        // span {
        // class: "done",
        // style: "width: {segment.chinese.chars().count() as i64}em",
        // "{segment.chinese} "
        // }
        // } else if nth == index {
        // input {
        // style: "width: {segment.chinese.chars().count() as i64}em;",
        // name: "input",
        // onmounted: move |cx| {
        // spawn(async move {
        // let _ = cx.data().set_focus(true).await;
        // });
        // },
        // value: "{input}",
        // oninput: move |event| oninput.call(event)
        // }
        // } else {
        // span {
        // class: "future",
        // style: "width: {segment.chinese.chars().count() as i64}em",
        // "{segment.chinese} "
        // }
        // }
        // }
        </form>
    }
}
// #[component]
// fn AudioExercise(
//     exercise: Exercise,
//     index: usize,
//     input: Signal<String>,
//     onsubmit: EventHandler<FormEvent>,
//     oninput: EventHandler<FormEvent>,
// ) -> Element {
//     rsx! {
//         form { class: "exercise", onsubmit: move |event| onsubmit.call(event),
//             for (nth , segment) in exercise.segments.iter().enumerate() {
//                 if nth < index || segment.pinyin.is_empty() {
//                     span {
//                         class: "done",
//                         style: "width: {segment.chinese.chars().count() as i64}em",
//                         "{segment.chinese} "
//                     }
//                 } else if nth == index {
//                     input {
//                         style: "width: {segment.chinese.chars().count() as i64}em;",
//                         name: "input",
//                         onmounted: move |cx| {
//                             spawn(async move {
//                                 let _ = cx.data().set_focus(true).await;
//                             });
//                         },
//                         value: "{input}",
//                         oninput: move |event| oninput.call(event)
//                     }
//                 } else {
//                     span {
//                         class: "future",
//                         style: "width: {segment.chinese.chars().count() as i64}em",
//                         "{segment.chinese} "
//                     }
//                 }
//             }
//         }
//         button { class: "btn btn-blue", "Replay audio" }
//         button { class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
//             "Next exercise"
//         }
//     }
// }

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet href="/pkg/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/pkg/favicon.ico" />
        <p>Dictionary app</p>
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
        server_fn::axum::register_explicit::<super::ShoutingText>();
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
