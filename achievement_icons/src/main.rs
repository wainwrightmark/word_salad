use resvg::usvg::*;
fn main() {

    for c in 'A'..='Z'{
        write_image_file(c.to_string().as_str(), 512, 512);
    }
    
}


fn write_image_file(new_text: &str, width: u32, height: u32){
    let data = draw_image(new_text, width, height);        
        let path = format!("achievement_{new_text}_{width}x{height}.png");
        std::fs::write(path, data.as_slice()).unwrap();
}


fn draw_image(new_text: &str, width: u32, height: u32) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();

    let bytes: &'static [u8] = include_bytes!("achievement.svg");

    let mut tree = Tree::from_data(bytes, &opt).expect("Could not parse template");
    

    let text_node = tree
            .node_by_id("text1")
            .expect("Could not find text node by id");

    if let NodeKind::Text(ref mut text) = *text_node.borrow_mut() {
        text.chunks[0].text = new_text.to_string();
    } else {
        panic!("Node was not a text node")
    };

    let font_data: Vec<u8> =
        include_bytes!("../../assets/fonts/Montserrat-Bold.ttf").to_vec();

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