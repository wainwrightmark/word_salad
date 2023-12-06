#define_import_path fill::fireworks
#import smud::view_bindings::globals

const NUM_EXPLOSIONS = 5.;
const NUM_PARTICLES =  75.;

fn Hash12(t: f32) -> vec2<f32> {

    let x = fract(sin(t * 674.3) * 453.2);
    let y = fract(sin((t + x) * 714.3) * 263.2);

    return vec2<f32>(x, y);
}

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

fn fill(d: f32, color1: vec4<f32>, point: vec2<f32>) -> vec4<f32> {
    if d >= 0.0 {
        return vec4<f32>(0.0);
    }

    var col = vec3(0.0);
    let iTime: f32 = globals.time;

    for (var i: f32 = 0.; i < NUM_EXPLOSIONS; i += 1.0) {
        var t = iTime + i / NUM_EXPLOSIONS;
        var ft = floor(t);
        var color = sin(4. * vec3(.34, .54, .43) * ft) * .25 + .75;


        var offset = Hash12(i + 1. + ft) - .5;
        offset *= vec2<f32>(1.77, 1.);
        //col+=.0004/length(uv-offset);

        col +=  Explosion(point - offset, fract(t)) * color;
    }

    col *= 2.;

    let a = max(max(col.x, col.y), col.z);
    return vec4(col, a);
}