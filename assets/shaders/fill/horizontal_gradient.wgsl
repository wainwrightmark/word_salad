#define_import_path fill::horizontal_gradient

fn fill(d: f32, color: vec4<f32>, p: vec2<f32>, offset: f32, col2: vec3<f32>) -> vec4<f32> {

    let a = sd_fill_alpha_fwidth(d);
    let amount = step(offset - (p.x * 0.5) , 0.5);
    let mixed_color = mix(col2,color.rgb,  amount);

    return vec4<f32>(mixed_color, a * color.a);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}