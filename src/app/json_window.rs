mod world_json;

use super::texture_window::TextureWindow;
use crate::line::Line;
use leptos::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use self::world_json::{empty_world_json, parse_world_json, serialize_world_with_lines};

const INITIAL_IMPORT_STATUS: &str = "Paste a JsonWorld object to import its walls.";
const INVALID_JSON_STATUS: &str = "Waiting for valid JsonWorld JSON.";

#[component]
pub fn JsonWindow(
    lines: RwSignal<Vec<Line>>,
    textures: RwSignal<Vec<String>>,
    painting_textures: RwSignal<Vec<String>>,
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
    next_line_id: RwSignal<usize>,
    selected_line: RwSignal<Option<usize>>,
) -> impl IntoView {
    let minimized = RwSignal::new(true);
    let import_status = RwSignal::new(INITIAL_IMPORT_STATUS.to_owned());
    let json_text = RwSignal::new(String::new());
    let world_json = RwSignal::new(None::<Value>);
    let wall_data_by_line_id = RwSignal::new(None::<HashMap<usize, Value>>);

    Effect::new(move |_| {
        synchronize_json_text(
            lines,
            floor_texture_id,
            ceiling_texture_id,
            world_json,
            wall_data_by_line_id,
            json_text,
            import_status,
        )
    });

    let import_json = move |event| {
        import_world_from_text(
            event_target_value(&event),
            lines,
            textures,
            painting_textures,
            floor_texture_id,
            ceiling_texture_id,
            wall_data_by_line_id,
            next_line_id,
            selected_line,
            json_text,
            world_json,
            import_status,
        );
    };

    view! {
        <aside class="world-panel" aria-labelledby="world-panel-title">
            <header class="world-panel-heading">
                <div>
                    <p>"Scene settings"</p>
                    <h2 id="world-panel-title">"World"</h2>
                </div>
                <button
                    class="world-panel-toggle"
                    type="button"
                    aria-label=move || {
                        if minimized.get() {
                            "Show world settings"
                        } else {
                            "Hide world settings"
                        }
                    }
                    title=move || {
                        if minimized.get() {
                            "Show world settings"
                        } else {
                            "Hide world settings"
                        }
                    }
                    aria-expanded=move || (!minimized.get()).to_string()
                    on:click=move |_| minimized.update(|minimized| *minimized = !*minimized)
                >
                    {move || if minimized.get() { "Edit" } else { "−" }}
                </button>
            </header>
            <div class="world-panel-content" hidden=move || minimized.get()>
                <section class="world-texture-controls" aria-labelledby="world-texture-title">
                    <h3 id="world-texture-title">"World textures"</h3>
                    <label>
                        "Ceiling"
                        <select
                            class="world-texture-select"
                            aria-label="Ceiling texture"
                            prop:value=move || ceiling_texture_id.get()
                            on:change=move |event| ceiling_texture_id.set(event_target_value(&event))
                        >
                            <option value="">"Unselected"</option>
                            <For
                                each=move || textures.get()
                                key=|texture| texture.clone()
                                children=move |texture| {
                                    let texture_value = texture.clone();
                                    view! {
                                        <option value=texture_value>{texture}</option>
                                    }
                                }
                            />
                        </select>
                    </label>
                    <label>
                        "Floor"
                        <select
                            class="world-texture-select"
                            aria-label="Floor texture"
                            prop:value=move || floor_texture_id.get()
                            on:change=move |event| floor_texture_id.set(event_target_value(&event))
                        >
                            <option value="">"Unselected"</option>
                            <For
                                each=move || textures.get()
                                key=|texture| texture.clone()
                                children=move |texture| {
                                    let texture_value = texture.clone();
                                    view! {
                                        <option value=texture_value>{texture}</option>
                                    }
                                }
                            />
                        </select>
                    </label>
                </section>
                <TextureWindow
                    textures=textures
                    painting_textures=painting_textures
                    lines=lines
                    floor_texture_id=floor_texture_id
                    ceiling_texture_id=ceiling_texture_id
                />
                <details class="world-json-editor">
                    <summary>"Import / Export JSON"</summary>
                    <textarea
                        class="json-textarea"
                        aria-label="World JSON"
                        placeholder="Paste a JSON object here..."
                        spellcheck="false"
                        prop:value=move || json_text.get()
                        on:input=import_json
                    />
                    <p class="json-import-status" role="status">
                        {move || import_status.get()}
                    </p>
                </details>
            </div>
        </aside>
    }
}

