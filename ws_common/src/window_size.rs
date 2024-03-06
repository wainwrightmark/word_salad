use bevy::window::WindowResizeConstraints;

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct WindowSizeValues {
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

impl WindowSizeValues {
    #[cfg(target_arch = "wasm32")]
    pub fn from_web_window() -> Self {
        let web_window = web_sys::window().expect("no global `window` exists");
        let width: f32 = web_window.inner_width().unwrap().as_f64().unwrap() as f32;
        let height: f32 = web_window.inner_height().unwrap().as_f64().unwrap() as f32;
        let device_pixel_ratio: f32 = web_window.device_pixel_ratio() as f32;

        Self {
            width,
            height,
            device_pixel_ratio,
        }
    }

    pub fn to_window_resolution(&self) -> bevy::window::WindowResolution {
        let mut res = bevy::window::WindowResolution::default();

        res.set_scale_factor(self.device_pixel_ratio);
        res.set(self.width, self.height);

        res
    }

    pub fn clamp_to_resize_constraints(&mut self, constraints: &WindowResizeConstraints) {
        self.width = self
            .width
            .clamp(constraints.min_width, constraints.max_width);
        self.height = self
            .height
            .clamp(constraints.min_height, constraints.max_height);
    }
}
