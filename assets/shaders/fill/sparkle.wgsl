
#define_import_path smud::sparkle

#import smud::view_bindings::globals

fn modulo_euclidean(a: f32, b: f32) -> f32 {
    var m = a % b;
    if m < 0.0 {
        if b < 0.0 {
            m -= b;
        } else {
            m += b;
        }
    }
    return m;
}

fn Hash31(p: vec3<f32>) -> f32 {
    return fract(937.276 * cos(836.826 * p.x + 263.736 * p.y + 374.723 * p.z + 637.839));
}

fn fill(d: f32, color1: vec4<f32>, uv: vec2<f32>, count1: f32, count2: f32, seed: f32) -> vec4<f32> {

    var color = vec3(0.0);
    let time: f32 = globals.time;
    var alpha = 0.0;

    for (var i = count1 * -1.0; i <= count1; i += 1.25) {
        for (var j = count2 * -1.0; j <= count2; j += 1.25) {
            var p = uv;

            let freq = fract(643.376  *  cos(264.863 * i + 136.937)) + 1.0;
            var pos = 5.0 * vec2(i, j) + vec2(sin(freq * (time + 10.0 * j) - i), freq * time);
            pos.y = modulo_euclidean(pos.y + 15.0 + seed, 30.0) - 15.0;
            pos.x *= 0.1 * pos.y + 1.0;
            p -= 0.2 * pos;

            let an = modulo_euclidean(atan2(p.y, p.x) + 6.2831 / 3.0, 6.2831 / 6.0) - 6.2831 / 3.0;
            p = vec2(cos(an), sin(an)) * length(p);

            let sec: f32 = floor(time * 2.0);
            let frac: f32 = fract(time * 2.0 );
            let flicker: f32 = mix(Hash31(vec3(i, j * seed, sec)), Hash31(vec3(i, j * seed, sec + 1.0)), frac);

            let rad: f32 = 25.0 + 20.0 * flicker;
            let br = 250.0 * pow(1.0 / max(10.0, rad * (sqrt(abs(p.x)) + sqrt(abs(p.y))) + 0.9), 2.5);
            let rand = fract(847.384 * cos(483.846 * seed * i + 737.487 * j + 264.836));
            if rand > 0.5 {
                color += mix(vec3(br, 0.4 * br, 0.0), vec3(1.0), br);
            } else {
                color += mix(vec3(0.0, 0.0, 0.6 * br), vec3(1.0), br);
            }

            color *= 0.955 + 0.1 * flicker;
            alpha += br * step(0.001, br);
        }
    }

    return vec4(color, alpha);
}


