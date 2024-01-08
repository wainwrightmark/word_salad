fn main() {
    let Ok(text) = std::fs::read_to_string("sheet1.txt") else {
        println!("There must be a file called `sheet1.txt`");
        std::io::stdin().read_line(&mut String::new()).unwrap();
        return;
    };

    let mut lines = text.lines();

    let headers = lines.next().unwrap();
    let mut categories: Vec<Category> = vec![];
    for header in headers.split('\t') {
        categories.push(Category {
            name: header.trim().to_string(),
            data: vec![],
        });
    }

    for line in lines {
        let values = line.split('\t');

        values.zip(categories.iter_mut()).for_each(|(v, c)| {
            let v = v.trim();
            if !v.is_empty() {
                c.data.push(v.to_string());
            }
        });
    }
    std::fs::create_dir_all("sheet_reader_data").unwrap();

    for category in categories.into_iter() {
        let path = format!("sheet_reader_data/{}.txt", category.name);

        let contents = category.data.join("\n");

        std::fs::write(path.clone(), contents).unwrap();

        println!("{path}");
    }

    println!("Finished... Press enter");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

struct Category {
    name: String,
    data: Vec<String>,
}
