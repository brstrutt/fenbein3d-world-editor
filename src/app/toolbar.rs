use super::zoom::{ZOOM_STEP, change_zoom};
use crate::line::Line;
use leptos::prelude::*;

#[component]
pub fn WorkspaceToolbar(
    lines: RwSignal<Vec<Line>>,
    next_line_id: RwSignal<usize>,
    zoom: RwSignal<f64>,
) -> impl IntoView {
    let add_line = move |_| {
        let id = next_line_id.get_untracked();
        next_line_id.set(id + 1);
        lines.update(|lines| lines.push(Line::new(id)));
    };
    let zoom_in = move |_| change_zoom(zoom, ZOOM_STEP);
    let zoom_out = move |_| change_zoom(zoom, 1.0 / ZOOM_STEP);

    view! {
        <header class="workspace-heading">
            <h1>"World Editor"</h1>
            <div class="workspace-actions">
                <button class="add-action add-line" type="button" on:click=add_line>
                    "+ Add line"
                </button>
                <div class="zoom-controls">
                    <button
                        class="zoom-button"
                        type="button"
                        aria-label="Zoom out"
                        title="Zoom out"
                        on:click=zoom_out
                    >
                        "-"
                    </button>
                    <output class="zoom-level">
                        {move || format!("{:.0}%", zoom.get() * 100.0)}
                    </output>
                    <button
                        class="zoom-button"
                        type="button"
                        aria-label="Zoom in"
                        title="Zoom in"
                        on:click=zoom_in
                    >
                        "+"
                    </button>
                </div>
            </div>
        </header>
    }
}
