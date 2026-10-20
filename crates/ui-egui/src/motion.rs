//! Motion grammar: every animation in brahma's interface has a purpose (confirm an action, show
//! a state change, reveal where something came from) and one of a few shapes. Durations are
//! short, entrances decelerate, exits are quicker than entrances, and position changes settle
//! like a lightly damped spring rather than stopping dead. Preferences › Interface › Enable
//! Animations turns all of it off (see [`time`]).
//!
//! The curves are closed-form, so a frame costs one `exp` and two trig calls, nothing is
//! integrated and nothing accumulates error: fine at any frame rate, including a throttled
//! background tab.

/// How long an entrance (a drawer sliding in, a panel expanding) takes, in seconds.
pub const ENTER: f32 = 0.24;
/// How long an exit takes: quicker, so the interface feels light.
pub const EXIT: f32 = 0.16;
/// A small state change (hover, a toggle): shorter still.
pub const QUICK: f32 = 0.11;

/// `seconds` when animations are on, else 0 (everything snaps), following Preferences ›
/// Interface › Enable Animations, which `prefs_ui::sync_animations` mirrors into egui's style.
pub fn time(ctx: &egui::Context, seconds: f32) -> f32 {
    if ctx.global_style().animation_time > 0.0 { seconds } else { 0.0 }
}

/// A lightly under-damped spring settling at 1: fast start, one small overshoot (about 4 %),
/// settled by `t = 1`. For things that move into place: drawers, panels, sliding tabs.
pub fn spring(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    // Damping ratio and natural frequency chosen so the response is within 0.1 % of 1 at t = 1.
    let zeta = 0.72_f32;
    let omega = 12.0_f32;
    let wd = omega * (1.0 - zeta * zeta).sqrt();
    let decay = (-zeta * omega * t).exp();
    1.0 - decay * ((wd * t).cos() + zeta / (1.0 - zeta * zeta).sqrt() * (wd * t).sin())
}

/// Decelerating curve for entrances that must not overshoot (opacity, a reveal).
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Accelerating curve for exits: things leave faster than they arrive.
pub fn ease_in(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

/// Animate `open` for `id` as a 0..1 value: the spring on the way in, the quicker ease-in on the
/// way out. Requests repaints while moving; returns the end value at once with animations off.
pub fn open_progress(ctx: &egui::Context, id: egui::Id, open: bool) -> f32 {
    let (secs, easing): (f32, fn(f32) -> f32) = if open { (ENTER, spring) } else { (EXIT, ease_in) };
    ctx.animate_bool_with_time_and_easing(id, open, time(ctx, secs), easing)
}

/// A one-way entrance for `id` (a dialog appearing): 0 on the first frame it is drawn, easing
/// out to 1 over [`ENTER`]. Use it as an opacity. 1 at once with animations off.
pub fn enter_progress(ctx: &egui::Context, id: egui::Id) -> f32 {
    ctx.animate_bool_with_time_and_easing(id, true, time(ctx, ENTER), ease_out)
}

/// A hover or other small state as a smooth 0..1 for `id` (over [`QUICK`]), to lerp a fill or
/// a stroke colour instead of flipping it.
pub fn state(ctx: &egui::Context, id: egui::Id, on: bool) -> f32 {
    ctx.animate_bool_with_time_and_easing(id, on, time(ctx, QUICK), ease_out)
}

/// A size moving to `target` (a dock group collapsing or expanding): follows over [`ENTER`]
/// with the spring's deceleration. `target` at once with animations off.
pub fn size(ctx: &egui::Context, id: egui::Id, target: f32) -> f32 {
    ctx.animate_value_with_time(id, target, time(ctx, ENTER))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spring_starts_at_rest_overshoots_a_little_and_settles() {
        assert_eq!(spring(0.0), 0.0);
        assert_eq!(spring(1.0), 1.0);
        assert_eq!(spring(-1.0), 0.0);
        assert_eq!(spring(2.0), 1.0);
        let peak = (0..=100).map(|i| spring(i as f32 / 100.0)).fold(0.0_f32, f32::max);
        assert!(peak > 1.0 && peak < 1.06, "one small overshoot: {peak}");
        assert!((spring(0.999) - 1.0).abs() < 0.002, "settled by the end: {}", spring(0.999));
        // Fast out of the gate: past the half-way point in the first fifth of the time.
        assert!(spring(0.2) > 0.5, "{}", spring(0.2));
    }

    #[test]
    fn with_animations_off_every_helper_snaps_to_its_target() {
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| s.animation_time = 0.0);
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let ctx = ui.ctx();
            assert_eq!(enter_progress(ctx, egui::Id::new("e")), 1.0);
            assert_eq!(state(ctx, egui::Id::new("s"), true), 1.0);
            assert_eq!(state(ctx, egui::Id::new("s2"), false), 0.0);
            assert_eq!(size(ctx, egui::Id::new("z"), 120.0), 120.0);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn the_eases_are_monotonic_and_bounded() {
        for f in [ease_out as fn(f32) -> f32, ease_in] {
            let mut prev = f(0.0);
            assert_eq!(prev, 0.0);
            for i in 1..=50 {
                let v = f(i as f32 / 50.0);
                assert!(v >= prev && (0.0..=1.0).contains(&v));
                prev = v;
            }
            assert_eq!(prev, 1.0);
        }
    }

    #[test]
    fn animations_off_makes_every_duration_zero() {
        let ctx = egui::Context::default();
        assert!(time(&ctx, ENTER) > 0.0);
        ctx.all_styles_mut(|s| s.animation_time = 0.0);
        assert_eq!(time(&ctx, ENTER), 0.0);
    }
}
