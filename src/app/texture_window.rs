use crate::line::{Line, PaintingTexture};
use leptos::{ev, prelude::*};

#[component]
pub fn TextureWindow(
    textures: RwSignal<Vec<String>>,
    painting_textures: RwSignal<Vec<PaintingTexture>>,
    lines: RwSignal<Vec<Line>>,
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
) -> impl IntoView {
    let texture_draft = RwSignal::new(String::new());
    let painting_texture_draft = RwSignal::new(String::new());

    view! {
        <details class="texture-library-panel">
            <summary>"Manage wall, floor & ceiling textures"</summary>
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
        <details class="texture-library-panel">
            <summary>"Manage painting textures"</summary>
            <section class="texture-library">
                <p class="texture-library-note">
                    "Painting textures do not need power-of-two dimensions. Set their source pixel width and height."
                </p>
                <form on:submit=move |event: ev::SubmitEvent| {
                    event.prevent_default();
                    add_painting_texture(painting_textures, painting_texture_draft);
                }>
                    <input
                        class="texture-id-input"
                        type="text"
                        aria-label="Painting texture identifier"
                        placeholder="Painting texture identifier"
                        prop:value=move || painting_texture_draft.get()
                        on:input=move |event| {
                            painting_texture_draft.set(event_target_value(&event))
                        }
                    />
                    <button class="texture-add-button" type="submit">"Add"</button>
                </form>
                <ul class="texture-list">
                    <For
                        each=move || painting_textures.get()
                        key=|texture| texture.id.clone()
                        children=move |texture| {
                            let texture_id = texture.id.clone();
                            let width_texture_id = texture_id.clone();
                            let height_texture_id = texture_id.clone();
                            let remove_texture_id = texture_id.clone();
                            let width = texture.width;
                            let height = texture.height;
                            view! {
                                <li class="painting-texture-row">
                                    <span class="painting-texture-id" title=texture_id.clone()>
                                        {texture_id.clone()}
                                    </span>
                                    <label class="painting-texture-dimension-field">
                                        "W"
                                        <input
                                            class="painting-texture-dimension"
                                            type="number"
                                            min="1"
                                            step="1"
                                            aria-label=format!("Painting texture {texture_id} width in pixels")
                                            title="Width in pixels"
                                            prop:value=width.to_string()
                                            on:change=move |event| {
                                                set_painting_texture_dimension(
                                                    painting_textures,
                                                    &width_texture_id,
                                                    true,
                                                    event_target_value(&event),
                                                )
                                            }
                                        />
                                    </label>
                                    <label class="painting-texture-dimension-field">
                                        "H"
                                        <input
                                            class="painting-texture-dimension"
                                            type="number"
                                            min="1"
                                            step="1"
                                            aria-label=format!("Painting texture {texture_id} height in pixels")
                                            title="Height in pixels"
                                            prop:value=height.to_string()
                                            on:change=move |event| {
                                                set_painting_texture_dimension(
                                                    painting_textures,
                                                    &height_texture_id,
                                                    false,
                                                    event_target_value(&event),
                                                )
                                            }
                                        />
                                    </label>
                                    <button
                                        type="button"
                                        aria-label=format!("Remove painting texture {texture_id}")
                                        on:click=move |_| {
                                            remove_painting_texture(
                                                painting_textures,
                                                lines,
                                                &remove_texture_id,
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

fn add_painting_texture(textures: RwSignal<Vec<PaintingTexture>>, texture_draft: RwSignal<String>) {
    let texture_id = texture_draft.get_untracked().trim().to_owned();
    if texture_id.is_empty() {
        return;
    }

    textures.update(|textures| {
        if !textures.iter().any(|existing| existing.id == texture_id) {
            textures.push(PaintingTexture {
                id: texture_id.clone(),
                width: 1.0,
                height: 1.0,
            });
        }
    });
    texture_draft.set(String::new());
}

fn set_painting_texture_dimension(
    textures: RwSignal<Vec<PaintingTexture>>,
    texture_id: &str,
    is_width: bool,
    value: String,
) {
    let Ok(dimension) = value.parse::<f64>() else {
        return;
    };
    if !dimension.is_finite() || dimension <= 0.0 || dimension.fract() != 0.0 {
        return;
    }

    textures.update(|textures| {
        if let Some(texture) = textures.iter_mut().find(|texture| texture.id == texture_id) {
            if is_width {
                texture.width = dimension;
            } else {
                texture.height = dimension;
            }
        }
    });
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

fn remove_painting_texture(
    textures: RwSignal<Vec<PaintingTexture>>,
    lines: RwSignal<Vec<Line>>,
    texture_id: &str,
) {
    textures.update(|textures| textures.retain(|existing| existing.id != texture_id));
    lines.with_untracked(|lines| {
        for line in lines {
            let mut paintings = line.paintings_untracked();
            for painting in &mut paintings {
                if painting.texture_id() == texture_id {
                    painting.set_texture_id(String::new());
                }
            }
            line.set_paintings(paintings);
        }
    });
}
