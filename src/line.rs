use leptos::{ev, prelude::*};

pub const WORLD_UNIT_PX: f64 = 32.0;

#[derive(Clone, Copy)]
pub struct Line {
    pub id: usize,
    start: RwSignal<(f64, f64)>,
    end: RwSignal<(f64, f64)>,
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
        }
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
    selected_line: RwSignal<Option<usize>>,
) -> impl IntoView {
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

    view! {
        <aside class="line-inspector" hidden=move || selected_line.get().is_none()>
            <header class="inspector-heading">
                <div>
                    <p>"Selected line"</p>
                    <h2>{move || selected().map_or_else(|| "Line".to_owned(), |line| format!("Line {}", line.id + 1))}</h2>
                </div>
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
