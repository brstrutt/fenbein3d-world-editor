use leptos::prelude::{RwSignal, Update};

const MIN_ZOOM: f64 = 0.25;
const MAX_ZOOM: f64 = 4.0;
pub const ZOOM_STEP: f64 = 1.2;

pub fn change_zoom(zoom: RwSignal<f64>, factor: f64) {
    zoom.update(|zoom| *zoom = (*zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM));
}
