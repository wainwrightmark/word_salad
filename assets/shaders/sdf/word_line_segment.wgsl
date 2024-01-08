#define_import_path sdf::word_line_segment

fn sdf(p: vec2<f32>, line_width: f32, direction: u32, progress: f32) -> f32 {

    let b = get_relative_point(direction);
    let a = b * -1.;

    return line(p, a, b, line_width, progress);
}

fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, line_width: f32, line_proportion: f32) -> f32 {
    let ba = (b - a) * line_proportion;
    let pa = (p - a);
    let k: f32 = saturate(dot(pa, ba) / dot(ba, ba));
    let len = length(pa - (ba * k));
    let len_a = len - line_width;
    return len_a;
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
