use crate::line::Line;
use leptos::{ev, prelude::*};

#[component]
pub fn TextureWindow(
    textures: RwSignal<Vec<String>>,
    lines: RwSignal<Vec<Line>>,
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
) -> impl IntoView {
    let texture_draft = RwSignal::new(String::new());

    view! {
        <details class="texture-library-panel">
            <summary>"Manage textures"</summary>
            <section class="texture-library">
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
                                        on:click=move |_| {
                                            remove_texture_id(
                                                textures,
                                                lines,
                                                floor_texture_id,
                                                ceiling_texture_id,
                                                &texture_id,
                                            )
                                        }
                                    >
                                        "Remove"
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </section>
        </details>
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
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
    texture_id: &str,
) {
    textures.update(|textures| textures.retain(|existing| existing != texture_id));
    if floor_texture_id.get_untracked() == texture_id {
        floor_texture_id.set(String::new());
    }
    if ceiling_texture_id.get_untracked() == texture_id {
        ceiling_texture_id.set(String::new());
    }
    lines.with_untracked(|lines| {
        for line in lines {
            if line.texture_id_untracked().as_deref() == Some(texture_id) {
                line.set_texture_id(None);
            }
        }
    });
}
