#define_import_path smud::fill_with_outline

#import smud


fn fill(d: f32, color: vec4<f32>, pos: vec2<f32>, border: f32, border_r: f32, border_g: f32, border_b: f32) -> vec4<f32> {

    let global_amount = sd_fill_alpha_fwidth(d);



    let border_color = vec3<f32>(border_r, border_g, border_b);
    let border_d = abs(d + border)  - border;
    let border_amount = sd_fill_alpha_fwidth(border_d);
    let rgb = mix(color.rgb, border_color, border_amount * 2.0 );

    return vec4<f32>(rgb, color.a * global_amount);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}