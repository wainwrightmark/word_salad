#define_import_path sdf::word_line_segment

fn sdf(p: vec2<f32>, line_width: f32, direction: u32, progress: f32, color: vec4<f32>) -> f32 {
    if direction == 0u{
        return sd_circle(p, line_width * progress * 2.0);
    }


    let b = get_relative_point(direction) * progress;
    let k: f32 = saturate(dot(p, b) / dot(b, b));
    let len = length(p - (b * k));
    let len_a = len - line_width;
    return len_a;
}

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn get_relative_point(index: u32) -> vec2<f32> {
    switch index{
        case 0u: {return vec2<f32>(0.0, 0.0);}
        case 1u: {return vec2<f32>(0.0, 0.5);}
        case 2u: {return vec2<f32>(0.5, 0.5);}
        case 3u: {return vec2<f32>(0.5, 0.0);}
        case 4u: {return vec2<f32>(0.5, -0.5);}
        case 5u: {return vec2<f32>(0.0, -0.5);}
        case 6u: {return vec2<f32>(-0.5, -0.5);}
        case 7u: {return vec2<f32>(-0.5, 0.0);}
        default: {return vec2<f32>(-0.5, 0.5);}
    }
}
