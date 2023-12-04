#define_import_path fill::gradient

fn fill(d: f32, color1: vec4<f32>, point: vec2<f32>, offset: f32, col2_r: f32, col2_g: f32, col2_b: f32) -> vec4<f32> {

    // let d1 = sd_fill_alpha_fwidth(d + 0.5);
    return mix(color1, vec4(col2_r, col2_g, col2_b, color1.a), d + offset);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}