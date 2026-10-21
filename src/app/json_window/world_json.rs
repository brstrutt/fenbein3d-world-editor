use crate::line::Line;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Deserialize)]
struct JsonWorld {
    walls: Vec<JsonWall>,
}

#[derive(Deserialize)]
struct JsonWall {
    start: JsonPoint,
    end: JsonPoint,
    texture_id: Option<String>,
}

#[derive(Deserialize)]
struct JsonPoint {
    x: f64,
    y: f64,
}

pub(super) struct ImportedWorld {
    pub lines: Vec<Line>,
    pub textures: Vec<String>,
    pub wall_data_by_line_id: HashMap<usize, Value>,
    pub source_json: Value,
}

pub(super) fn parse_world_json(
    json_text: &str,
    first_line_id: usize,
) -> Result<ImportedWorld, serde_json::Error> {
    let source_json: Value = serde_json::from_str(json_text)?;
    let world: JsonWorld = serde_json::from_value(source_json.clone())?;
    let textures = collect_texture_ids(&source_json);
    let wall_data_by_line_id = source_json
        .get("walls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, wall)| (first_line_id + index, wall.clone()))
        .collect();
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
            .with_texture_id(wall.texture_id)
        })
        .collect();

    Ok(ImportedWorld {
        lines,
        textures,
        wall_data_by_line_id,
        source_json,
    })
}

pub(super) fn serialize_world_with_lines(
    source_json: &Value,
    wall_data_by_line_id: &HashMap<usize, Value>,
    lines: &[Line],
) -> Result<String, String> {
    let mut world = source_json.clone();
    replace_walls(&mut world, wall_data_by_line_id, lines)?;
    serde_json::to_string_pretty(&world).map_err(|error| error.to_string())
}

fn replace_walls(
    world: &mut Value,
    wall_data_by_line_id: &HashMap<usize, Value>,
    lines: &[Line],
) -> Result<(), String> {
    world
        .get_mut("walls")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "The world JSON does not contain a walls array.".to_owned())?;

    let updated_walls = lines
        .iter()
        .map(|line| update_wall(line, wall_data_by_line_id.get(&line.id)))
        .collect::<Result<Vec<_>, _>>()?;
    world["walls"] = Value::Array(updated_walls);

    Ok(())
}

fn update_wall(line: &Line, wall_data: Option<&Value>) -> Result<Value, String> {
    let mut wall = wall_data.cloned().unwrap_or_else(|| json!({}));
    let wall_object = wall
        .as_object_mut()
        .ok_or_else(|| format!("Wall data for line {} is not a JSON object.", line.id))?;
    let ((start_x, start_y), (end_x, end_y)) = line.endpoints_untracked();

    wall_object.insert("start".to_owned(), json!({ "x": start_x, "y": start_y }));
    wall_object.insert("end".to_owned(), json!({ "x": end_x, "y": end_y }));
    wall_object.insert(
        "texture_id".to_owned(),
        json!(line.texture_id_untracked().unwrap_or_default()),
    );

    Ok(wall)
}

fn collect_texture_ids(world: &Value) -> Vec<String> {
    let mut texture_ids = Vec::new();

    for field in ["floor_texture_id", "ceiling_texture_id"] {
        if let Some(texture_id) = world.get(field).and_then(Value::as_str) {
            add_texture_id(&mut texture_ids, texture_id);
        }
    }

    if let Some(walls) = world.get("walls").and_then(Value::as_array) {
        for wall in walls {
            if let Some(texture_id) = wall.get("texture_id").and_then(Value::as_str) {
                add_texture_id(&mut texture_ids, texture_id);
            }
        }
    }

    texture_ids
}

