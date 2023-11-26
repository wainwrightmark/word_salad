#define_import_path shaders::word_line

fn sdf(p: vec2<f32>, params: vec4<f32>) -> f32 {

    let arg1: u32 = bitcast<u32>(params.x);
    let arg2: u32 = bitcast<u32>(params.y);
    let arg3: u32 = bitcast<u32>(params.z);
    let line_width: f32 = params.w;

    return grid_sdf(p, arg1, arg2, arg3, line_width);

}

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn grid_sdf(p: vec2<f32>, master: u32, block1: u32, block2: u32, line_width: f32) -> f32 {
    var result = 1.0;

    var count = master % 256u;
    if (count == 0u) {return result;};
    var current = get_position2(master, 256u);
    result = min(result, sd_circle(p - current, line_width));
    count -= 1u;
    if (count == 0u) {return result;};
    var next = get_position2(master, 256u* 16u);
    result = min(result, line(p, current, next, line_width));
    count -= 1u;
    if (count == 0u) {return result;};
    current = next;
    var divisor: u32 = 1u;
    for (var index: u32 = 0u; index < 7u; index ++) {
        let next = get_position2(block1, divisor);
        let line =  line(p, current, next, line_width);
        result = min(result, line);
        count -= 1u;
        if (count == 0u) {return result;};
        current = next;
        divisor = divisor * 16u;
    }
    divisor = 1u;
    for (var index: u32 = 0u; index < 7u; index ++) {
        let next = get_position2(block2, divisor);
        let line =  line(p, current, next, line_width);
        result = min(result, line);
        count -= 1u;
        if (count == 0u) {return result;};
        current = next;
        divisor = divisor * 16u;
    }
    return result;
}



fn get_position2(arg1: u32, divisor: u32) -> vec2<f32> {
    let shifted = arg1 / divisor;
    let x: u32 = shifted % 4u;
    let y: u32 =( shifted / 4u) % 4u;

    return get_position(x, y);
}


fn get_position(x_bits: u32, y_bits: u32) -> vec2<f32> {

    let x_f = int_to_float(x_bits);
    let y_f = int_to_float(y_bits);

    let v = vec2<f32>((x_f - 1.5) * 0.5, (y_f - 1.3) * -0.5);

    return v;
}

fn int_to_float(u: u32) -> f32 {
    if u == 3u {return 3.0;};
    if u == 2u {return 2.0;};
    if u == 1u {return 1.0;};
    return 0.0;
}

fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, line_width: f32) -> f32 {
    let ba = (b - a);
    let pa = (p - a);
    let k: f32 = saturate(dot(pa, ba) / dot(ba, ba));
    let len = length(pa - (ba * k));
    let len_a = len - line_width;
    return len_a;
}