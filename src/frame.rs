use std::time::{Duration, Instant};
use bevy::prelude::*;

pub struct FramePlugin;

impl Plugin for FramePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(FrameTimer::new(60, Duration::from_micros(1000)))
            .add_system(framerate_exact_limiter);
    }
}

#[derive(Debug)]
struct FrameTimer {
    enabled: bool,
    framerate_target: u64,
    frame_start: Instant,
    render_start: Instant,
    exact_sleep: Duration,
    /// How early should we cut the sleep time by, to make sure we have enough time to render our
    /// frame if it takes longer than expected? Increasing this number makes dropped frames less
    /// likely, but increases motion-to-photon latency of user input rendered to screen.
    safety_margin: Duration,
}

impl FrameTimer {
    fn new(framerate_limit: u64, margin: Duration) -> Self {
        FrameTimer {
            enabled: true,
            frame_start: Instant::now(),
            render_start: Instant::now(),
            exact_sleep: Duration::from_millis(0),
            framerate_target: framerate_limit,
            safety_margin: margin,
        }
    }
}

fn framerate_limit_forward_estimator(mut timer: ResMut<FrameTimer>) {
    let render_end = Instant::now();
    let target_frametime = Duration::from_micros(1_000_000 / timer.framerate_target);
    let last_frametime = render_end.duration_since(timer.frame_start);
    let last_render_time = last_frametime - timer.exact_sleep;
    let estimated_cpu_time_needed = last_render_time + timer.safety_margin;
    let estimated_sleep_time = target_frametime - target_frametime.min(estimated_cpu_time_needed);
    if timer.enabled {
        spin_sleep::sleep(estimated_sleep_time);
    }
    timer.frame_start = Instant::now();
}

fn framerate_exact_limiter(mut timer: ResMut<FrameTimer>) {
    let system_start = Instant::now();
    let target_frametime = Duration::from_micros(1_000_000 / timer.framerate_target);
    let sleep_needed =
        target_frametime - target_frametime.min(system_start.duration_since(timer.render_start));
    if timer.enabled {
        spin_sleep::sleep(sleep_needed);
    }
    timer.render_start = Instant::now();
    timer.exact_sleep = timer.render_start.duration_since(system_start);
}