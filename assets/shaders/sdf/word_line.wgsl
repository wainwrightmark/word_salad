#define_import_path shaders::word_line

fn sdf(p: vec2<f32>, line_width: f32, f_points1: f32, f_points2: f32, segments: f32) -> f32 {

    let points1: u32 = bitcast<u32>(f_points1);
    let points2: u32 = bitcast<u32>(f_points2);



    return grid_sdf(p, line_width, points1,points2, segments);

}

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn grid_sdf(p: vec2<f32>, line_width: f32, points1: u32, points2: u32, segments: f32) -> f32 {
    var result = 1.0;
    var segments_remaining = segments;
    if segments_remaining <= 0.0{
            return result;
    }
    var current = get_position2(points1, 1u);

    result = sd_circle(p - current, line_width);

    segments_remaining -= 1.0;

    var divisor: u32 = 16u;
    for (var index: u32 = 1u; index < 8u; index ++) {
        if segments_remaining <= 0.0{
            return result;
        }
        let next = get_position2(points1, divisor);
        let line =  line(p, current, next, line_width, min(segments_remaining, 1.0) );
        result = min(result, line);
        current = next;
        segments_remaining -= 1.0;
        divisor = divisor * 16u;
    }

    divisor = 1u;
    for (var index: u32 = 0u; index < 8u; index ++) {
        if segments_remaining <= 0.0{
            return result;
        }
        let next = get_position2(points2, divisor);
        let line =  line(p, current, next, line_width, min(segments_remaining, 1.0));
        result = min(result, line);
        current = next;
        segments_remaining -= 1.0;
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

    let v = vec2<f32>((x_f - 1.5) * 0.5, (y_f - 1.5) * -0.5);

    return v;
}

fn int_to_float(u: u32) -> f32 {
    if u == 3u {return 3.0;};
    if u == 2u {return 2.0;};
    if u == 1u {return 1.0;};
    return 0.0;
}

fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, line_width: f32, line_proportion: f32 ) -> f32 {
    let ba = (b - a) * line_proportion;
    let pa = (p - a);
    let k: f32 = saturate(dot(pa, ba) / dot(ba, ba));
    let len = length(pa - (ba * k));
    let len_a = len - line_width;
    return len_a;
}