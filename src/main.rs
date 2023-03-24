#[warn(missing_docs)]

use std::{collections::HashMap, env, fs, io::stdin, path::PathBuf};

/// Takes output path as input.
/// Must be run in the directory of the template.
fn main() {
    let output = PathBuf::from(env::args().collect::<Vec<String>>()[1..].join(" "));

    // retrieving index
    let binding = fs::read_to_string("./template.txt").expect("Unable to read template.txt");
    let binding = binding.trim();
    let keys: Vec<&str> = binding.lines().collect();

    let mut index = HashMap::new();

    println!("Input the replacement for the following:\n");

    // add keys to index
    for key in keys {
        let mut input = String::new();

        println!("- {}:", key);

        stdin().read_line(&mut input).expect("Unable to read line");

        index.insert(key.to_string(), input.trim().to_string());
    }

    // copy files over
    let working_directory = PathBuf::from("./");
    let mut work = vec![working_directory.clone()];

    while let Some(path) = work.pop() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();

            let (path2, file_type) = (entry.path(), entry.file_type().unwrap());

            // segment representing
            let path3 = path2
                .strip_prefix(&working_directory)
                .expect("Unable to separate directory from full path");
            let path4: PathBuf = [&output, &path3.to_path_buf()].iter().collect();

            // create folder and reappend to work
            if file_type.is_dir() {
                work.push(path2.clone());
                fs::create_dir(path4).expect("Unable to copy dir");

                continue;
            }

            if file_type.is_file() && entry.file_name() != "template.txt" {
                let mut content = fs::read_to_string(path2).expect("Unable to read file");

                // replace keywords
                for (key, val) in index.iter() {
                    content = content.replace(&format!("%{}%", key), val)
                }

                fs::write(path4, content).expect("Unable to copy file");
            }
        }
    }

    println!("Template copied successfully!");
}
