#[derive(Clone, Copy, Default)]
pub struct ReducedMotion(pub bool);

impl gpui::Global for ReducedMotion {}

impl ReducedMotion {
    pub fn enabled(cx: &gpui::App) -> bool {
        cx.try_global::<Self>().is_some_and(|setting| setting.0)
    }
}

pub(crate) fn next_random(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
    *state
}
pub(crate) fn ease_out(t: f32) -> f32 {
    1. - (1. - t.clamp(0., 1.)).powi(2)
}
pub(crate) fn row_slide(frame: f32) -> f32 {
    -36. * (1. - ease_out(frame / 20.))
}
pub(crate) fn row_alpha(frame: f32) -> f32 {
    ease_out(frame / 20.)
}
