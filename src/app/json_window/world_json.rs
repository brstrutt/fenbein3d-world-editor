use crate::line::Line;
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

pub(super) struct ImportedWorld {
    pub lines: Vec<Line>,
    pub source_json: Value,
}

pub(super) fn parse_world_json(
    json_text: &str,
    first_line_id: usize,
) -> Result<ImportedWorld, serde_json::Error> {
    let source_json: Value = serde_json::from_str(json_text)?;
    let world: JsonWorld = serde_json::from_value(source_json.clone())?;
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

    Ok(ImportedWorld { lines, source_json })
}

pub(super) fn serialize_world_with_lines(
    source_json: &Value,
    lines: &[Line],
) -> Result<String, String> {
    let mut world = source_json.clone();
    replace_wall_positions(&mut world, lines)?;
    serde_json::to_string_pretty(&world).map_err(|error| error.to_string())
}

fn replace_wall_positions(world: &mut Value, lines: &[Line]) -> Result<(), String> {
    let walls = world
        .get_mut("walls")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "The world JSON does not contain a walls array.".to_owned())?;

    let updated_walls = lines
        .iter()
        .enumerate()
        .map(|(index, line)| update_wall(index, line, walls))
        .collect::<Result<Vec<_>, _>>()?;
    *walls = updated_walls;

    Ok(())
}

fn update_wall(index: usize, line: &Line, existing_walls: &[Value]) -> Result<Value, String> {
    let mut wall = existing_walls
        .get(index)
        .cloned()
        .or_else(|| existing_walls.first().map(wall_template_without_paintings))
        .unwrap_or_else(|| json!({ "texture_id": "" }));
    let wall_object = wall
        .as_object_mut()
        .ok_or_else(|| format!("Wall {index} is not a JSON object."))?;
    let ((start_x, start_y), (end_x, end_y)) = line.endpoints();

    wall_object.insert("start".to_owned(), json!({ "x": start_x, "y": start_y }));
    wall_object.insert("end".to_owned(), json!({ "x": end_x, "y": end_y }));

    Ok(wall)
}

fn wall_template_without_paintings(wall: &Value) -> Value {
    let mut template = wall.clone();
    if let Some(object) = template.as_object_mut() {
        object.remove("paintings");
    }
    template
}

#[cfg(test)]
mod tests {
    use super::{parse_world_json, serialize_world_with_lines};
    use serde_json::Value;

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

        let updated = serialize_world_with_lines(&imported.source_json, &imported.lines).unwrap();
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
    fn new_walls_inherit_texture_without_copying_paintings() {
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

        let updated = serialize_world_with_lines(&imported.source_json, &imported.lines).unwrap();
        let updated: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(updated["walls"][1]["texture_id"], "wall");
        assert!(updated["walls"][1].get("paintings").is_none());
    }
}
