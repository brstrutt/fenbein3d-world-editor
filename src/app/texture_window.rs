use crate::line::Line;
use leptos::{ev, prelude::*};

#[component]
pub fn TextureWindow(textures: RwSignal<Vec<String>>, lines: RwSignal<Vec<Line>>) -> impl IntoView {
    let minimized = RwSignal::new(false);
    let texture_draft = RwSignal::new(String::new());

    view! {
        <aside class="texture-window" aria-labelledby="texture-window-title">
            <header class="texture-window-heading">
                <div>
                    <p>"Available identifiers"</p>
                    <h2 id="texture-window-title">"Textures"</h2>
                </div>
                <button
                    class="texture-window-toggle"
                    type="button"
                    aria-label=move || {
                        if minimized.get() {
                            "Expand textures window"
                        } else {
                            "Minimize textures window"
                        }
                    }
                    title=move || {
                        if minimized.get() {
                            "Expand textures window"
                        } else {
                            "Minimize textures window"
                        }
                    }
                    aria-expanded=move || (!minimized.get()).to_string()
                    on:click=move |_| minimized.update(|minimized| *minimized = !*minimized)
                >
                    {move || if minimized.get() { "+" } else { "-" }}
                </button>
            </header>
            <section class="texture-library" hidden=move || minimized.get()>
                <form on:submit=move |event: ev::SubmitEvent| {
                    event.prevent_default();
                    add_texture_id(textures, texture_draft);
                }>
                    <input
                        class="texture-id-input"
                        type="text"
                        aria-label="Texture identifier"
                        placeholder="Texture identifier"
                        prop:value=move || texture_draft.get()
                        on:input=move |event| texture_draft.set(event_target_value(&event))
                    />
                    <button class="texture-add-button" type="submit">"Add"</button>
                </form>
                <ul class="texture-list">
                    <For
                        each=move || textures.get()
                        key=|texture| texture.clone()
                        children=move |texture| {
                            let texture_id = texture.clone();
                            view! {
                                <li>
                                    <span>{texture}</span>
                                    <button
                                        type="button"
                                        aria-label=format!("Remove texture {texture_id}")
                                        on:click=move |_| remove_texture_id(textures, lines, &texture_id)
                                    >
                                        "Remove"
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </section>
        </aside>
    }
}

fn add_texture_id(textures: RwSignal<Vec<String>>, texture_draft: RwSignal<String>) {
    let texture_id = texture_draft.get_untracked().trim().to_owned();
    if texture_id.is_empty() {
        return;
    }

    textures.update(|textures| {
        if !textures.iter().any(|existing| existing == &texture_id) {
            textures.push(texture_id);
        }
    });
    texture_draft.set(String::new());
}

fn remove_texture_id(
    textures: RwSignal<Vec<String>>,
    lines: RwSignal<Vec<Line>>,
    texture_id: &str,
) {
    textures.update(|textures| textures.retain(|existing| existing != texture_id));
    lines.with_untracked(|lines| {
        for line in lines {
            if line.texture_id_untracked().as_deref() == Some(texture_id) {
                line.set_texture_id(None);
            }
        }
    });
}
