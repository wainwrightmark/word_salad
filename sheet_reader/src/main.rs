use std::path::PathBuf;

fn main() {
    do_folder("master");
    do_folder("production");

    println!("Finished... Press enter");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn do_folder(name: &str) {
    let master_folder = std::fs::read_dir(name).unwrap();
    let folder_path = format!("output/{name}");
    std::fs::create_dir_all(folder_path.as_str()).unwrap();

    let paths: Vec<_> = master_folder.collect();

    for dir_entry in paths.iter().flatten() {
        if dir_entry.file_type().ok().is_some_and(|x| x.is_file()) {
            if dir_entry
                .path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
            {
                try_write_file(dir_entry.path(), &folder_path);
            }
        }
    }
}

fn try_write_file(path: PathBuf, output_folder: &str) {
    let Ok(text) = std::fs::read_to_string(path) else {
        panic!("Could not read file");
    };

    let mut lines = text.lines();

    let headers = lines.next().expect("Sheet reader file should have at least one line");
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

    for category in categories.into_iter() {
        if !category.name.is_empty() && !category.data.is_empty() {
            let path = format!("{output_folder}/{}.txt", category.name);

            let contents = category.data.join("\n");

            std::fs::write(path.clone(), contents).unwrap();

            println!("Wrote file {path}");
        }
    }
}

struct Category {
    name: String,
    data: Vec<String>,
}
