// The bookmarks page: Leptos, compiled to WebAssembly.
// The API address is set when building: API_URL=https://... trunk build --release

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API: &str = match option_env!("API_URL") {
    Some(url) => url,
    None => "http://localhost:8090",
};

#[derive(Clone, Deserialize)]
struct Bookmark {
    id: i64,
    title: String,
    url: String,
}

#[derive(Serialize)]
struct NewBookmark {
    title: String,
    url: String,
}

async fn fetch_bookmarks() -> Vec<Bookmark> {
    match Request::get(&format!("{API}/api/bookmarks")).send().await {
        Ok(response) => response.json().await.unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
fn App() -> impl IntoView {
    // Signals are values the page watches: when one changes, the page updates.
    let bookmarks = RwSignal::new(Vec::<Bookmark>::new());
    let title = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());

    // Load the list when the page opens.
    leptos::task::spawn_local(async move { bookmarks.set(fetch_bookmarks().await) });

    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let new = NewBookmark { title: title.get(), url: url.get() };
        leptos::task::spawn_local(async move {
            let _ = Request::post(&format!("{API}/api/bookmarks"))
                .json(&new)
                .unwrap()
                .send()
                .await;
            title.set(String::new());
            url.set(String::new());
            bookmarks.set(fetch_bookmarks().await);
        });
    };

    let remove = move |id: i64| {
        leptos::task::spawn_local(async move {
            let _ = Request::delete(&format!("{API}/api/bookmarks/{id}")).send().await;
            bookmarks.set(fetch_bookmarks().await);
        });
    };

    view! {
        <h1>"Bookmarks"</h1>
        <form on:submit=add>
            <input
                placeholder="Title"
                required
                prop:value=move || title.get()
                on:input:target=move |event| title.set(event.target().value())
            />
            <input
                placeholder="https://..."
                required
                prop:value=move || url.get()
                on:input:target=move |event| url.set(event.target().value())
            />
            <button type="submit">"Add"</button>
        </form>
        <ul>
            <For
                each=move || bookmarks.get()
                key=|bookmark| bookmark.id
                children=move |bookmark| {
                    let id = bookmark.id;
                    view! {
                        <li>
                            <a href=bookmark.url target="_blank">{bookmark.title}</a>
                            <button on:click=move |_| remove(id)>"Delete"</button>
                        </li>
                    }
                }
            />
        </ul>
    }
}

fn main() {
    leptos::mount::mount_to_body(App);
}
