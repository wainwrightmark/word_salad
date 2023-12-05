#define_import_path fill::horizontal_gradient

fn fill(d: f32, color: vec4<f32>, p: vec2<f32>, offset: f32, col2_r: f32, col2_g: f32, col2_b: f32) -> vec4<f32> {

    let a = sd_fill_alpha_fwidth(d);
    let other_color = vec3<f32>(col2_r, col2_g, col2_b);
    let amount = clamp((p.x + offset), 0.0, 1.0);
    let mixed_color = mix(color.rgb, other_color, amount);

    return vec4<f32>(mixed_color, a * color.a);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}