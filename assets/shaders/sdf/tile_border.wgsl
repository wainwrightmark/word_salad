#define_import_path shaders::tile_border

#import smud

// params.x is rounding ratio for all four corners
// params.y is the border ratio
fn sdf(p: vec2<f32>, params: vec4<f32>) -> f32 {

    let tile = smud::sd_rounded_box(p, vec2<f32>(1.0, 1.0), vec4<f32>(params.x, params.x, params.x, params.x) );

    let border = abs(tile) - params.y;

    return border;
}