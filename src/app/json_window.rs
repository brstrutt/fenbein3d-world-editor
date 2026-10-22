mod world_json;

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
    next_line_id: RwSignal<usize>,
    selected_line: RwSignal<Option<usize>>,
) -> impl IntoView {
    let minimized = RwSignal::new(false);
    let import_status = RwSignal::new(INITIAL_IMPORT_STATUS.to_owned());
    let json_text = RwSignal::new(String::new());
    let world_json = RwSignal::new(None::<Value>);
    let wall_data_by_line_id = RwSignal::new(None::<HashMap<usize, Value>>);

    Effect::new(move |_| {
        synchronize_json_text(
            lines,
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
            wall_data_by_line_id,
            next_line_id,
            selected_line,
            json_text,
            world_json,
            import_status,
        );
    };

    view! {
        <aside class="json-window" aria-labelledby="json-window-title">
            <header class="json-window-heading">
                <div>
                    <p>"Line data"</p>
                    <h2 id="json-window-title">"JSON"</h2>
                </div>
                <button
                    class="json-window-toggle"
                    type="button"
                    aria-label=move || {
                        if minimized.get() {
                            "Expand JSON window"
                        } else {
                            "Minimize JSON window"
                        }
                    }
                    title=move || {
                        if minimized.get() {
                            "Expand JSON window"
                        } else {
                            "Minimize JSON window"
                        }
                    }
                    aria-expanded=move || (!minimized.get()).to_string()
                    on:click=move |_| minimized.update(|minimized| *minimized = !*minimized)
                >
                    {move || if minimized.get() { "+" } else { "-" }}
                </button>
            </header>
            <textarea
                class="json-textarea"
                aria-label="Line positions JSON"
                placeholder="Paste a JSON object here..."
                spellcheck="false"
                hidden=move || minimized.get()
                prop:value=move || json_text.get()
                on:input=import_json
            />
            <p class="json-import-status" role="status">{move || import_status.get()}</p>
        </aside>
    }
}

fn synchronize_json_text(
    lines: RwSignal<Vec<Line>>,
    world_json: RwSignal<Option<Value>>,
    wall_data_by_line_id: RwSignal<Option<HashMap<usize, Value>>>,
    json_text: RwSignal<String>,
    import_status: RwSignal<String>,
) {
    let current_lines = lines.get();
    for line in &current_lines {
        let _ = line.endpoints();
        let _ = line.texture_id();
    }
    let source_json = match world_json.get_untracked() {
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

    match serialize_world_with_lines(&source_json, &wall_data_by_line_id, &current_lines) {
        Ok(updated_json) => json_text.set(updated_json),
        Err(error) => import_status.set(format!("Could not update JSON: {error}")),
    }
}

fn import_world_from_text(
    json_text_value: String,
    lines: RwSignal<Vec<Line>>,
    textures: RwSignal<Vec<String>>,
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
