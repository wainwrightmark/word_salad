#define_import_path fill::fireworks
#import smud::view_bindings::globals

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

fn fill(d: f32, color1: vec4<f32>, point: vec2<f32>, fract: f32) -> vec4<f32> {
    if d >= 0.0  {
        return vec4<f32>(0.0);
    }

    var col = vec3(0.0);


    var color = color1.rgb;

    col += Explosion(point, fract) * color;

    col *= 2.;

    let a = max(max(col.x, col.y), col.z);
    return vec4(col, a);
}