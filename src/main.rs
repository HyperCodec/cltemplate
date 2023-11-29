mod error;

use error::Result;

use async_recursion::async_recursion;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::task::JoinHandle;
use tracing::{debug, info, error};
use tracing_subscriber::EnvFilter;

lazy_static! {
    static ref ITEMS: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Parser, Debug)]
#[command(
    name = "template",
    author = "HyperCodec",
    about = "A small commandline tool for quickly creating and using templates"
)]
struct Cli {
    output_path: PathBuf,

    #[clap(short, long, help = "The path to the template")]
    template_path: Option<PathBuf>,
}

#[tokio::main]
#[allow(clippy::await_holding_lock)]
async fn main() -> Result<()> {
    let args = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::from_str("info"))
                .unwrap(),
        )
        .init();

    let output = args.output_path;
    let template = match args.template_path {
        Some(p) => p,
        None => std::env::current_dir().expect("Failed to detect current path"),
    };

    // check and read template.txt in local dir
    info!("Parsing manifest");
    let manifestdir = template.join("template.txt");
    let mut items = ITEMS.lock().unwrap();
    let theme = ColorfulTheme::default();

    let content = match std::fs::read_to_string(manifestdir) {
        Ok(c) => c,
        Err(_) => {
            error!("Failed to read template.txt in current directory");
            std::process::exit(1);
        }
    };
    let content = content.trim();

    for k in content.lines() {
        debug!("Found key: {k}");
        let v: String = Input::with_theme(&theme).with_prompt(k).interact_text()?;

        items.insert(k.to_string(), v);
    }

    drop(items);

    info!("Beginning template copying process");
    let start = Instant::now();
    let tasks = Arc::new(Mutex::new(Vec::new()));
    template_async(template.clone(), output, template, tasks.clone()).await?;

    let lock = Arc::try_unwrap(tasks).unwrap();
    let handles = lock.into_inner().unwrap();

    for h in handles.into_iter() {
        h.await??;
    }

    info!(
        "Template copied successfully (elapsed: {:#?})",
        start.elapsed()
    );

    Ok(())
}

#[async_recursion]
async fn template_async(
    path: PathBuf,
    current_output: PathBuf,
    template_path: PathBuf,
    tasks: Arc<Mutex<Vec<JoinHandle<Result<()>>>>>,
) -> Result<()> {
    debug!("Worker started");

    let subdirs = fs::read_dir(path)?;

    // try create current dir
    fs::create_dir_all(&current_output).ok();

    for dir in subdirs {
        let dir = dir.unwrap();
        let (path, filetype) = (dir.path(), dir.file_type().unwrap());

        if filetype.is_file() {
            let fname = path.file_name().unwrap();
            if fname == "template.txt" {
                continue;
            }

            info!(
                "Copying {}",
                path.strip_prefix(&template_path).unwrap().display()
            );

            // file stuff
            debug!("Replacing template text");
            let mut content = fs::read_to_string(&path)?;
            let items = ITEMS.lock().unwrap();

            for (k, v) in items.iter() {
                content = content.replace(&format!("%{}%", k), v);
            }

            fs::write(&current_output.join(fname), content)?;

            debug!("Finished replacing template text");
            continue;
        }

        // directory
        info!(
            "Copying {}",
            path.strip_prefix(&template_path).unwrap().display()
        );
        let tasks2 = tasks.clone();
        let mut handles = tasks2.lock().unwrap();

        let new_output = current_output.join(dir.file_name());
        let template_path2 = template_path.clone();
        let tasks2 = tasks.clone();
        handles.push(tokio::spawn(async move {
            template_async(path, new_output, template_path2, tasks2).await
        }));
    }

    debug!("Worker finished");

    Ok(())
}
