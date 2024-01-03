#define_import_path shaders::play_pause

fn sdf(p: vec2<f32>, progress: f32 ) -> f32 {
    return mix(pause(p), play(p), progress) - 0.05;
}

fn pause(p: vec2<f32>)->f32{
     let left = sd_box(p + vec2<f32>(0.2, 0.0) ,vec2<f32>(0.1, 0.4)) ;
     let right = sd_box(p - vec2<f32>(0.2, 0.0),vec2<f32>(0.1, 0.4) );

    return min(left, right);

}

fn play(p: vec2<f32>)->f32{
    return sd_equilateral_triangle(p.yx, 0.5);
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}

fn sd_equilateral_triangle(p: vec2<f32>, r: f32) -> f32 {
    var p_2 = p;
    let k = sqrt(3.);
    p_2.x = abs(p_2.x) - r;
    p_2.y = p_2.y + r / k;
    if (p_2.x + k * p_2.y > 0.) {
        p_2 = vec2<f32>(p_2.x - k * p_2.y, -k * p_2.x - p_2.y) / 2.;
    }
    p_2.x = p_2.x - clamp(p_2.x, -2. * r, 0.);
    return -length(p_2) * sign(p_2.y);
}