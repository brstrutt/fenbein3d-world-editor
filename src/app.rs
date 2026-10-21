mod json_window;
mod toolbar;
mod world_scene;
mod zoom;

use crate::line::{Line, LineInspector};
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let lines = RwSignal::new(Vec::<Line>::new());
    let next_line_id = RwSignal::new(0usize);
    let selected_line = RwSignal::new(None::<usize>);
    let zoom = RwSignal::new(1.0_f64);
    let scene_offset = RwSignal::new((0.0, 0.0));

    view! {
        <main class="workspace">
            <toolbar::WorkspaceToolbar lines=lines next_line_id=next_line_id zoom=zoom />
            <world_scene::WorldScene
                lines=lines
                selected_line=selected_line
                scene_offset=scene_offset
                zoom=zoom
            />
            <LineInspector lines=lines selected_line=selected_line />
            <json_window::JsonWindow
                lines=lines
                next_line_id=next_line_id
                selected_line=selected_line
            />
        </main>
    }
}
