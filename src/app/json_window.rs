use crate::line::Line;
use leptos::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct JsonWorld {
    walls: Vec<JsonWall>,
}

#[derive(Deserialize)]
struct JsonWall {
    start: JsonPoint,
    end: JsonPoint,
}

#[derive(Deserialize)]
struct JsonPoint {
    x: f64,
    y: f64,
}

fn parse_world_lines(
    json: &str,
    first_line_id: usize,
) -> Result<(Vec<Line>, Value), serde_json::Error> {
    let world_json: Value = serde_json::from_str(json)?;
    let world: JsonWorld = serde_json::from_value(world_json.clone())?;
    let lines = world
        .walls
        .into_iter()
        .enumerate()
        .map(|(index, wall)| {
            Line::from_endpoints(
                first_line_id + index,
                (wall.start.x, wall.start.y),
                (wall.end.x, wall.end.y),
            )
        })
        .collect();
    Ok((lines, world_json))
}

fn update_world_json(world: &mut Value, lines: &[Line]) -> Result<String, String> {
    let walls = world
        .get_mut("walls")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "The world JSON does not contain a walls array.".to_owned())?;

    let mut new_walls = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        let mut wall = if let Some(wall) = walls.get(index) {
            wall.clone()
        } else if let Some(wall) = walls.first() {
            let mut wall = wall.clone();
            if let Some(object) = wall.as_object_mut() {
                object.remove("paintings");
            }
            wall
        } else {
            json!({ "texture_id": "" })
        };

        let ((start_x, start_y), (end_x, end_y)) = line.endpoints();
        let wall_object = wall
            .as_object_mut()
            .ok_or_else(|| format!("Wall {index} is not a JSON object."))?;
        wall_object.insert("start".to_owned(), json!({ "x": start_x, "y": start_y }));
        wall_object.insert("end".to_owned(), json!({ "x": end_x, "y": end_y }));
        new_walls.push(wall);
    }

    *walls = new_walls;
    serde_json::to_string_pretty(world).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_world_lines, update_world_json};
    use serde_json::Value;

    #[test]
    fn imports_wall_endpoints_and_ignores_other_world_data() {
        let json = r#"{
            "floor_texture_id": "floor",
            "ceiling_texture_id": "ceiling",
            "walls": [{
                "start": { "x": -1.25, "y": 2.5 },
                "end": { "x": 3.75, "y": -4.0 },
                "texture_id": "wall",
                "paintings": [{ "id": "art", "top_left": { "x": 0.2, "y": 0.3 } }]
            }]
        }"#;

        let (lines, _) = parse_world_lines(json, 0).unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].id, 0);
        assert_eq!(lines[0].endpoints_untracked(), ((-1.25, 2.5), (3.75, -4.0)));
    }

    #[test]
    fn rejects_invalid_world_json() {
        assert!(parse_world_lines(r#"{"walls":[{}]}"#, 0).is_err());
    }

    #[test]
    fn updates_wall_positions_while_preserving_other_world_data() {
        let json = r#"{
            "floor_texture_id": "floor",
            "ceiling_texture_id": "ceiling",
            "walls": [{
                "start": { "x": 0.0, "y": 1.0 },
                "end": { "x": 2.0, "y": 3.0 },
                "texture_id": "wall",
                "paintings": [{ "id": "art" }]
            }]
        }"#;
        let (lines, mut world) = parse_world_lines(json, 0).unwrap();
        lines[0].set_endpoints((-4.5, 2.25), (6.0, -7.75));

        let updated = update_world_json(&mut world, &lines).unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(updated["floor_texture_id"], "floor");
        assert_eq!(updated["ceiling_texture_id"], "ceiling");
        assert_eq!(updated["walls"][0]["texture_id"], "wall");
        assert_eq!(updated["walls"][0]["paintings"][0]["id"], "art");
        assert_eq!(updated["walls"][0]["start"]["x"], -4.5);
        assert_eq!(updated["walls"][0]["start"]["y"], 2.25);
        assert_eq!(updated["walls"][0]["end"]["x"], 6.0);
        assert_eq!(updated["walls"][0]["end"]["y"], -7.75);
    }
}

#[component]
pub fn JsonWindow(
    lines: RwSignal<Vec<Line>>,
    next_line_id: RwSignal<usize>,
    selected_line: RwSignal<Option<usize>>,
) -> impl IntoView {
    let minimized = RwSignal::new(false);
    let import_status = RwSignal::new("Paste a JsonWorld object to import its walls.".to_owned());
    let json_text = RwSignal::new(String::new());
    let world_json = RwSignal::new(None::<Value>);

    Effect::new(move |_| {
        let current_lines = lines.get();
        let Some(mut world) = world_json.get_untracked() else {
            return;
        };
        match update_world_json(&mut world, &current_lines) {
            Ok(updated_json) => {
                world_json.set(Some(world));
                json_text.set(updated_json);
            }
            Err(error) => import_status.set(format!("Could not update JSON: {error}")),
        }
    });

    let import_json = move |event| {
        let json = event_target_value(&event);
        json_text.set(json.clone());
        let first_line_id = next_line_id.get_untracked();
        match parse_world_lines(&json, first_line_id) {
            Ok((imported_lines, imported_world)) => {
                let line_count = imported_lines.len();
                world_json.set(Some(imported_world));
                lines.set(imported_lines);
                next_line_id.set(first_line_id + line_count);
                selected_line.set(None);
                import_status.set(format!("Imported {line_count} walls."));
            }
            Err(_) => import_status.set("Waiting for valid JsonWorld JSON.".to_owned()),
        }
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