fn add_texture_id(texture_ids: &mut Vec<String>, texture_id: &str) {
    if !texture_id.is_empty() && !texture_ids.iter().any(|existing| existing == texture_id) {
        texture_ids.push(texture_id.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_world_json, serialize_world_with_lines};
    use crate::line::Line;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn imports_wall_endpoints_and_ignores_other_world_data() {
        let json_text = r#"{
            "floor_texture_id": "floor",
            "ceiling_texture_id": "ceiling",
            "walls": [{
                "start": { "x": -1.25, "y": 2.5 },
                "end": { "x": 3.75, "y": -4.0 },
                "texture_id": "wall",
                "paintings": [{ "id": "art", "top_left": { "x": 0.2, "y": 0.3 } }]
            }]
        }"#;

        let imported = parse_world_json(json_text, 0).unwrap();

        assert_eq!(imported.lines.len(), 1);
        assert_eq!(imported.lines[0].id, 0);
        assert_eq!(
            imported.lines[0].endpoints_untracked(),
            ((-1.25, 2.5), (3.75, -4.0))
        );
        assert_eq!(
            imported.lines[0].texture_id_untracked().as_deref(),
            Some("wall")
        );
        assert_eq!(imported.textures, ["floor", "ceiling", "wall"]);
    }

    #[test]
    fn rejects_invalid_world_json() {
        assert!(parse_world_json(r#"{"walls":[{}]}"#, 0).is_err());
    }

    #[test]
    fn updates_wall_positions_while_preserving_other_world_data() {
        let json_text = r#"{
            "floor_texture_id": "floor",
            "ceiling_texture_id": "ceiling",
            "walls": [{
                "start": { "x": 0.0, "y": 1.0 },
                "end": { "x": 2.0, "y": 3.0 },
                "texture_id": "wall",
                "paintings": [{ "id": "art" }]
            }]
        }"#;
        let imported = parse_world_json(json_text, 0).unwrap();
        imported.lines[0].set_endpoints((-4.5, 2.25), (6.0, -7.75));

        let updated = serialize_world_with_lines(
            &imported.source_json,
            &imported.wall_data_by_line_id,
            &imported.lines,
        )
        .unwrap();
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

    #[test]
    fn updates_wall_texture_and_keeps_new_lines_unselected() {
        let json_text = r#"{
            "walls": [{
                "start": { "x": 0.0, "y": 1.0 },
                "end": { "x": 2.0, "y": 3.0 },
                "texture_id": "stone"
            }]
        }"#;
        let imported = parse_world_json(json_text, 0).unwrap();
        let new_line = Line::from_endpoints(1, (4.0, 5.0), (6.0, 7.0));
        assert_eq!(new_line.texture_id_untracked(), None);

        imported.lines[0].set_texture_id(Some("brick".to_owned()));
        let lines = [imported.lines[0], new_line];

        let updated = serialize_world_with_lines(
            &imported.source_json,
            &imported.wall_data_by_line_id,
            &lines,
        )
        .unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(updated["walls"][0]["texture_id"], "brick");
        assert_eq!(updated["walls"][1]["texture_id"], "");
    }

    #[test]
    fn new_walls_are_unselected_and_start_without_wall_metadata() {
        let json_text = r#"{
            "walls": [{
                "start": { "x": 0.0, "y": 1.0 },
                "end": { "x": 2.0, "y": 3.0 },
                "texture_id": "wall",
                "paintings": [{ "id": "art" }]
            }]
        }"#;
        let mut imported = parse_world_json(json_text, 0).unwrap();
        imported
            .lines
            .push(crate::line::Line::from_endpoints(1, (4.0, 5.0), (6.0, 7.0)));

        let updated = serialize_world_with_lines(
            &imported.source_json,
            &imported.wall_data_by_line_id,
            &imported.lines,
        )
        .unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(updated["walls"][1]["texture_id"], "");
        assert!(updated["walls"][1].get("paintings").is_none());
    }

    #[test]
    fn deleting_a_line_removes_its_wall_metadata_without_reassigning_it() {
        let json_text = r#"{
            "walls": [
                {
                    "start": { "x": 0.0, "y": 0.0 },
                    "end": { "x": 1.0, "y": 0.0 },
                    "texture_id": "first",
                    "paintings": [{ "id": "first-art" }]
                },
                {
                    "start": { "x": 0.0, "y": 1.0 },
                    "end": { "x": 1.0, "y": 1.0 },
                    "texture_id": "deleted",
                    "paintings": [
                        { "id": "deleted-art-1" },
                        { "id": "deleted-art-2" },
                        { "id": "deleted-art-3" },
                        { "id": "deleted-art-4" },
                        { "id": "deleted-art-5" },
                        { "id": "deleted-art-6" },
                        { "id": "deleted-art-7" }
                    ]
                },
                {
                    "start": { "x": 0.0, "y": 2.0 },
                    "end": { "x": 1.0, "y": 2.0 },
                    "texture_id": "last",
                    "paintings": [{ "id": "last-art" }]
                }
            ]
        }"#;
        let imported = parse_world_json(json_text, 10).unwrap();
        let surviving_lines = [imported.lines[0], imported.lines[2]];

        let updated = serialize_world_with_lines(
            &imported.source_json,
            &imported.wall_data_by_line_id,
            &surviving_lines,
        )
        .unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();
        let walls = updated["walls"].as_array().unwrap();

        assert_eq!(walls.len(), 2);
        assert_eq!(walls[0]["texture_id"], "first");
        assert_eq!(walls[0]["paintings"][0]["id"], "first-art");
        assert_eq!(walls[1]["texture_id"], "last");
        assert_eq!(walls[1]["paintings"][0]["id"], "last-art");
        assert!(!updated.to_string().contains("deleted-art-"));
    }

    #[test]
    fn new_line_without_wall_metadata_serializes_without_inheriting_other_wall_data() {
        let source_json: Value = serde_json::from_str(
            r#"{"walls":[{"start":{"x":0.0,"y":0.0},"end":{"x":1.0,"y":0.0},"texture_id":"first","paintings":[{"id":"art"}]}]}"#,
        )
        .unwrap();
        let new_line = Line::from_endpoints(100, (3.0, 4.0), (5.0, 6.0));
        let updated =
            serialize_world_with_lines(&source_json, &HashMap::new(), &[new_line]).unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(updated["walls"][0]["texture_id"], "");
        assert!(updated["walls"][0].get("paintings").is_none());
    }
}
