#define_import_path voronoi_gradient
#import smud::view_bindings::globals


const SIZE = 30.;





fn ran(uv: vec2<f32>) -> vec2<f32> {
    let uv1 = uv * vec2<f32>(dot(uv, vec2<f32>(127.1, 311.7)), dot(uv, vec2<f32>(227.1, 521.7)));
    return 1.0 - fract(tan(cos(uv1) * 123.6) * 3533.3) * fract(tan(cos(uv1) * 123.6) * 3533.3);
}

fn pt(id: vec2<f32>) -> vec2<f32> {
    let t: f32 = globals.time * 2.0;
    let ret = sin(t * (ran(id + .5) - 0.5) + ran(id - 20.1) * 8.0) * 0.5;
    return ret;
}


fn fill (d: f32, color1: vec4<f32>, point: vec2<f32>, col2_r: f32, col2_g: f32, col2_b: f32) -> vec4<f32> {
    let iTime = globals.time;
    var uv = point;
    var off: vec2<f32> = iTime / vec2<f32>(50., 30.);
    uv += off;
    uv *= SIZE;

    let gv: vec2<f32> = fract(uv) - .5;
    let id: vec2<f32> = floor(uv);

    var  mindist: f32 = 1e9;
    var vorv: vec2<f32>;
    for (var i = -1.; i <= 1.; i+= 1.0) {
        for (var j = -1.; j <= 1.; j+= 1.0) {
            let offv = vec2<f32>(i, j);
            let dist = length(gv + pt(id + offv) - offv);
            if dist < mindist {
                mindist = dist;
                vorv = (id + pt(id + offv) + offv) / SIZE - off;
            }
        }
    }


    let col2 = vec3(col2_r, col2_g, col2_b);

    let col: vec3<f32> = mix(color1.rgb, col2, clamp(vorv.x * 2.2 + vorv.y, -1., 1.) * 0.5 + 0.5);

    return vec4(col, 1.0);

    // /*
    // fragColor += vec4(vec3(smoothstep(0.08,0.05,gv.x+pt(id).x)),0.0);
    // fragColor -= vec4(vec3(smoothstep(0.05,0.03,gv.x+pt(id).x)),0.0);
	// */
}