use base64::Engine;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use resvg::usvg::*;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use ws_core::{DesignedLevel, Grid, Tile};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(image_request_handler)).await
}

fn get_parameter<'a>(e: &'a Request, name: &'static str) -> Option<&'a str> {
    e.query_string_parameters_ref().and_then(|x| x.first(name))
}

async fn image_request_handler(
    lambda_event: Request,
) -> Result<Response<Body>, Error> {

    let daily: Option<DesignedLevel> = get_parameter(&lambda_event, "daily")
        .and_then(|x| x.parse().ok())
        .map(|i| level_from_daily_index(i));

    let game: Option<DesignedLevel> = get_parameter(&lambda_event, "game")
        .and_then(|data| base64::engine::general_purpose::URL_SAFE.decode(data).ok())
        .and_then(|data| String::from_utf8(data).ok())
        .map(|data| DesignedLevel::from_tsv_line(&data.trim()).expect("Could not parse level"));

    let width: u32 = get_parameter(&lambda_event, "width")
        .and_then(|x| x.parse().ok())
        .unwrap_or(512);
    let height: u32 = get_parameter(&lambda_event, "height")
        .and_then(|x| x.parse().ok())
        .unwrap_or(512);

    let level = daily.or(game).unwrap_or_else(|| DesignedLevel::unknown());

    let data = draw_image(level, width, height);

    let resp = Response::builder()
        .status(200)
        .header("content-type", "image/png")
        .body(Body::Binary(data))
        .map_err(Box::new)?;
    Ok(resp)
}

fn level_from_daily_index(index: usize) -> DesignedLevel {
    let daily_index = index.saturating_sub(1);

    let level = include_str!("../../../../ws_game/daily.tsv")
        .lines()
        .nth(daily_index as usize)
        .map(|line| DesignedLevel::from_tsv_line(line).expect("Could not parse level"))
        .expect("Could not get level");
    level
}

fn draw_image(level: DesignedLevel, width: u32, height: u32) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();

    let bytes: &'static [u8] = include_bytes!("template_square.svg");

    let mut tree: Tree = Tree::from_data(bytes, &opt).expect("Could not parse template");

    loop_nodes(&mut tree.root, &level.grid);

    let font_data: Vec<u8> =
        include_bytes!("../../../../assets/fonts/Montserrat-Bold.ttf").to_vec();

    let mut font_database: fontdb::Database = fontdb::Database::new();
    font_database.load_font_data(font_data);

    tree.postprocess(PostProcessingSteps::default(), &font_database);

    let x_scale = width as f32 / tree.size.width();
    let y_scale = height as f32 / tree.size.height();
    let scale = x_scale.min(y_scale);
    let tx = (x_scale - scale) * 0.5 * tree.size.width();
    let ty = (y_scale - scale) * 0.5 * tree.size.height();

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).expect("Could not create Pixmap");

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().unwrap()
}

fn loop_nodes(root: &mut Group, grid: &Grid) {
    let mut index = 0;
    while let Some(node) = root.children.get_mut(index) {
        match node {
            Node::Group(ref mut group) => {
                if group.id.starts_with("group") {
                    if let Ok(i) = group.id[5..].parse::<u8>() {
                        if let Some(tile) = Tile::try_from_inner(i) {
                            let char = grid[tile];
                            if char.is_blank() {
                                root.children.remove(index);
                                continue;
                            }
                        }
                    }
                }
                loop_nodes(group, grid);
            }
            Node::Path(_) => {}
            Node::Image(_) => {}
            Node::Text(text) => {
                if text.id.starts_with("text") {
                    if let Ok(i) = text.id[4..].parse::<u8>() {
                        if let Some(tile) = Tile::try_from_inner(i) {
                            let char = grid[tile];
                            text.chunks[0].text = char.to_string();
                        }
                    }
                }
            }
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use ntest::test_case;
    use std::hash::{Hash, Hasher};

    #[test_case(1, 512, 512)]
    #[test_case(2, 512, 512)]
    #[test_case(3, 512, 512)]
    fn test_image(daily: usize, width: u32, height: u32) {
        let level = level_from_daily_index(daily);
        let data = draw_image(level, width, height);
        let len = data.len();
        let path = format!("image_{daily}_{width}x{height}.png");
        std::fs::write(path, data.as_slice()).unwrap();

        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);

        assert!(len < 300000, "Image is too big - {len} bytes");
    }

    #[test_case(
        "Sk5FSU1BTFpSWUlaVFRESwlCZW5uZXQgU2lzdGVycwlKYW5lCUtpdHR5CUxpenppZQlMeWRpYQlNYXJ5",
        512,
        512
    )]
    fn test_game_image(game: &str, width: u32, height: u32) {
        let level = base64::engine::general_purpose::URL_SAFE
            .decode(game)
            .unwrap();
        let level = String::from_utf8(level).unwrap();
        let level = DesignedLevel::from_tsv_line(&level.trim()).expect("Could not parse level");

        let data = draw_image(level, width, height);
        let len = data.len();
        let path = format!("image_{game}_{width}x{height}.png");
        std::fs::write(path, data.as_slice()).unwrap();

        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);

        assert!(len < 300000, "Image is too big - {len} bytes");
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}
