#[warn(missing_docs)]
use std::{
    collections::HashMap, env, fs, fs::File, io::prelude::*, io::stdin, path::Path, path::PathBuf,
    process,
};

/// Simply prints `Error: <something>` and exits with provided code.
fn error(s: &str, code: i32) {
    println!("Error: {}", s);
    process::exit(code);
}

/// Takes output path as input.
/// Must be run in the directory of the template.
fn main() {
    let output = PathBuf::from(env::args().collect::<Vec<String>>()[1..].join(" "));

    // retrieving index
    let binding = match fs::read_to_string("./template.txt") {
        Ok(b) => b,
        _ => {
            error("Unable to read template.txt", 2);
            String::new()
        }
    };
    let binding = binding.trim();
    let keys: Vec<&str> = binding.lines().collect();

    let mut index = HashMap::new();

    println!("Input the replacement for the following:\n");

    // add keys to index
    for key in keys {
        let mut input = String::new();

        println!("- {}:", key);

        match stdin().read_line(&mut input) {
            Ok(_) => (),
            _ => error("Unable to read line", 30),
        };

        index.insert(key.to_string(), input.trim().to_string());
    }

    // copy files over
    let working_directory = PathBuf::from("./");
    let mut work = vec![working_directory.clone()];

    while let Some(path) = work.pop() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();

            let (path2, file_type) = (entry.path(), entry.file_type().unwrap());

            // split segment from full path
            let path3 = match path2.strip_prefix(&working_directory) {
                Ok(p) => p,
                _ => {
                    error("Unable to separate directory from full path", 1);
                    Path::new("") // dummy object, should exit before reaching
                }
            };

            // add segment to output
            let path4: PathBuf = [&output, &path3.to_path_buf()].iter().collect();

            // create folder and reappend to work
            if file_type.is_dir() {
                work.push(path2.clone());
                match fs::create_dir(path4) {
                    Ok(_) => (),
                    _ => error("Unable to copy directory", 29),
                }

                continue;
            }

            if file_type.is_file() && entry.file_name() != "template.txt" {
                match fs::read_to_string(path2.clone()) {
                    Ok(mut content) => {
                        // replace keywords
                        for (key, val) in index.iter() {
                            content = content.replace(&format!("%{}%", key), val)
                        }

                        match fs::write(path4, content) {
                            Ok(_) => (),
                            _ => error("Unable to copy file", 29),
                        };
                    }
                    _ => {
                        let content = match fs::read(path2) {
                            Ok(c) => c,
                            _ => {
                                error("Unable to read file", 30);
                                Vec::new() // dummy object, should exit before reaching
                            }
                        };

                        let mut file = match File::create(path4) {
                            Ok(f) => f,
                            _ => {
                                error("Unable to create new file", 29);
                                File::open("./").unwrap() // dummy object, should exit before reaching
                            }
                        };

                        match file.write_all(&content) {
                            Ok(_) => (),
                            _ => error("Unable to write to file", 29),
                        }
                    }
                };
            }
        }
    }

    println!("Template copied successfully!");
}
