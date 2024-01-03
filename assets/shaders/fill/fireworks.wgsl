#define_import_path fill::fireworks

const NUM_PARTICLES =  50.;

fn Hash12_Polar(t: f32) -> vec2<f32> {

    let p_Angle = fract(sin(t * 674.3) * 453.2) * 6.2832;
    let p_Dist = fract(sin((t + p_Angle) * 714.3) * 263.2);

    return vec2<f32>(sin(p_Angle), cos(p_Angle)) * p_Dist;
}

fn Explosion(uv: vec2<f32>, t: f32) -> f32 {

    var sparks = 0.;

    for (var i: f32 = 0.; i < NUM_PARTICLES; i += 1.0) {

        let dir = Hash12_Polar(i + 1.) * .5;
        let dist = length(uv - dir * t);
        var brightness = mix(.0005, .0005, smoothstep(.05, 0., t));

        brightness *= sin(t * 20. + i) * .5 + .5;
        brightness *= smoothstep(1., .6, t);
        sparks += brightness / dist;
    }
    return sparks;
}

fn fill(color: vec3<f32>, point: vec2<f32>, progress: f32) -> vec4<f32> {
    let explosion = Explosion(point, progress);
    let a = smoothstep(0.1, 0.5, explosion);
    let col = explosion * color.rgb * 2.;

    return vec4(col, a);
}