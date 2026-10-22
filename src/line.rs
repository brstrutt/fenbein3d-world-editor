use leptos::{ev, prelude::*};
use serde_json::{Value, json};

pub const WORLD_UNIT_PX: f64 = 32.0;

#[derive(Clone, Copy)]
pub struct Line {
    pub id: usize,
    start: RwSignal<(f64, f64)>,
    end: RwSignal<(f64, f64)>,
    texture_id: RwSignal<Option<String>>,
    paintings: RwSignal<Vec<Painting>>,
    next_painting_id: RwSignal<usize>,
}

#[derive(Clone)]
pub struct Painting {
    key: usize,
    texture_id: String,
    top_left: (f64, f64),
    bottom_right: Option<(f64, f64)>,
    source_data: Value,
}

impl Painting {
    pub fn new(
        key: usize,
        texture_id: String,
        top_left: (f64, f64),
        bottom_right: Option<(f64, f64)>,
    ) -> Self {
        Self {
            key,
            texture_id,
            top_left,
            bottom_right,
            source_data: json!({}),
        }
    }

    pub fn from_json(
        key: usize,
        texture_id: String,
        top_left: (f64, f64),
        bottom_right: Option<(f64, f64)>,
        source_data: Value,
    ) -> Self {
        Self {
            key,
            texture_id,
            top_left,
            bottom_right,
            source_data,
        }
    }

    pub fn texture_id(&self) -> &str {
        &self.texture_id
    }

    pub fn key(&self) -> usize {
        self.key
    }

    pub fn top_left(&self) -> (f64, f64) {
        self.top_left
    }

    pub fn bottom_right(&self) -> Option<(f64, f64)> {
        self.bottom_right
    }

    pub fn set_texture_id(&mut self, texture_id: String) {
        self.texture_id = texture_id;
    }

    pub fn set_top_left(&mut self, top_left: (f64, f64)) {
        self.top_left = top_left;
    }

    pub fn set_bottom_right(&mut self, bottom_right: Option<(f64, f64)>) {
        self.bottom_right = bottom_right;
    }

    pub fn to_json(&self) -> Value {
        let mut data = self.source_data.clone();
        if let Some(object) = data.as_object_mut() {
            object.insert("id".to_owned(), json!(self.texture_id));
            object.insert(
                "top_left".to_owned(),
                json!({ "x": self.top_left.0, "y": self.top_left.1 }),
            );
            if let Some((x, y)) = self.bottom_right {
                object.insert("bottom_right".to_owned(), json!({ "x": x, "y": y }));
            } else {
                object.remove("bottom_right");
            }
        }
        data
    }
}

impl Line {
    pub fn new(id: usize) -> Self {
        let y = 1.0 + id as f64;
        Self::from_endpoints(id, (-1.5, y), (1.5, y))
    }

    pub fn from_endpoints(id: usize, start: (f64, f64), end: (f64, f64)) -> Self {
        Self {
            id,
            start: RwSignal::new(start),
            end: RwSignal::new(end),
            texture_id: RwSignal::new(None),
            paintings: RwSignal::new(Vec::new()),
            next_painting_id: RwSignal::new(0),
        }
    }

    pub fn with_texture_id(self, texture_id: Option<String>) -> Self {
        self.texture_id.set(texture_id);
        self
    }

    pub fn texture_id(&self) -> Option<String> {
        self.texture_id.get()
    }

    pub fn texture_id_untracked(&self) -> Option<String> {
        self.texture_id.get_untracked()
    }

    pub fn set_texture_id(&self, texture_id: Option<String>) {
        self.texture_id.set(texture_id);
    }

    pub fn with_paintings(self, paintings: Vec<Painting>) -> Self {
        let next_painting_id = paintings
            .iter()
            .map(|painting| painting.key() + 1)
            .max()
            .unwrap_or_default();
        self.paintings.set(paintings);
        self.next_painting_id.set(next_painting_id);
        self
    }

    pub fn paintings(&self) -> Vec<Painting> {
        self.paintings.get()
    }

    pub fn paintings_untracked(&self) -> Vec<Painting> {
        self.paintings.get_untracked()
    }

    pub fn set_paintings(&self, paintings: Vec<Painting>) {
        self.paintings.set(paintings);
    }

    pub fn add_painting(
        &self,
        texture_id: String,
        top_left: (f64, f64),
        bottom_right: Option<(f64, f64)>,
    ) {
        let key = self.next_painting_id.get_untracked();
        self.next_painting_id.set(key + 1);
        self.paintings.update(|paintings| {
            paintings.push(Painting::new(key, texture_id, top_left, bottom_right));
        });
    }

