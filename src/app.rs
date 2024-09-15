use haoxue_dict::DictEntry;
use leptos::{component, view, IntoView, SignalGet, SignalSet};
use leptos_meta::*;

use leptos::*;
use leptos_use::*;

use crate::dict_context::DictContext;
use crate::llm;

#[component]
pub fn SourceField<A: Clone + 'static>(
    #[prop(into)] src: RwSignal<Source>,
    pack: fn(String) -> Source,
    unpack: fn(&Source) -> Option<String>,
    resource: Resource<A, String>,
) -> impl IntoView {
    let source_str = move || src.with(unpack).unwrap_or_else(|| resource.get().unwrap_or_default());
    view! {
        <p>
            <input
                type="text"
                prop:value=source_str
                on:input=move |ev| {
                    src.set(pack(event_target_value(&ev)));
                }
                class="w-1/2 mx-2 my-1 px-2 py-1 border border-gray-300"
            />
            {move || {
                view! {
                    <span class:loader=resource.loading().get()
                        && src.with(unpack).is_none()></span>
                }
            }}
        </p>
    }
}

#[component]
pub fn WordList(words: impl Fn() -> String + 'static) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || {
                    let dict = DictContext::use_context().get();
                    if let Some(dict) = dict {
                        dict.segment(&words())
                            .into_iter()
                            .map(|either| either.right_or_else(DictEntry::simplified).to_string())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                }
                key=|word| word.clone()
                let:word
            >
                <li>
                    {{
                        let dict = DictContext::use_context().get();
                        if let Some(dict) = dict {
                            if let Some(entry) = dict.get_entry(&word) {
                                view! {
                                    <span lang="zh">{word}</span>
                                    {" "}
                                    {entry.definitions().next().unwrap_or_default().to_string()}
                                }
                                    .into_view()
                            } else {
                                view! { <span lang="zh">{word}</span> }.into_view()
                            }
                        } else {
                            view! { <span lang="zh">{word}</span> }.into_view()
                        }
                    }}
                </li>
            </For>
        </ul>
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Chinese(String),
    English(String),
    Pinyin(String),
}

impl Source {
    fn get_chinese(&self) -> Option<String> {
        match self {
            Source::Chinese(text) => Some(text.clone()),
            _ => None,
        }
    }

    fn get_english(&self) -> Option<String> {
        match self {
            Source::English(text) => Some(text.clone()),
            _ => None,
        }
    }

    fn get_pinyin(&self) -> Option<String> {
        match self {
            Source::Pinyin(text) => Some(text.clone()),
            _ => None,
        }
    }
}

#[component]
pub fn Dictionary() -> impl IntoView {
    let source = create_rw_signal(Source::Chinese(String::from("我忘记带钥匙了。")));
    let source_throttled = signal_debounced(source, 1000.0);

    let chinese_resource = create_local_resource(
        move || source_throttled.get(),
        |src| async move {
            log::debug!("chinese src: {:?}", src);
            match src {
                Source::Chinese(txt) => txt,
                Source::English(txt) => llm::english_to_chinese(txt).await.unwrap_or_default(),
                Source::Pinyin(txt) => llm::pinyin_to_chinese(txt).await.unwrap_or_default(),
            }
        },
    );

    let english_resource = create_local_resource(
        move || chinese_resource.get().unwrap_or_default(),
        move |chin| async move {
            match source.get() {
                Source::English(txt) => txt,
                _ => llm::chinese_to_english(chin).await.unwrap_or_default(),
            }
        },
    );

    let pinyin_resource = create_local_resource(
        move || chinese_resource.get().unwrap_or_default(),
        move |chin| async move {
            match source.get() {
                Source::Pinyin(txt) => txt,
                _ => llm::chinese_to_pinyin(chin).await.unwrap_or_default(),
            }
        },
    );

    view! {
        <fieldset class="border border-black border-dashed p-2">
            <legend>Chinese</legend>
            <SourceField
                src=source
                unpack=Source::get_chinese
                pack=Source::Chinese
                resource=chinese_resource
            />
        </fieldset>
        <fieldset class="border border-black border-dashed p-2">
            <legend>Pinyin</legend>
            <SourceField
                src=source
                unpack=Source::get_pinyin
                pack=Source::Pinyin
                resource=pinyin_resource
            />
        </fieldset>
        <fieldset class="border border-black border-dashed p-2">
            <legend>English</legend>
            <SourceField
                src=source
                unpack=Source::get_english
                pack=Source::English
                resource=english_resource
            />
        </fieldset>
        <fieldset class="border border-black border-dashed p-2">
            <legend>Words</legend>
            <WordList words=move || chinese_resource.get().unwrap_or_default() />
        </fieldset>
    }
}

#[component]
pub fn FileDownloader() -> impl IntoView {
    let dict = DictContext::use_context();

    view! {
        <Suspense fallback=move || {
            view! { <p>Please wait, loading dictionary <span class:loader=true></span></p> }
                .into_view()
        }>
            <div>
                {move || {
                    let _ = dict.get();
                    view! {}
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
        <div class="max-w-2xl mx-auto">
            <FileDownloader />
            <Dictionary />
        </div>
    }
}
