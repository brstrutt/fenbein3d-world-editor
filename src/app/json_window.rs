use crate::line::Line;
use leptos::prelude::*;
use serde::Deserialize;

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

fn parse_world_lines(json: &str, first_line_id: usize) -> Result<Vec<Line>, serde_json::Error> {
    let world: JsonWorld = serde_json::from_str(json)?;
    Ok(world
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
        .collect())
}

#[cfg(test)]
mod tests {
    use super::parse_world_lines;

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

        let lines = parse_world_lines(json, 0).unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].id, 0);
        assert_eq!(lines[0].endpoints_untracked(), ((-1.25, 2.5), (3.75, -4.0)));
    }

    #[test]
    fn rejects_invalid_world_json() {
        assert!(parse_world_lines(r#"{"walls":[{}]}"#, 0).is_err());
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

    let import_json = move |event| {
        let json = event_target_value(&event);
        let first_line_id = next_line_id.get_untracked();
        match parse_world_lines(&json, first_line_id) {
            Ok(imported_lines) => {
                let line_count = imported_lines.len();
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
                on:input=import_json
            />
            <p class="json-import-status" role="status">{move || import_status.get()}</p>
        </aside>
    }
}