    pub fn set_endpoint(&self, is_start: bool, position: (f64, f64)) {
        if is_start {
            self.start.set(position);
        } else {
            self.end.set(position);
        }
    }

    pub fn endpoints(&self) -> ((f64, f64), (f64, f64)) {
        (self.start.get(), self.end.get())
    }

    pub fn endpoints_untracked(&self) -> ((f64, f64), (f64, f64)) {
        (self.start.get_untracked(), self.end.get_untracked())
    }

    pub fn set_endpoints(&self, start: (f64, f64), end: (f64, f64)) {
        self.start.set(start);
        self.end.set(end);
    }
}

#[component]
pub fn WorldLine(
    line: Line,
    active_endpoint: RwSignal<Option<(usize, bool)>>,
    endpoint_drag_start: RwSignal<Option<(f64, f64, f64, f64)>>,
    active_line: RwSignal<Option<usize>>,
    line_drag_start: RwSignal<Option<(f64, f64, f64, f64, f64, f64)>>,
    selected_line: RwSignal<Option<usize>>,
    scene_offset: RwSignal<(f64, f64)>,
    zoom: RwSignal<f64>,
) -> impl IntoView {
    let id = line.id;
    let start = line.start;
    let end = line.end;
    let texture_id = line.texture_id;

    let begin_drag = move |event: ev::PointerEvent, is_start: bool| {
        event.prevent_default();
        event.stop_propagation();
        selected_line.set(Some(id));
        let (x, y) = if is_start {
            start.get_untracked()
        } else {
            end.get_untracked()
        };
        endpoint_drag_start.set(Some((
            f64::from(event.client_x()),
            f64::from(event.client_y()),
            x,
            y,
        )));
        active_endpoint.set(Some((id, is_start)));
    };

    let line_geometry = move || {
        let (start_x, start_y) = start.get();
        let (end_x, end_y) = end.get();
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx).to_degrees();
        (start_x, start_y, length, angle)
    };

    view! {
        <div class="line-layer">
            <div
                class="world-line"
                class:missing-texture=move || {
                    texture_id.get().is_none_or(|texture_id| texture_id.is_empty())
                }
                class:dragging=move || active_line.get() == Some(id)
                style:left=move || {
                    let (x, _, _, _) = line_geometry();
                    let (pan_x, _) = scene_offset.get();
                    format!("calc(50% + {}px)", pan_x + x * WORLD_UNIT_PX * zoom.get())
                }
                style:top=move || {
                    let (_, y, _, _) = line_geometry();
                    let (_, pan_y) = scene_offset.get();
                    format!("calc(50% + {}px - 1px)", pan_y + y * WORLD_UNIT_PX * zoom.get())
                }
                style:width=move || format!("{}px", line_geometry().2 * WORLD_UNIT_PX * zoom.get())
                style:transform=move || format!("rotate({}deg)", line_geometry().3)
                on:pointerdown=move |event: ev::PointerEvent| {
                    event.prevent_default();
                    event.stop_propagation();
                    selected_line.set(Some(id));
                    let ((start_x, start_y), (end_x, end_y)) = line.endpoints();
                    line_drag_start.set(Some((
                        f64::from(event.client_x()),
                        f64::from(event.client_y()),
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                    )));
                    active_line.set(Some(id));
                }
            />
            <button
                class="line-endpoint"
                class:dragging=move || active_endpoint.get() == Some((id, true))
                type="button"
                aria-label="Move line start"
                style:left=move || {
                    let (x, _, _, _) = line_geometry();
                    let (pan_x, _) = scene_offset.get();
                    format!("calc(50% + {}px - 7px)", pan_x + x * WORLD_UNIT_PX * zoom.get())
                }
                style:top=move || {
                    let (_, y, _, _) = line_geometry();
                    let (_, pan_y) = scene_offset.get();
                    format!("calc(50% + {}px - 7px)", pan_y + y * WORLD_UNIT_PX * zoom.get())
                }
                on:pointerdown=move |event| begin_drag(event, true)
            />
            <button
                class="line-endpoint"
                class:dragging=move || active_endpoint.get() == Some((id, false))
                type="button"
                aria-label="Move line end"
                style:left=move || {
                    let (_, _, _, _) = line_geometry();
                    let (x, _) = end.get();
                    let (pan_x, _) = scene_offset.get();
                    format!("calc(50% + {}px - 7px)", pan_x + x * WORLD_UNIT_PX * zoom.get())
                }
                style:top=move || {
                    let (_, y) = end.get();
                    let (_, pan_y) = scene_offset.get();
                    format!("calc(50% + {}px - 7px)", pan_y + y * WORLD_UNIT_PX * zoom.get())
                }
                on:pointerdown=move |event| begin_drag(event, false)
            />
        </div>
    }
}

