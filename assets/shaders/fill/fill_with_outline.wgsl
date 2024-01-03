#define_import_path fill::fill_with_outline

fn fill(d: f32, color: vec4<f32>, border: f32, border_color: vec3<f32>) -> vec4<f32> {

    let global_amount = sd_fill_alpha_fwidth(d);

    let border_d = abs(d + border)  - border;
    let border_amount = sd_fill_alpha_fwidth(border_d);
    let rgb = mix(color.rgb, border_color, border_amount );

    return vec4<f32>(rgb, color.a * global_amount);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}