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
        <p class="mx-2 mb-1">
            <input
                type="text"
                prop:value=source_str
                on:input=move |ev| {
                    src.set(pack(event_target_value(&ev)));
                }
                class="w-full py-1 px-2 border border-gray-300"
            />
        </p>
    }
}

#[component]
pub fn WordList(words: impl Fn() -> String + Copy + 'static) -> impl IntoView {
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

    let chinese_resource = create_local_resource_with_initial_value(
        move || source_throttled.get(),
        |src| async move {
            match src {
                Source::Chinese(txt) => txt,
                Source::English(txt) => llm::english_to_chinese(txt).await.unwrap_or_default(),
                Source::Pinyin(txt) => llm::pinyin_to_chinese(txt).await.unwrap_or_default(),
            }
        },
        Some("我忘记带瑞典食物了。".to_string()),
    );

    let english_resource = create_local_resource_with_initial_value(
        move || chinese_resource.get().unwrap_or_default(),
        move |chin| async move {
            match source.get() {
                Source::English(txt) => txt,
                _ => llm::chinese_to_english(chin).await.unwrap_or_default(),
            }
        },
        Some("I forgot to bring the keys.".to_string()),
    );

    let pinyin_resource = create_local_resource_with_initial_value(
        move || chinese_resource.get().unwrap_or_default(),
        move |chin| async move {
            match source.get() {
                Source::Pinyin(txt) => txt,
                _ => llm::chinese_to_pinyin(chin).await.unwrap_or_default(),
            }
        },
        Some("Wǒ wàngjì dài yàoshi le.".to_string()),
    );

    let chinese_memo = create_memo(move |_| {
        source
            .get()
            .get_chinese()
            .unwrap_or(chinese_resource.get().unwrap_or_default())
    });

    view! {
        <dl>
            <dt class="font-bold">
                Chinese
                {move || {
                    view! { <span class:loader=chinese_resource.loading().get()></span> }
                }}
            </dt>
            <dd class="mb-4">
                <SourceField src=source unpack=Source::get_chinese pack=Source::Chinese resource=chinese_resource />
            </dd>
            <dt class="font-bold">
                Pinyin
                {move || {
                    view! { <span class:loader=pinyin_resource.loading().get()></span> }
                }}
            </dt>
            <dd class="mb-4">
                <SourceField src=source unpack=Source::get_pinyin pack=Source::Pinyin resource=pinyin_resource />
            </dd>
            <dt class="font-bold">
                English
                {move || {
                    view! { <span class:loader=english_resource.loading().get()></span> }
                }}
            </dt>
            <dd class="mb-4">
                <SourceField src=source unpack=Source::get_english pack=Source::English resource=english_resource />
            </dd>
        </dl>
        <fieldset class="border border-black border-dashed p-2">
            <legend>Words</legend>
            <WordList words=move || chinese_memo.get() />
        </fieldset>
    }
}

#[component]
pub fn AppWithFallback() -> impl IntoView {
    let dict = DictContext::use_context();

    view! {
        <p
            class="text-center font-bold text-2xl h-96 flex items-center justify-center"
            class:hidden=move || !dict.loading()
        >
            Please wait, loading dictionary
            <span class:loader=true></span>
        </p>

        <div class:hidden=move || dict.loading()>
            <Dictionary />
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    DictContext::provide_context();

    view! {
        <Body class="bg-sky-100" />

        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta http-equiv="content-type" content="text/html; charset=utf-8" />
        <Stylesheet href="/pkg/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/pkg/favicon.ico" />
        <div class="w-full h-128 bg-gradient-to-b from-sky-700 from-30% to-sky-100"></div>

        <h1 class="text-6xl font-bold text-center pt-6 mb-2 -mt-128 text-white">"Erudify Dictionary"</h1>
        <h2 class="text-2xl text-center mb-6 text-white">"Chinese-English-Pinyin"</h2>

        <div class="max-w-4xl mx-auto p-4 bg-white" style:box-shadow="0 0px 5px rgba(0, 0, 0, 0.4)">
            <AppWithFallback />
        </div>
    }
}
