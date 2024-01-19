use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::http::{HeaderMap, HeaderValue};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::usvg::*;
use ws_core::DesignedLevel;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let f = service_fn(image_request_handler);
    lambda_runtime::run(f).await?;
    Ok(())
}

fn get_parameter<'a>(
    e: &'a LambdaEvent<ApiGatewayProxyRequest>,
    name: &'static str,
) -> Option<&'a str> {
    e.payload
        .query_string_parameters
        .iter()
        .filter(|x| x.0.eq_ignore_ascii_case(name))
        .map(|x| x.1)
        .next()
}

async fn image_request_handler(
    lambda_event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));

    let daily: u32 = get_parameter(&lambda_event, "daily")
        .and_then(|x| x.parse().ok())
        .expect("Could not parse daily index");
    let width: u32 = get_parameter(&lambda_event, "width")
        .and_then(|x| x.parse().ok())
        .unwrap_or(512);
    let height: u32 = get_parameter(&lambda_event, "height")
        .and_then(|x| x.parse().ok())
        .unwrap_or(512);

    let data = draw_image(daily, width, height);

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Binary(data)),
        is_base64_encoded: true,
    };

    Ok(resp)
}

fn draw_image(daily_index: u32, width: u32, height: u32) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();

    let bytes: &'static [u8] = include_bytes!("template_square.svg");

    let mut tree = Tree::from_data(bytes, &opt).expect("Could not parse template");
    let daily_index = daily_index.saturating_sub(1);

    let level = include_str!("../../../../ws_game/daily.tsv")
        .lines()
        .nth(daily_index as usize)
        .map(|line| DesignedLevel::from_tsv_line(line).expect("Could not parse level"))
        .expect("Could not get level");

    for (index, character) in level.grid.into_iter().enumerate() {
        let text_id = format!("text{index}",);
        let text_node = tree
            .node_by_id(text_id.as_str())
            .expect("Could not find text node by id");

        if character.is_blank() {
            text_node.detach();
            let square_id = format!("square{index}");
            let square_node = tree
                .node_by_id(&square_id.as_str())
                .expect("Could not find square node by id");

            square_node.detach();
        } else {
            if let NodeKind::Text(ref mut text) = *text_node.borrow_mut() {
                text.chunks[0].text = character.to_string();
            } else {
                panic!("Node was not a text node")
            };
        }
        //todo blank characters
    }

    let font_data: Vec<u8> = include_bytes!("../../../../assets/fonts/Montserrat-Bold.ttf").to_vec();

    let mut font_database: fontdb::Database = fontdb::Database::new();
    font_database.load_font_data(font_data);

    tree.convert_text(&font_database);

    let x_scale = width as f32 / tree.size.width();
    let y_scale = height as f32 / tree.size.height();
    let scale = x_scale.min(y_scale);
    let tx = (x_scale - scale) * 0.5 * tree.size.width();
    let ty = (y_scale - scale) * 0.5 * tree.size.height();

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).expect("Could not create Pixmap");
    resvg::Tree::render(
        &resvg::Tree::from_usvg(&tree),
        transform,
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::*;
    use ntest::test_case;
    use std::hash::{Hash, Hasher};

    #[test_case(1, 512, 512)]
    #[test_case(2, 512, 512)]
    #[test_case(3, 512, 512)]
    fn test_image(daily: u32, width: u32, height: u32) {
        let data = draw_image(daily, width, height);
        let len = data.len();
        let path = format!("image_{daily}_{width}x{height}.png");
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
