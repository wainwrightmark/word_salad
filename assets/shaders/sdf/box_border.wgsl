#define_import_path shaders::tile_border


// params.x is width ratio
// params.y is height ratio
// params.z is rounding ratio for all four corners
// params.w is the border width
fn sdf(p: vec2<f32>, params: vec4<f32>) -> f32 {

    let tile = sd_rounded_box(p, vec2<f32>(params.x, params.y), vec4<f32>(params.z, params.z, params.z, params.z));

    let border = abs(tile) - params.w;

    return border;
}



fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var r_2 = r;
    // swizzle assigment isn't supported yet
    // r_2.xy = select(r_2.zw, r_2.xy, p.x > 0.);
    let tmp = select(r_2.zw, r_2.xy, p.x > 0.);
    r_2.x = tmp.x;
    r_2.y = tmp.y;
    r_2.x = select(r_2.y, r_2.x, p.y > 0.);
    let q = abs(p) - b + r_2.x;
    return min(
        max(q.x, q.y),
        0.
    ) + length(max(q, vec2<f32>(0.))) - r_2.x;
}