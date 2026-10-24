use leptos::{ev, prelude::*, wasm_bindgen::JsCast};
use serde_json::{Value, json};

pub const WORLD_UNIT_PX: f64 = 32.0;
const PREVIEW_PAN_DRAG_THRESHOLD_PX: f64 = 3.0;
const MIN_PAINTING_EDGE_LENGTH: f64 = 0.02;

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

#[derive(Clone)]
struct PaintingDragStart {
    line_id: usize,
    painting_key: usize,
    pointer_x: f64,
    pointer_y: f64,
    top_left: (f64, f64),
    bottom_right: Option<(f64, f64)>,
    transform: (f64, f64, f64, f64),
    action: PaintingDragAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaintingEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
enum PaintingDragAction {
    Move,
    Resize(PaintingEdge),
}

#[derive(Clone, Copy)]
struct PreviewViewBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Copy)]
struct PreviewPanStart {
    pointer_x: f64,
    pointer_y: f64,
    view_box: PreviewViewBox,
    transform: (f64, f64, f64, f64),
    moved: bool,
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
    let painting_drag_start = RwSignal::new(None::<PaintingDragStart>);
    let preview_view_box = RwSignal::new(None::<PreviewViewBox>);
    let preview_pan_start = RwSignal::new(None::<PreviewPanStart>);

    let painting_move_listener = window_event_listener(ev::pointermove, move |event| {
        if let Some(drag_start) = painting_drag_start.get_untracked() {
            let screen_delta = (
                f64::from(event.client_x()) - drag_start.pointer_x,
                f64::from(event.client_y()) - drag_start.pointer_y,
            );
            let Some(delta) = inverse_svg_transform_delta(drag_start.transform, screen_delta)
            else {
                return;
            };
            let (top_left, bottom_right) = match drag_start.action {
                PaintingDragAction::Move => {
                    translate_painting(drag_start.top_left, drag_start.bottom_right, delta)
                }
                PaintingDragAction::Resize(edge) => {
                    resize_painting(drag_start.top_left, drag_start.bottom_right, edge, delta)
                }
            };

            lines.with_untracked(|lines| {
                if let Some(line) = lines.iter().find(|line| line.id == drag_start.line_id) {
                    let mut paintings = line.paintings_untracked();
                    if let Some(painting) = paintings
                        .iter_mut()
                        .find(|painting| painting.key() == drag_start.painting_key)
                    {
                        painting.set_top_left(top_left);
                        painting.set_bottom_right(bottom_right);
                        line.set_paintings(paintings);
                    }
                }
            });
        } else if let Some(pan_start) = preview_pan_start.get_untracked() {
            let screen_delta = (
                f64::from(event.client_x()) - pan_start.pointer_x,
                f64::from(event.client_y()) - pan_start.pointer_y,
            );
            if !pan_start.moved && !preview_pan_threshold_reached(screen_delta) {
                return;
            }
            let Some(delta) = inverse_svg_transform_delta(pan_start.transform, screen_delta) else {
                return;
            };
            preview_pan_start.set(Some(PreviewPanStart {
                moved: true,
                ..pan_start
            }));
            preview_view_box.set(Some(PreviewViewBox {
                x: pan_start.view_box.x - delta.0,
                y: pan_start.view_box.y - delta.1,
                ..pan_start.view_box
            }));
        }
    });
    let stop_preview_drag = move |_| {
        if preview_pan_start
            .get_untracked()
            .is_some_and(|pan_start| !pan_start.moved)
        {
            active_painting_key.set(None);
        }
        painting_drag_start.set(None);
        preview_pan_start.set(None);
    };
    let preview_up_listener = window_event_listener(ev::pointerup, stop_preview_drag);
    let preview_cancel_listener = window_event_listener(ev::pointercancel, move |_| {
        painting_drag_start.set(None);
        preview_pan_start.set(None);
    });

    Effect::new(move |_| {
        let _ = selected_line.get();
        active_painting_key.set(None);
        preview_view_box.set(None);
        preview_pan_start.set(None);
    });

