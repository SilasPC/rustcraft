
use cgmath::Deg;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Settings {
    #[serde(default = "fov_def")]
    pub fov: Deg<f32>,
    #[serde(default = "one")]
    pub mouse_sensitivity: f32,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub vsync: bool,
    #[serde(default)]
    pub third_person: bool,
    #[serde(default = "two")]
    pub gui_scale: i32, // TODO implement
    #[serde(default)]
    pub debug: bool,
}

impl Settings {
    pub fn load() -> Self {
        let tomlstr = std::fs::read_to_string("assets/settings.toml").unwrap();
        toml::from_str(tomlstr.as_ref()).unwrap()
    }
}

const fn one() -> f32 {1.}
const fn two() -> i32 {2}
const fn fov_def() -> Deg<f32> {Deg(90.)}