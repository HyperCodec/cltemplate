mod error;

use error::Result;

use std::{path::PathBuf, str::FromStr, sync::{Arc, Mutex}, collections::HashMap, fs};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;
use tracing::{info, debug};
use lazy_static::lazy_static;

lazy_static! {
    static ref ITEMS: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref TASKS: Arc<Mutex<Vec<JoinHandle<Result<()>>>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Parser, Debug)]
#[command(
    name =  "template",
    author = "HyperCodec",
    about = "A small CLI tool for quickly creating and using templates",
)]
struct Cli {
    output_path: PathBuf,

    #[clap(short, long, help = "The path to the template")]
    input_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::from_str("info"))
            .unwrap()
        )
        .init();

    let output = args.output_path;
    let input = match args.input_path {
        Some(v) => v,
        None => std::env::current_dir().expect("Failed to detect current path"),
    };

    // check and read template.txt in local dir
    info!("Parsing manifest");
    let manifestdir = input.join("template.txt");
    let mut items = ITEMS.lock().unwrap();
    let theme = ColorfulTheme::default();
    
    let content = std::fs::read_to_string(manifestdir)?.trim();

    for k in content.lines() {
        debug!("Found key: {k}");
        let v: String = Input::with_theme(&theme)
            .with_prompt(k)
            .interact_text()?;

        items.insert(k.to_string(), v);
    }

    info!("Copying template");
    template_async(input, output, input).await;

    let handles = TASKS.into_inner().unwrap();

    for h in handles.into_iter() { // hmmmmm need to consume but can't
        h.await??;
    }

    Ok(())
}

async fn template_async(path: PathBuf, current_output: PathBuf, og_input: PathBuf) -> Result<()> {
    let subdirs = fs::read_dir(path)?;

    // try create current dir
    fs::create_dir_all(&current_output).ok();

    for dir in subdirs {
        let dir = dir.unwrap();
        let (path, filetype) = (dir.path(), dir.file_type().unwrap());

        info!("Copying {:#?}", path.strip_prefix(&og_input));
        if filetype.is_file() {
            if path.file_name().unwrap() == "template.txt" {
                continue;
            }

            // file stuff
            debug!("Replacing template text");
            let mut content = fs::read_to_string(path)?;
            let items = ITEMS.lock().unwrap();
            
            for (k, v) in items.iter() {
                content = content.replace(&format!("%{}%", k), v);
            }

            fs::write(&current_output, content)?;

            continue;
        }

        // directory
        let mut handles = TASKS.lock().unwrap();

        let new_output = current_output.join(dir.file_name());
        let og_input2 = og_input.clone();
        handles.push(tokio::spawn(async move {
            template_async(path, new_output, og_input2).await
        }));
    }

    Ok(())
}