    let selected = move || {
        selected_line
            .get()
            .and_then(|id| lines.with(|lines| lines.iter().find(|line| line.id == id).copied()))
    };
    let selected_wall_width = move || {
        selected()
            .map(|line| wall_width(line.endpoints()))
            .unwrap_or(0.01)
            .max(0.01)
    };
    let get_preview_view_box = move || {
        preview_view_box
            .get()
            .unwrap_or_else(|| default_preview_view_box(selected_wall_width()))
    };
    let zoom_preview = move |factor: f64| {
        let current = get_preview_view_box();
        let anchor = (
            current.x + current.width / 2.0,
            current.y + current.height / 2.0,
        );
        let base_width = selected_wall_width() + 0.05;
        preview_view_box.set(Some(zoom_preview_view_box(
            current,
            anchor,
            factor,
            base_width / 100.0,
            base_width,
        )));
    };

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
                line.add_painting(texture_id, (0.1, 0.1), None);
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
    let set_painting_custom_bottom_right = move |key: usize, enabled: bool| {
        update_selected_painting(lines, selected_line, key, |painting| {
            if enabled {
                painting
                    .set_bottom_right(Some(implicit_painting_bottom_right(painting.top_left())));
            } else {
                painting.set_bottom_right(None);
            }
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
                    active_painting_key.set(None);
                }
                line.set_paintings(paintings);
            }
        });
    };
    let delete_selected = move |_| delete_selected_line(lines, selected_line);

    on_cleanup(move || {
        painting_move_listener.remove();
        preview_up_listener.remove();
        preview_cancel_listener.remove();
    });

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
            <section class="endpoint-details wall-painting-preview">
                <header class="wall-preview-heading">
                    <h3>"Wall preview"</h3>
                    <div class="wall-preview-controls" aria-label="Wall preview controls">
                        <button
                            type="button"
                            aria-label="Zoom out wall preview"
                            title="Zoom out"
                            on:click=move |_| zoom_preview(1.25)
                        >
                            "−"
                        </button>
                        <button
                            type="button"
                            aria-label="Zoom in wall preview"
                            title="Zoom in"
                            on:click=move |_| zoom_preview(0.8)
                        >
                            "+"
                        </button>
                        <button
                            type="button"
                            aria-label="Reset wall preview zoom and pan"
                            title="Reset view"
                            on:click=move |_| preview_view_box.set(None)
                        >
                            "Reset"
                        </button>
                    </div>
                </header>
                <p>
                    {move || {
                        format!(
                            "{:.2} × 2.0 wall units · Scroll to zoom, drag empty space to pan",
                            selected_wall_width()
                        )
                    }}
                </p>
                <svg
                    class="wall-preview-svg"
                    class:panning=move || {
                        preview_pan_start.get().is_some_and(|pan_start| pan_start.moved)
                    }
                    role="group"
                    aria-label="Wall preview. Scroll to zoom, drag empty space to pan, and drag a painting to move it."
                    viewBox=move || {
                        let view_box = get_preview_view_box();
                        format!(
                            "{} {} {} {}",
                            view_box.x, view_box.y, view_box.width, view_box.height
                        )
                    }
                    preserveAspectRatio="xMidYMid meet"
                    on:pointerdown=move |event: ev::PointerEvent| {
                        if event.button() != 0 {
                            return;
                        }
                        event.prevent_default();
                        event.stop_propagation();
                        let Some(target) = event.current_target().and_then(|target| {
                            target.dyn_into::<leptos::web_sys::SvgGraphicsElement>().ok()
                        }) else {
                            return;
                        };
                        let Some(matrix) = target.get_screen_ctm() else {
                            return;
                        };
                        preview_pan_start.set(Some(PreviewPanStart {
                            pointer_x: f64::from(event.client_x()),
                            pointer_y: f64::from(event.client_y()),
                            view_box: get_preview_view_box(),
                            transform: (
                                f64::from(matrix.a()),
                                f64::from(matrix.b()),
                                f64::from(matrix.c()),
                                f64::from(matrix.d()),
                            ),
                            moved: false,
                        }));
                    }
                    on:wheel=move |event: ev::WheelEvent| {
                        event.prevent_default();
                        event.stop_propagation();
                        let Some(target) = event.current_target().and_then(|target| {
                            target.dyn_into::<leptos::web_sys::SvgGraphicsElement>().ok()
                        }) else {
                            return;
                        };
                        let Some(matrix) = target.get_screen_ctm() else {
                            return;
                        };
                        let view_box = get_preview_view_box();
                        let Some(anchor) = inverse_svg_transform_point(
                            (
                                f64::from(matrix.a()),
                                f64::from(matrix.b()),
                                f64::from(matrix.c()),
                                f64::from(matrix.d()),
                                f64::from(matrix.e()),
                                f64::from(matrix.f()),
                            ),
                            (
                                f64::from(event.client_x()),
                                f64::from(event.client_y()),
                            ),
                        ) else {
                            return;
                        };
                        let base_width = selected_wall_width() + 0.05;
                        preview_view_box.set(Some(zoom_preview_view_box(
                            view_box,
                            anchor,
                            if event.delta_y() < 0.0 { 0.8 } else { 1.25 },
                            base_width / 100.0,
                            base_width,
                        )));
                    }
                >
                    <rect
                        class="wall-preview-surface"
                        x="0"
                        y="0"
                        width=move || selected_wall_width().to_string()
                        height="2"
                    />
                    <For
                        each=move || {
                            selected()
                                .map(|line| {
                                    line.paintings()
                                        .into_iter()
                                        .map(|painting| ((line.id, painting.key()), painting))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        }
                        key=|(line_id, painting)| (*line_id, painting.key())
                        children=move |((line_id, painting_key), painting)| {
                            let painting_label =
                                format!("Select painting {} ({})", painting_key + 1, painting.texture_id());
                            let painting_bounds = move || {
                                lines.with(|lines| {
                                    lines
                                        .iter()
                                        .find(|line| line.id == line_id)
                                        .and_then(|line| {
                                            line.paintings().into_iter().find(|painting| {
                                                painting.key() == painting_key
                                            })
                                        })
                                        .map(|painting| {
                                            let (left, top) = painting.top_left();
                                            let (right, bottom) = painting
                                                .bottom_right()
                                                .unwrap_or_else(|| {
                                                    implicit_painting_bottom_right((left, top))
                                                });
                                            (left, top, right, bottom)
                                        })
                                })
                            };
                            view! {
                                <rect
                                    class="wall-preview-painting"
                                    class:selected=move || {
                                        active_painting_key.get() == Some(painting_key)
                                    }
                                    class:dragging=move || {
                                        painting_drag_start
                                            .get()
                                            .is_some_and(|drag| drag.painting_key == painting_key)
                                    }
                                    class:resizing-horizontal=move || {
                                        painting_drag_start.get().is_some_and(|drag| {
                                            drag.painting_key == painting_key
                                                && matches!(
                                                    drag.action,
                                                    PaintingDragAction::Resize(
                                                        PaintingEdge::Left | PaintingEdge::Right
                                                    )
                                                )
                                        })
                                    }
                                    class:resizing-vertical=move || {
                                        painting_drag_start.get().is_some_and(|drag| {
                                            drag.painting_key == painting_key
                                                && matches!(
                                                    drag.action,
                                                    PaintingDragAction::Resize(
                                                        PaintingEdge::Top | PaintingEdge::Bottom
                                                    )
                                                )
                                        })
                                    }
                                    role="button"
                                    tabindex="0"
                                    aria-label=painting_label
                                    x=move || painting_bounds().map_or(0.0, |bounds| bounds.0)
                                    y=move || painting_bounds().map_or(0.0, |bounds| bounds.1)
                                    width=move || {
                                        painting_bounds()
                                            .map_or(0.0, |bounds| (bounds.2 - bounds.0).max(0.0))
                                    }
                                    height=move || {
                                        painting_bounds()
                                            .map_or(0.0, |bounds| (bounds.3 - bounds.1).max(0.0))
                                    }
                                    on:click=move |_| {
                                        active_painting_key.set(Some(painting_key))
                                    }
                                    on:pointerdown=move |event: ev::PointerEvent| {
                                        if event.button() != 0 {
                                            return;
                                        }
                                        event.prevent_default();
                                        event.stop_propagation();
                                        active_painting_key.set(Some(painting_key));
                                        let Some((top_left, bottom_right)) = lines.with_untracked(|lines| {
                                            lines
                                                .iter()
                                                .find(|line| line.id == line_id)
                                                .and_then(|line| {
                                                    line.paintings_untracked()
                                                        .into_iter()
                                                        .find(|painting| painting.key() == painting_key)
                                                        .map(|painting| {
                                                            (painting.top_left(), painting.bottom_right())
                                                        })
                                                })
                                        }) else {
                                            return;
                                        };
                                        let (right, bottom) = bottom_right.unwrap_or_else(|| {
                                            implicit_painting_bottom_right(top_left)
                                        });
                                        let Some(target) = event
                                            .current_target()
                                            .and_then(|target| {
                                                target
                                                    .dyn_into::<leptos::web_sys::SvgGraphicsElement>()
                                                    .ok()
                                            })
                                        else {
                                            return;
                                        };
                                        let Some(matrix) = target.get_screen_ctm() else {
                                            return;
                                        };
                                        let transform = (
                                            f64::from(matrix.a()),
                                            f64::from(matrix.b()),
                                            f64::from(matrix.c()),
                                            f64::from(matrix.d()),
                                        );
                                        let action = inverse_svg_transform_point(
                                            (
                                                transform.0,
                                                transform.1,
                                                transform.2,
                                                transform.3,
                                                f64::from(matrix.e()),
                                                f64::from(matrix.f()),
                                            ),
                                            (
                                                f64::from(event.client_x()),
                                                f64::from(event.client_y()),
                                            ),
                                        )
                                        .and_then(|point| {
                                            painting_edge_at_point(
                                                (top_left.0, top_left.1, right, bottom),
                                                transform,
                                                point,
                                                8.0,
                                            )
                                        })
                                        .map_or(PaintingDragAction::Move, PaintingDragAction::Resize);
                                        painting_drag_start.set(Some(PaintingDragStart {
                                            line_id,
                                            painting_key,
                                            pointer_x: f64::from(event.client_x()),
                                            pointer_y: f64::from(event.client_y()),
                                            top_left,
                                            bottom_right,
                                            transform,
                                            action,
                                        }));
                                    }
                                    on:keydown=move |event: ev::KeyboardEvent| {
                                        if event.key() == "Enter" || event.key() == " " {
                                            event.prevent_default();
                                            active_painting_key.set(Some(painting_key));
                                        }
                                    }
                                />
                            }
                        }
                    />
                </svg>
                <div class="wall-preview-axis-labels" aria-hidden="true">
                    <span>"0, 0 — top left"</span>
                    <span>"Wall width →"</span>
                </div>
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
                    when=move || {
                        selected().is_some_and(|line| {
                            active_painting_key.get().is_some_and(|key| {
                                line.paintings().iter().any(|painting| painting.key() == key)
                            })
                        })
                    }
                    fallback=|| {
                        view! {
                            <p class="empty-paintings">
                                "Click a painting in the preview to edit its details."
                            </p>
                        }
                    }
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
                                .unwrap_or_else(|| {
                                    implicit_painting_bottom_right((top_left_x, top_left_y))
                                });
                            Some(view! {
                                <div class="painting-editor">
                                    <div class="painting-picker">
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
                                    <label class="painting-custom-size">
                                        <input
                                            type="checkbox"
                                            aria-label=format!("Specify painting {painting_number} bottom-right coordinates")
                                            prop:checked=painting.bottom_right().is_some()
                                            on:change=move |event| {
                                                set_painting_custom_bottom_right(
                                                    painting_key,
                                                    event_target_checked(&event),
                                                )
                                            }
                                        />
                                        "Specify bottom-right coordinates"
                                    </label>
                                    <Show when=move || {
                                        selected()
                                            .and_then(|line| {
                                                line.paintings()
                                                    .into_iter()
                                                    .find(|item| item.key() == painting_key)
                                            })
                                            .is_some_and(|item| item.bottom_right().is_some())
                                    }>
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
                                    </Show>
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

fn wall_width(((start_x, start_y), (end_x, end_y)): ((f64, f64), (f64, f64))) -> f64 {
    (end_x - start_x).hypot(end_y - start_y)
}

fn default_preview_view_box(wall_width: f64) -> PreviewViewBox {
    PreviewViewBox {
        x: -0.025,
        y: -0.025,
        width: wall_width + 0.05,
        height: 2.05,
    }
}

fn zoom_preview_view_box(
    view_box: PreviewViewBox,
    (anchor_x, anchor_y): (f64, f64),
    factor: f64,
    min_width: f64,
    max_width: f64,
) -> PreviewViewBox {
    let width = (view_box.width * factor).clamp(min_width, max_width);
    let scale = width / view_box.width;
    let height = view_box.height * scale;
    let anchor_ratio_x = (anchor_x - view_box.x) / view_box.width;
    let anchor_ratio_y = (anchor_y - view_box.y) / view_box.height;
    PreviewViewBox {
        x: anchor_x - anchor_ratio_x * width,
        y: anchor_y - anchor_ratio_y * height,
        width,
        height,
    }
}

fn inverse_svg_transform_delta(
    (a, b, c, d): (f64, f64, f64, f64),
    (screen_x, screen_y): (f64, f64),
) -> Option<(f64, f64)> {
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() < f64::EPSILON {
        return None;
    }
    Some((
        (d * screen_x - c * screen_y) / determinant,
        (-b * screen_x + a * screen_y) / determinant,
    ))
}

fn inverse_svg_transform_point(
    (a, b, c, d, e, f): (f64, f64, f64, f64, f64, f64),
    (screen_x, screen_y): (f64, f64),
) -> Option<(f64, f64)> {
    inverse_svg_transform_delta((a, b, c, d), (screen_x - e, screen_y - f))
}

fn painting_edge_at_point(
    (left, top, right, bottom): (f64, f64, f64, f64),
    (a, b, c, d): (f64, f64, f64, f64),
    (x, y): (f64, f64),
    threshold_px: f64,
) -> Option<PaintingEdge> {
    let x_scale = a.hypot(b);
    let y_scale = c.hypot(d);
    let mut closest = None::<(PaintingEdge, f64)>;

    for (edge, distance) in [
        (PaintingEdge::Left, (x - left).abs() * x_scale),
        (PaintingEdge::Right, (right - x).abs() * x_scale),
        (PaintingEdge::Top, (y - top).abs() * y_scale),
        (PaintingEdge::Bottom, (bottom - y).abs() * y_scale),
    ] {
        if distance <= threshold_px
            && closest.is_none_or(|(_, closest_distance)| distance < closest_distance)
        {
            closest = Some((edge, distance));
        }
    }

    closest.map(|(edge, _)| edge)
}

fn resize_painting(
    (left, top): (f64, f64),
    bottom_right: Option<(f64, f64)>,
    edge: PaintingEdge,
    (delta_x, delta_y): (f64, f64),
) -> ((f64, f64), Option<(f64, f64)>) {
    let (right, bottom) =
        bottom_right.unwrap_or_else(|| implicit_painting_bottom_right((left, top)));
    if bottom_right.is_some() {
        return match edge {
            PaintingEdge::Left => (
                ((left + delta_x).min(right - MIN_PAINTING_EDGE_LENGTH), top),
                Some((right, bottom)),
            ),
            PaintingEdge::Right => (
                (left, top),
                Some((
                    (right + delta_x).max(left + MIN_PAINTING_EDGE_LENGTH),
                    bottom,
                )),
            ),
            PaintingEdge::Top => (
                (left, (top + delta_y).min(bottom - MIN_PAINTING_EDGE_LENGTH)),
                Some((right, bottom)),
            ),
            PaintingEdge::Bottom => (
                (left, top),
                Some((
                    right,
                    (bottom + delta_y).max(top + MIN_PAINTING_EDGE_LENGTH),
                )),
            ),
        };
    }

    match edge {
        PaintingEdge::Left => {
            let new_left = (left + delta_x).min(right - MIN_PAINTING_EDGE_LENGTH);
            let height = right - new_left;
            let new_top = 1.0 - height / 2.0;
            ((new_left, new_top), None)
        }
        PaintingEdge::Right => {
            let new_right = (right + delta_x).max(left + MIN_PAINTING_EDGE_LENGTH);
            let height = new_right - left;
            let new_top = 1.0 - height / 2.0;
            ((left, new_top), None)
        }
        PaintingEdge::Top => {
            let new_top = (top + delta_y).min(1.0 - MIN_PAINTING_EDGE_LENGTH / 2.0);
            ((left, new_top), None)
        }
        PaintingEdge::Bottom => {
            let new_bottom = (bottom + delta_y).max(1.0 + MIN_PAINTING_EDGE_LENGTH / 2.0);
            ((left, 2.0 - new_bottom), None)
        }
    }
}

fn preview_pan_threshold_reached((delta_x, delta_y): (f64, f64)) -> bool {
    delta_x.hypot(delta_y) >= PREVIEW_PAN_DRAG_THRESHOLD_PX
}

fn translate_painting(
    (left, top): (f64, f64),
    bottom_right: Option<(f64, f64)>,
    (delta_x, delta_y): (f64, f64),
) -> ((f64, f64), Option<(f64, f64)>) {
    match bottom_right {
        Some((right, bottom)) => (
            (left + delta_x, top + delta_y),
            Some((right + delta_x, bottom + delta_y)),
        ),
        None => ((left + delta_x, top), None),
    }
}

fn implicit_painting_bottom_right((left, top): (f64, f64)) -> (f64, f64) {
    let bottom = 2.0 - top;
    (left + bottom - top, bottom)
}

fn delete_selected_line(lines: RwSignal<Vec<Line>>, selected_line: RwSignal<Option<usize>>) {
    let Some(selected_id) = selected_line.get_untracked() else {
        return;
    };
    lines.update(|lines| lines.retain(|line| line.id != selected_id));
    selected_line.set(None);
}

#[cfg(test)]
mod tests {
    use super::{
        PaintingEdge, PreviewViewBox, default_preview_view_box, implicit_painting_bottom_right,
        inverse_svg_transform_delta, inverse_svg_transform_point, painting_edge_at_point,
        preview_pan_threshold_reached, resize_painting, translate_painting, wall_width,
        zoom_preview_view_box,
    };

    #[test]
    fn wall_preview_width_matches_line_length() {
        assert_eq!(wall_width(((0.0, 0.0), (3.0, 4.0))), 5.0);
        assert_eq!(wall_width(((-1.5, 2.0), (1.5, 2.0))), 3.0);
    }

    #[test]
    fn implicit_painting_bottom_right_is_square_and_balanced_on_wall() {
        assert_eq!(implicit_painting_bottom_right((0.25, 0.5)), (1.25, 1.5));
        assert_eq!(implicit_painting_bottom_right((2.0, 0.0)), (4.0, 2.0));
    }

    #[test]
    fn painting_drag_is_horizontal_without_custom_size() {
        assert_eq!(
            translate_painting((0.5, 0.25), None, (1.0, 0.75)),
            ((1.5, 0.25), None)
        );
    }

    #[test]
    fn painting_drag_preserves_custom_size_and_allows_vertical_movement() {
        assert_eq!(
            translate_painting((0.5, 0.25), Some((1.5, 1.25)), (1.0, 0.75)),
            ((1.5, 1.0), Some((2.5, 2.0)))
        );
    }

    #[test]
    fn svg_drag_delta_uses_the_inverse_screen_transform() {
        assert_eq!(
            inverse_svg_transform_delta((10.0, 0.0, 0.0, 10.0), (20.0, 30.0)),
            Some((2.0, 3.0))
        );
        assert_eq!(
            inverse_svg_transform_delta((1.0, 0.0, 0.0, 0.0), (1.0, 1.0)),
            None
        );
    }

    #[test]
    fn preview_zoom_keeps_the_pointer_anchor_fixed() {
        let initial = PreviewViewBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 2.0,
        };
        let zoomed = zoom_preview_view_box(initial, (2.0, 1.0), 0.5, 0.01, 10.0);

        assert_eq!(zoomed.width, 5.0);
        assert_eq!(zoomed.height, 1.0);
        assert_eq!(zoomed.x, 1.0);
        assert_eq!(zoomed.y, 0.5);
    }

    #[test]
    fn preview_zoom_respects_minimum_and_maximum_scale() {
        let initial = default_preview_view_box(10.0);
        let minimum = zoom_preview_view_box(initial, (0.0, 0.0), 0.001, 0.1, 10.05);
        let maximum = zoom_preview_view_box(initial, (0.0, 0.0), 100.0, 0.1, 10.05);

        assert_eq!(minimum.width, 0.1);
        assert_eq!(maximum.width, 10.05);
    }

    #[test]
    fn screen_point_conversion_accounts_for_svg_translation() {
        assert_eq!(
            inverse_svg_transform_point((10.0, 0.0, 0.0, 10.0, 100.0, 50.0), (120.0, 80.0)),
            Some((2.0, 3.0))
        );
    }

    #[test]
    fn preview_pan_threshold_ignores_click_jitter_but_accepts_drag_motion() {
        assert!(!preview_pan_threshold_reached((2.0, 2.0)));
        assert!(preview_pan_threshold_reached((3.0, 0.0)));
    }

    #[test]
    fn painting_edge_hit_testing_uses_a_screen_pixel_threshold() {
        assert_eq!(
            painting_edge_at_point(
                (0.0, 0.0, 2.0, 2.0),
                (10.0, 0.0, 0.0, 10.0),
                (0.5, 1.0),
                6.0
            ),
            Some(PaintingEdge::Left)
        );
        assert_eq!(
            painting_edge_at_point(
                (0.0, 0.0, 2.0, 2.0),
                (10.0, 0.0, 0.0, 10.0),
                (1.0, 1.0),
                6.0
            ),
            None
        );
    }

    #[test]
    fn custom_painting_edges_resize_without_moving_the_opposite_edges() {
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Left,
                (0.25, 0.0)
            ),
            ((0.75, 0.5), Some((1.5, 1.5)))
        );
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Bottom,
                (0.0, 0.25)
            ),
            ((0.5, 0.5), Some((1.5, 1.75)))
        );
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Bottom,
                (0.0, -0.25)
            ),
            ((0.5, 0.5), Some((1.5, 1.25)))
        );
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Right,
                (0.25, 0.0)
            ),
            ((0.5, 0.5), Some((1.75, 1.5)))
        );
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Right,
                (-0.25, 0.0)
            ),
            ((0.5, 0.5), Some((1.25, 1.5)))
        );
        assert_eq!(
            resize_painting(
                (0.5, 0.5),
                Some((1.5, 1.5)),
                PaintingEdge::Top,
                (0.0, -0.25)
            ),
            ((0.5, 0.25), Some((1.5, 1.5)))
        );
    }

    #[test]
    fn implicit_painting_resizing_keeps_square_shape_and_balanced_vertical_gaps() {
        let resized = resize_painting((0.5, 0.5), None, PaintingEdge::Left, (-0.25, 0.0));
        let (left, top) = resized.0;
        let (right, bottom) = implicit_painting_bottom_right((left, top));

        assert_eq!(resized.1, None);
        assert_eq!(right - left, bottom - top);
        assert_eq!(top, 1.0 - (bottom - top) / 2.0);

        let resized = resize_painting((0.5, 0.5), None, PaintingEdge::Top, (0.0, -0.25));
        let (left, top) = resized.0;
        let (right, bottom) = implicit_painting_bottom_right((left, top));
        assert_eq!(top, 0.25);
        assert_eq!(bottom, 1.75);
        assert_eq!(right - left, bottom - top);

        let resized = resize_painting((0.5, 0.5), None, PaintingEdge::Right, (0.25, 0.0));
        let (left, top) = resized.0;
        let (right, bottom) = implicit_painting_bottom_right((left, top));
        assert_eq!(right, 1.75);
        assert_eq!(right - left, bottom - top);

        let resized = resize_painting((0.5, 0.5), None, PaintingEdge::Bottom, (0.0, 0.25));
        let (left, top) = resized.0;
        let (right, bottom) = implicit_painting_bottom_right((left, top));
        assert_eq!(bottom, 1.75);
        assert_eq!(right - left, bottom - top);
    }
}