fn synchronize_json_text(
    lines: RwSignal<Vec<Line>>,
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
    world_json: RwSignal<Option<Value>>,
    wall_data_by_line_id: RwSignal<Option<HashMap<usize, Value>>>,
    json_text: RwSignal<String>,
    import_status: RwSignal<String>,
) {
    let current_lines = lines.get();
    for line in &current_lines {
        let _ = line.endpoints();
        let _ = line.texture_id();
        let _ = line.paintings();
    }
    let mut source_json = match world_json.get_untracked() {
        Some(source_json) => source_json,
        None if current_lines.is_empty() => return,
        None => {
            let source_json = empty_world_json();
            world_json.set(Some(source_json.clone()));
            wall_data_by_line_id.set(Some(HashMap::new()));
            source_json
        }
    };
    let Some(wall_data_by_line_id) = wall_data_by_line_id.get_untracked() else {
        return;
    };
    source_json["floor_texture_id"] = serde_json::json!(floor_texture_id.get());
    source_json["ceiling_texture_id"] = serde_json::json!(ceiling_texture_id.get());

    match serialize_world_with_lines(&source_json, &wall_data_by_line_id, &current_lines) {
        Ok(updated_json) => json_text.set(updated_json),
        Err(error) => import_status.set(format!("Could not update JSON: {error}")),
    }
}

fn import_world_from_text(
    json_text_value: String,
    lines: RwSignal<Vec<Line>>,
    textures: RwSignal<Vec<String>>,
    painting_textures: RwSignal<Vec<String>>,
    floor_texture_id: RwSignal<String>,
    ceiling_texture_id: RwSignal<String>,
    wall_data_by_line_id: RwSignal<Option<HashMap<usize, Value>>>,
    next_line_id: RwSignal<usize>,
    selected_line: RwSignal<Option<usize>>,
    json_text: RwSignal<String>,
    world_json: RwSignal<Option<Value>>,
    import_status: RwSignal<String>,
) {
    json_text.set(json_text_value.clone());
    let first_line_id = next_line_id.get_untracked();

    match parse_world_json(&json_text_value, first_line_id) {
        Ok(imported_world) => {
            let line_count = imported_world.lines.len();
            textures.set(imported_world.textures);
            painting_textures.set(imported_world.painting_textures);
            floor_texture_id.set(
                imported_world
                    .source_json
                    .get("floor_texture_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            );
            ceiling_texture_id.set(
                imported_world
                    .source_json
                    .get("ceiling_texture_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            );
            wall_data_by_line_id.set(Some(imported_world.wall_data_by_line_id));
            world_json.set(Some(imported_world.source_json));
            lines.set(imported_world.lines);
            next_line_id.set(first_line_id + line_count);
            selected_line.set(None);
            import_status.set(format!("Imported {line_count} walls."));
        }
        Err(_) => import_status.set(INVALID_JSON_STATUS.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::synchronize_json_text;
    use crate::app::json_window::world_json::empty_world_json;
    use crate::line::Line;
    use leptos::prelude::*;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn selected_world_textures_are_written_to_json() {
        let floor_texture_id = RwSignal::new("stone-floor".to_owned());
        let ceiling_texture_id = RwSignal::new("plaster-ceiling".to_owned());
        let json_text = RwSignal::new(String::new());

        synchronize_json_text(
            RwSignal::new(Vec::new()),
            floor_texture_id,
            ceiling_texture_id,
            RwSignal::new(Some(empty_world_json())),
            RwSignal::new(Some(HashMap::new())),
            json_text,
            RwSignal::new(String::new()),
        );

        let updated: Value = serde_json::from_str(&json_text.get_untracked()).unwrap();
        assert_eq!(updated["floor_texture_id"], "stone-floor");
        assert_eq!(updated["ceiling_texture_id"], "plaster-ceiling");
    }

    #[test]
    fn painting_coordinate_changes_are_written_to_json() {
        let line = Line::from_endpoints(0, (0.0, 0.0), (3.0, 0.0));
        line.add_painting("art".to_owned(), (0.1, 0.1), None);
        let mut paintings = line.paintings_untracked();
        paintings[0].set_top_left((0.75, 0.4));
        line.set_paintings(paintings);
        let json_text = RwSignal::new(String::new());

        synchronize_json_text(
            RwSignal::new(vec![line]),
            RwSignal::new(String::new()),
            RwSignal::new(String::new()),
            RwSignal::new(Some(empty_world_json())),
            RwSignal::new(Some(HashMap::new())),
            json_text,
            RwSignal::new(String::new()),
        );

        let updated: Value = serde_json::from_str(&json_text.get_untracked()).unwrap();
        assert_eq!(updated["walls"][0]["paintings"][0]["top_left"]["x"], 0.75);
        assert_eq!(updated["walls"][0]["paintings"][0]["top_left"]["y"], 0.4);
        assert!(
            updated["walls"][0]["paintings"][0]
                .get("bottom_right")
                .is_none()
        );
    }
}
