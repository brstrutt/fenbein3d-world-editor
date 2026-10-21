use super::zoom::{ZOOM_STEP, change_zoom};
use crate::line::{Line, WORLD_UNIT_PX, WorldLine};
use leptos::{ev, prelude::*};

const SNAP_DISTANCE: f64 = 14.0;

fn snap_delta(
    candidates: &[(f64, f64)],
    lines: &[Line],
    moving_line_id: usize,
    screen_scale: f64,
) -> Option<(f64, f64)> {
    let mut closest: Option<(f64, f64, f64)> = None;
    let max_distance_squared = SNAP_DISTANCE * SNAP_DISTANCE;

    for &(x, y) in candidates {
        for line in lines.iter().filter(|line| line.id != moving_line_id) {
            let (start, end) = line.endpoints();
            for (target_x, target_y) in [start, end] {
                let delta_x = target_x - x;
                let delta_y = target_y - y;
                let distance_squared =
                    (delta_x * delta_x + delta_y * delta_y) * screen_scale * screen_scale;

                if distance_squared <= max_distance_squared
                    && closest.map_or(true, |(_, _, distance)| distance_squared < distance)
                {
                    closest = Some((delta_x, delta_y, distance_squared));
                }
            }
        }
    }

    closest.map(|(delta_x, delta_y, _)| (delta_x, delta_y))
}

#[component]
pub fn WorldScene(
    lines: RwSignal<Vec<Line>>,
    selected_line: RwSignal<Option<usize>>,
    scene_offset: RwSignal<(f64, f64)>,
    zoom: RwSignal<f64>,
) -> impl IntoView {
    let active_endpoint = RwSignal::new(None::<(usize, bool)>);
    let active_line = RwSignal::new(None::<usize>);
    let endpoint_drag_start = RwSignal::new(None::<(f64, f64, f64, f64)>);
    let line_drag_start = RwSignal::new(None::<(f64, f64, f64, f64, f64, f64)>);
    let pan_start = RwSignal::new(None::<(f64, f64, f64, f64)>);

    let move_listener = window_event_listener(ev::pointermove, move |event| {
        if let (Some((line_id, is_start)), Some((pointer_x, pointer_y, x, y))) = (
            active_endpoint.get_untracked(),
            endpoint_drag_start.get_untracked(),
        ) {
            let scale = WORLD_UNIT_PX * zoom.get_untracked();
            let position = (
                x + (f64::from(event.client_x()) - pointer_x) / scale,
                y + (f64::from(event.client_y()) - pointer_y) / scale,
            );

            lines.with_untracked(|lines| {
                if let Some(line) = lines.iter().find(|line| line.id == line_id) {
                    let delta = snap_delta(&[position], lines, line_id, scale);
                    let position =
                        delta.map_or(position, |(x, y)| (position.0 + x, position.1 + y));
                    line.set_endpoint(is_start, position);
                }
            });
        } else if let (
            Some(line_id),
            Some((pointer_x, pointer_y, start_x, start_y, end_x, end_y)),
        ) = (active_line.get_untracked(), line_drag_start.get_untracked())
        {
            let scale = WORLD_UNIT_PX * zoom.get_untracked();
            let delta_x = (f64::from(event.client_x()) - pointer_x) / scale;
            let delta_y = (f64::from(event.client_y()) - pointer_y) / scale;

            lines.with_untracked(|lines| {
                if let Some(line) = lines.iter().find(|line| line.id == line_id) {
                    let start = (start_x + delta_x, start_y + delta_y);
                    let end = (end_x + delta_x, end_y + delta_y);
                    let delta = snap_delta(&[start, end], lines, line_id, scale);
                    let (snap_x, snap_y) = delta.unwrap_or((0.0, 0.0));
                    line.set_endpoints(
                        (start.0 + snap_x, start.1 + snap_y),
                        (end.0 + snap_x, end.1 + snap_y),
                    );
                }
            });
        } else if let Some((start_x, start_y, offset_x, offset_y)) = pan_start.get_untracked() {
            scene_offset.set((
                offset_x + f64::from(event.client_x()) - start_x,
                offset_y + f64::from(event.client_y()) - start_y,
            ));
        }
    });
    let stop_drag = move |_| {
        active_endpoint.set(None);
        endpoint_drag_start.set(None);
        active_line.set(None);
        line_drag_start.set(None);
        pan_start.set(None);
    };
    let up_listener = window_event_listener(ev::pointerup, stop_drag);
    let cancel_listener = window_event_listener(ev::pointercancel, stop_drag);

    on_cleanup(move || {
        move_listener.remove();
        up_listener.remove();
        cancel_listener.remove();
    });

    view! {
        <div
            class="world-scene"
            class:panning=move || pan_start.get().is_some()
            style:background-position=move || {
                let (x, y) = scene_offset.get();
                format!("calc(50% + {x}px) calc(50% + {y}px)")
            }
            style:background-size=move || {
                let grid_size = WORLD_UNIT_PX * zoom.get();
                format!("{grid_size}px {grid_size}px")
            }
            on:wheel=move |event: ev::WheelEvent| {
                event.prevent_default();
                    change_zoom(zoom, if event.delta_y() < 0.0 { ZOOM_STEP } else { 1.0 / ZOOM_STEP });
            }
            on:pointerdown=move |event: ev::PointerEvent| {
                event.prevent_default();
                let (offset_x, offset_y) = scene_offset.get_untracked();
                pan_start.set(Some((
                    f64::from(event.client_x()),
                    f64::from(event.client_y()),
                    offset_x,
                    offset_y,
                )));
            }
        >
            <For
                each=move || lines.get()
                key=|line| line.id
                children=move |line| view! {
                    <WorldLine
                        line=line
                        active_endpoint=active_endpoint
                        endpoint_drag_start=endpoint_drag_start
                        active_line=active_line
                        line_drag_start=line_drag_start
                        selected_line=selected_line
                        scene_offset=scene_offset
                        zoom=zoom
                    />
                }
            />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::snap_delta;
    use crate::line::{Line, WORLD_UNIT_PX};

    #[test]
    fn snapping_threshold_is_consistent_across_zoom_levels() {
        let lines = [Line::from_endpoints(1, (0.0, 0.0), (10.0, 0.0))];

        assert_eq!(
            snap_delta(&[(0.4, 0.0)], &lines, 0, WORLD_UNIT_PX),
            Some((-0.4, 0.0))
        );
        assert_eq!(snap_delta(&[(0.5, 0.0)], &lines, 0, WORLD_UNIT_PX), None);
        assert_eq!(
            snap_delta(&[(0.2, 0.0)], &lines, 0, WORLD_UNIT_PX * 2.0),
            Some((-0.2, 0.0))
        );
    }
}