#[component]
pub fn LineInspector(
    lines: RwSignal<Vec<Line>>,
    textures: RwSignal<Vec<String>>,
    painting_textures: RwSignal<Vec<String>>,
    selected_line: RwSignal<Option<usize>>,
) -> impl IntoView {
    let mobile_expanded = RwSignal::new(false);
    let active_painting_key = RwSignal::new(None::<usize>);

    Effect::new({
        let active_painting_key = active_painting_key;
        move |_| {
            let selected_id = selected_line.get();
            let current_lines = lines.get();
            let first_painting_key = selected_id.and_then(|id| {
                current_lines
                    .iter()
                    .find(|line| line.id == id)
                    .and_then(|line| line.paintings_untracked().first().map(Painting::key))
            });
            active_painting_key.set(first_painting_key);
        }
    });

    let set_coordinate = move |is_start: bool, is_x: bool, value: String| {
        let (Some(id), Ok(value)) = (selected_line.get_untracked(), value.parse::<f64>()) else {
            return;
        };
        if !value.is_finite() {
            return;
        }

        lines.with_untracked(|lines| {
            if let Some(line) = lines.iter().find(|line| line.id == id) {
                let ((mut start_x, mut start_y), (mut end_x, mut end_y)) =
                    line.endpoints_untracked();
                let point = if is_start {
                    if is_x {
                        start_x = value;
                    } else {
                        start_y = value;
                    }
                    (start_x, start_y)
                } else {
                    if is_x {
                        end_x = value;
                    } else {
                        end_y = value;
                    }
                    (end_x, end_y)
                };
                line.set_endpoint(is_start, point);
            }
        });
    };

    let selected = move || {
        selected_line
            .get()
            .and_then(|id| lines.with(|lines| lines.iter().find(|line| line.id == id).copied()))
    };
    let set_texture = move |value: String| {
        let Some(id) = selected_line.get_untracked() else {
            return;
        };
        lines.with_untracked(|lines| {
            if let Some(line) = lines.iter().find(|line| line.id == id) {
                line.set_texture_id((!value.is_empty()).then_some(value));
            }
        });
    };
    let add_painting = move |_| {
        let Some(id) = selected_line.get_untracked() else {
            return;
        };
        let texture_id = painting_textures
            .get_untracked()
            .first()
            .cloned()
            .unwrap_or_default();
        lines.with_untracked(|lines| {
            if let Some(line) = lines.iter().find(|line| line.id == id) {
                line.add_painting(texture_id, (0.1, 0.1), Some((0.6, 0.6)));
                if let Some(painting) = line.paintings_untracked().last() {
                    active_painting_key.set(Some(painting.key()));
                }
            }
        });
    };
    let set_painting_texture = move |key: usize, value: String| {
        update_selected_painting(lines, selected_line, key, |painting| {
            painting.set_texture_id(value);
        });
    };
    let set_painting_coordinate = move |key: usize, is_x: bool, value: String| {
        let Ok(value) = value.parse::<f64>() else {
            return;
        };
        if !value.is_finite() {
            return;
        }
        update_selected_painting(lines, selected_line, key, |painting| {
            let (mut x, mut y) = painting.top_left();
            if is_x {
                x = value;
            } else {
                y = value;
            }
            painting.set_top_left((x, y));
        });
    };
    let set_painting_bottom_right = move |key: usize, is_x: bool, value: String| {
        let Ok(value) = value.parse::<f64>() else {
            return;
        };
        if !value.is_finite() {
            return;
        }
        update_selected_painting(lines, selected_line, key, |painting| {
            let (top_x, top_y) = painting.top_left();
            let (mut x, mut y) = painting
                .bottom_right()
                .unwrap_or((top_x + 0.5, top_y + 0.5));
            if is_x {
                x = value;
            } else {
                y = value;
            }
            painting.set_bottom_right(Some((x, y)));
        });
    };
    let remove_painting = move |key: usize| {
        let Some(id) = selected_line.get_untracked() else {
            return;
        };
        lines.with_untracked(|lines| {
            if let Some(line) = lines.iter().find(|line| line.id == id) {
                let mut paintings = line.paintings_untracked();
                paintings.retain(|painting| painting.key() != key);
                if active_painting_key.get_untracked() == Some(key) {
                    active_painting_key.set(paintings.first().map(Painting::key));
                }
                line.set_paintings(paintings);
            }
        });
    };
    let delete_selected = move |_| delete_selected_line(lines, selected_line);

    view! {
        <aside
            class="line-inspector"
            class:mobile-collapsed=move || !mobile_expanded.get()
            hidden=move || selected_line.get().is_none()
        >
            <header class="inspector-heading">
                <div>
                    <p>"Selected line"</p>
                    <h2>{move || selected().map_or_else(|| "Line".to_owned(), |line| format!("Line {}", line.id + 1))}</h2>
                </div>
                <button
                    class="inspector-expand"
                    type="button"
                    aria-label=move || {
                        if mobile_expanded.get() {
                            "Collapse line details"
                        } else {
                            "Expand line details"
                        }
                    }
                    aria-expanded=move || mobile_expanded.get().to_string()
                    on:click=move |_| {
                        mobile_expanded.update(|expanded| *expanded = !*expanded)
                    }
                >
                    {move || if mobile_expanded.get() { "Done" } else { "Edit" }}
                </button>
                <button
                    class="inspector-delete"
                    type="button"
                    aria-label="Delete selected line"
                    on:click=delete_selected
                >
                    "Delete"
                </button>
                <button
                    class="inspector-close"
                    type="button"
                    aria-label="Close line details"
                    on:click=move |_| selected_line.set(None)
                >
                    "Close"
                </button>
            </header>
            <section class="endpoint-details">
                <h3>"Texture"</h3>
                <select
                    class="coordinate-input"
                    aria-label="Wall texture"
                    prop:value=move || selected().and_then(|line| line.texture_id()).unwrap_or_default()
                    on:change=move |event| set_texture(event_target_value(&event))
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
            </section>
            <section class="endpoint-details painting-details">
                <header class="painting-heading">
                    <h3>"Paintings"</h3>
                    <button
                        class="painting-add"
                        type="button"
                        on:click=add_painting
                    >
                        "+ Add painting"
                    </button>
                </header>
                <Show
                    when=move || selected().is_some_and(|line| !line.paintings().is_empty())
                    fallback=|| view! { <p class="empty-paintings">"No paintings yet."</p> }
                >
                    {move || {
                        selected().and_then(|line| {
                            let paintings = line.paintings();
                            let Some(painting) = paintings
                                .iter()
                                .find(|painting| Some(painting.key()) == active_painting_key.get())
                                .cloned()
                            else {
                                return None;
                            };
                            let painting_key = painting.key();
                            let painting_number = painting_key + 1;
                            let texture_id = painting.texture_id().to_owned();
                            let (top_left_x, top_left_y) = painting.top_left();
                            let (bottom_right_x, bottom_right_y) = painting
                                .bottom_right()
                                .unwrap_or((top_left_x + 0.5, top_left_y + 0.5));
                            Some(view! {
                                <div class="painting-editor">
                                    <div class="painting-picker">
                                        <select
                                            class="coordinate-input"
                                            aria-label="Selected painting"
                                            prop:value=move || {
                                                active_painting_key
                                                    .get()
                                                    .map(|key| key.to_string())
                                                    .unwrap_or_default()
                                            }
                                            on:change=move |event| {
                                                active_painting_key.set(
                                                    event_target_value(&event).parse().ok(),
                                                )
                                            }
                                        >
                                            <For
                                                each=move || {
                                                    selected()
                                                        .map(|line| line.paintings())
                                                        .unwrap_or_default()
                                                }
                                                key=|painting| painting.key()
                                                children=move |painting| {
                                                    let key = painting.key();
                                                    let label = format!(
                                                        "Painting {} — {}",
                                                        key + 1,
                                                        if painting.texture_id().is_empty() {
                                                            "Unselected"
                                                        } else {
                                                            painting.texture_id()
                                                        }
                                                    );
                                                    view! {
                                                        <option value=key.to_string()>{label}</option>
                                                    }
                                                }
                                            />
                                        </select>
                                        <button
                                            class="painting-remove"
                                            type="button"
                                            aria-label=format!("Remove painting {painting_number}")
                                            on:click=move |_| remove_painting(painting_key)
                                        >
                                            "Remove"
                                        </button>
                                    </div>
                                    <label class="painting-texture-field">
                                        "Texture"
                                        <select
                                            class="coordinate-input"
                                            aria-label=format!("Painting {painting_number} texture")
                                            prop:value=texture_id
                                            on:change=move |event| {
                                                set_painting_texture(
                                                    painting_key,
                                                    event_target_value(&event),
                                                )
                                            }
                                        >
                                            <option value="">"Unselected"</option>
                                            <For
                                                each=move || painting_textures.get()
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
                                    <div class="painting-position">
                                        <label>
                                            "Left"
                                            <input
                                                class="coordinate-input"
                                                type="number"
                                                step="any"
                                                aria-label=format!("Painting {painting_number} left position")
                                                prop:value=top_left_x.to_string()
                                                on:change=move |event| {
                                                    set_painting_coordinate(
                                                        painting_key,
                                                        true,
                                                        event_target_value(&event),
                                                    )
                                                }
                                            />
                                        </label>
                                        <label>
                                            "Top"
                                            <input
                                                class="coordinate-input"
                                                type="number"
                                                step="any"
                                                aria-label=format!("Painting {painting_number} top position")
                                                prop:value=top_left_y.to_string()
                                                on:change=move |event| {
                                                    set_painting_coordinate(
                                                        painting_key,
                                                        false,
                                                        event_target_value(&event),
                                                    )
                                                }
                                            />
                                        </label>
                                    </div>
                                    <div class="painting-position">
                                        <label>
                                            "Right"
                                            <input
                                                class="coordinate-input"
                                                type="number"
                                                step="any"
                                                aria-label=format!("Painting {painting_number} right position")
                                                prop:value=bottom_right_x.to_string()
                                                on:change=move |event| {
                                                    set_painting_bottom_right(
                                                        painting_key,
                                                        true,
                                                        event_target_value(&event),
                                                    )
                                                }
                                            />
                                        </label>
                                        <label>
                                            "Bottom"
                                            <input
                                                class="coordinate-input"
                                                type="number"
                                                step="any"
                                                aria-label=format!("Painting {painting_number} bottom position")
                                                prop:value=bottom_right_y.to_string()
                                                on:change=move |event| {
                                                    set_painting_bottom_right(
                                                        painting_key,
                                                        false,
                                                        event_target_value(&event),
                                                    )
                                                }
                                            />
                                        </label>
                                    </div>
                                </div>
                            })
                        })
                    }}
                </Show>
            </section>
            <section class="endpoint-details">
                <h3>"Start"</h3>
                <dl>
                    <div>
                        <dt>"X"</dt>
                        <dd>
                            <input
                                class="coordinate-input"
                                type="number"
                                step="any"
                                aria-label="Start X coordinate"
                                prop:value=move || selected().map_or_else(String::new, |line| line.endpoints().0.0.to_string())
                                on:change=move |event| set_coordinate(true, true, event_target_value(&event))
                            />
                        </dd>
                    </div>
                    <div>
                        <dt>"Y"</dt>
                        <dd>
                            <input
                                class="coordinate-input"
                                type="number"
                                step="any"
                                aria-label="Start Y coordinate"
                                prop:value=move || selected().map_or_else(String::new, |line| line.endpoints().0.1.to_string())
                                on:change=move |event| set_coordinate(true, false, event_target_value(&event))
                            />
                        </dd>
                    </div>
                </dl>
            </section>
            <section class="endpoint-details">
                <h3>"End"</h3>
                <dl>
                    <div>
                        <dt>"X"</dt>
                        <dd>
                            <input
                                class="coordinate-input"
                                type="number"
                                step="any"
                                aria-label="End X coordinate"
                                prop:value=move || selected().map_or_else(String::new, |line| line.endpoints().1.0.to_string())
                                on:change=move |event| set_coordinate(false, true, event_target_value(&event))
                            />
                        </dd>
                    </div>
                    <div>
                        <dt>"Y"</dt>
                        <dd>
                            <input
                                class="coordinate-input"
                                type="number"
                                step="any"
                                aria-label="End Y coordinate"
                                prop:value=move || selected().map_or_else(String::new, |line| line.endpoints().1.1.to_string())
                                on:change=move |event| set_coordinate(false, false, event_target_value(&event))
                            />
                        </dd>
                    </div>
                </dl>
            </section>
        </aside>
    }
}

fn update_selected_painting(
    lines: RwSignal<Vec<Line>>,
    selected_line: RwSignal<Option<usize>>,
    key: usize,
    update: impl FnOnce(&mut Painting),
) {
    let Some(id) = selected_line.get_untracked() else {
        return;
    };
    lines.with_untracked(|lines| {
        if let Some(line) = lines.iter().find(|line| line.id == id) {
            let mut paintings = line.paintings_untracked();
            if let Some(painting) = paintings.iter_mut().find(|painting| painting.key() == key) {
                update(painting);
                line.set_paintings(paintings);
            }
        }
    });
}

fn delete_selected_line(lines: RwSignal<Vec<Line>>, selected_line: RwSignal<Option<usize>>) {
    let Some(selected_id) = selected_line.get_untracked() else {
        return;
    };
    lines.update(|lines| lines.retain(|line| line.id != selected_id));
    selected_line.set(None);
}
