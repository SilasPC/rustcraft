
#[derive(serde::Deserialize)]
pub struct Settings {
    pub fov: cgmath::Deg<f32>,
    pub mouse_sensitivity: f32,
}

impl Settings {
    pub fn load() -> Self {
        let tomlstr = std::fs::read_to_string("assets/settings.toml").unwrap();
        toml::from_str(tomlstr.as_ref()).unwrap()
    }
}