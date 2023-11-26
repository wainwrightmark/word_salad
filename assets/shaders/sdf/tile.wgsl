#define_import_path shaders::tile

#import smud

// params.x is rounding ratio for all four corners
fn sdf(p: vec2<f32>, params: vec4<f32>) -> f32 {

    return smud::sd_rounded_box(p, vec2<f32>(1.0, 1.0), vec4<f32>(params.x, params.x, params.x, params.x) );
}