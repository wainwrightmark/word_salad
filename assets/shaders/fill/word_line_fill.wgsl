#define_import_path smud::word_line_fill

#import smud

fn fill(d: f32, color: vec4<f32>, p: vec2<f32>, line_width: f32, segments: f32, points1: u32, points2: u32, points3: u32, points4: u32) -> vec4<f32> {

    let d_and_s = dist_and_segment(p, line_width, segments, points1, points2, points3, points4);

    let r_color = get_color(d_and_s.segment);
    let a = sd_fill_alpha_nearest(d_and_s.dist);

    return vec4<f32>(r_color, a);
}

// Dirt cheap, but ugly
fn sd_fill_alpha_nearest(distance: f32) -> f32 {
    return step(distance, 0.);
}

struct DistAndSegment {
    dist: f32,
    segment: u32
}

fn dist_and_segment(p: vec2<f32>, line_width: f32, segments: f32, points1: u32, points2: u32, points3: u32, points4: u32) -> DistAndSegment {
    var result = DistAndSegment(1.0, 0u);
    var segments_remaining = segments;


    if segments_remaining <= 0.0 {
        return result;
    }
    var current = get_position2(points1, 1u);

    result.dist = sd_circle(p - current, line_width);

    segments_remaining -= 1.0;

    var divisor: u32 = 16u;
    for (var index: u32 = 1u; index < 4u; index ++) {
        if segments_remaining <= 0.0 {
            return result;
        }
        let next = get_position2(points1, divisor);
        let line = line(p, current, next, line_width, min(segments_remaining, 1.0));

        if line < 0.0 {
            result = DistAndSegment(line, index);
        }
        current = next;
        segments_remaining -= 1.0;
        divisor = divisor * 16u;
    }

    divisor = 1u;
    for (var index: u32 = 0u; index < 4u; index ++) {
        if segments_remaining <= 0.0 {
            return result;
        }
        let next = get_position2(points2, divisor);
        let line = line(p, current, next, line_width, min(segments_remaining, 1.0));

        if line < 0.0 {
            result = DistAndSegment(line, index + 4u);
        }

        current = next;
        segments_remaining -= 1.0;
        divisor = divisor * 16u;
    }

    divisor = 1u;
    for (var index: u32 = 0u; index < 4u; index ++) {
        if segments_remaining <= 0.0 {
            return result;
        }
        let next = get_position2(points3, divisor);
        let line = line(p, current, next, line_width, min(segments_remaining, 1.0));

        if line < 0.0 {
            result = DistAndSegment(line, index + 8u);
        }

        current = next;
        segments_remaining -= 1.0;
        divisor = divisor * 16u;
    }

    divisor = 1u;
    for (var index: u32 = 0u; index < 4u; index ++) {
        if segments_remaining <= 0.0 {
            return result;
        }
        let next = get_position2(points4, divisor);
        let line = line(p, current, next, line_width, min(segments_remaining, 1.0));


        if line < 0.0 {
            result = DistAndSegment(line, index + 12u);
        }


        current = next;
        segments_remaining -= 1.0;
        divisor = divisor * 16u;
    }
    return result;
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}

fn get_color(index: u32) -> vec3<f32> {
    switch index{
        case 0u: {return vec3<f32>(169.0, 30.0, 30.0) / 255.0;}
        case 1u: {return vec3<f32>(169.0, 53.0, 30.0) / 255.0;}
        case 2u: {return vec3<f32>(169.0, 76.0, 30.0) / 255.0;}
        case 3u: {return vec3<f32>(169.0, 99.0, 30.0) / 255.0;}
        case 4u: {return vec3<f32>(169.0, 123.0, 30.0) / 255.0;}
        case 5u: {return vec3<f32>(169.0, 146.0, 30.0) / 255.0;}
        case 6u: {return vec3<f32>(169.0, 169.0, 30.0) / 255.0;}
        case 7u: {return vec3<f32>(146.0, 169.0, 30.0) / 255.0;}
        case 8u: {return vec3<f32>(123.0, 169.0, 30.0) / 255.0;}
        case 9u: {return vec3<f32>(99.0, 169.0, 30.0) / 255.0;}
        case 10u: {return vec3<f32>(76.0, 169.0, 30.0) / 255.0;}
        case 11u: {return vec3<f32>(53.0, 169.0, 30.0) / 255.0;}
        case 12u: {return vec3<f32>(30.0, 169.0, 30.0) / 255.0;}
        case 13u: {return vec3<f32>(30.0, 169.0, 53.0) / 255.0;}
        case 14u: {return vec3<f32>(30.0, 169.0, 76.0) / 255.0;}
        case 15u: {return vec3<f32>(30.0, 169.0, 99.0) / 255.0;}
        default: {return vec3<f32>(30.0, 169.0, 123.0) / 255.0;}
    }
}


#define_import_path shaders::word_line

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}


fn get_position2(arg1: u32, divisor: u32) -> vec2<f32> {
    let shifted = arg1 / divisor;
    let x: u32 = shifted % 4u;
    let y: u32 = (shifted / 4u) % 4u;

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

fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, line_width: f32, line_proportion: f32) -> f32 {
    let ba = (b - a) * line_proportion;
    let pa = (p - a);
    let k: f32 = saturate(dot(pa, ba) / dot(ba, ba));
    let len = length(pa - (ba * k));
    let len_a = len - line_width;
    return len_a;
}