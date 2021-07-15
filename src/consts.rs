
/// Tick interval duration
pub const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(50);
/// Number of random ticks per chunk per game tick
pub const RANDOM_TICK_SPEED: usize = 3;
/// Sky minimum brightness
pub const SKY_MIN_BRIGHTNESS: f32 = 0.4;
/// Minimum block brightness
pub const MIN_BRIGHTNESS: f32 = 0.4;
/// Sky color
pub const SKY: (f32,f32,f32) = (110./256., 160./256., 240./256.,);