#define_import_path shaders::circle

fn sdf(p: vec2<f32>, ) -> f32 {
    return length(p) - 1.0;
}