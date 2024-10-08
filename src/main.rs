mod error;

use error::Result;

use async_recursion::async_recursion;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{debug, error, info};
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
    debug!("Parsing manifest");
    let manifestdir = template.join("template.txt");
    let mut items = ITEMS.lock().unwrap();
    let theme = ColorfulTheme::default();

    let content = match std::fs::read_to_string(manifestdir) {
        Ok(c) => c,
        Err(_) => {
            error!("Failed to read template.txt in template directory");
            std::process::exit(1);
        }
    };
    let content = content.trim();

    for k in content.lines() {
        debug!("Found key: {k}");
        let v: String = Input::with_theme(&theme).with_prompt(k).interact_text()?;

        items.insert(format!(r#"\{{%\s*{}\s*%\}}"#, k), v);
    }

    drop(items);

    info!("Beginning template copying process");
    let progress = Arc::new(MultiProgress::new());
    let start = Instant::now();
    template_async(template.clone(), output, template, progress.clone()).await?;

    println!();
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
    multiprogress: Arc<MultiProgress>,
) -> Result<()> {
    debug!("Worker started");

    let subdirs = fs::read_dir(&path)?;
    let mut handles = Vec::new();

    // init progress spinner
    let pb = multiprogress.add(ProgressBar::new_spinner());
    if path == template_path {
        pb.set_message("Copy Template");
    } else {
        pb.set_message(
            path.strip_prefix(&template_path)
                .unwrap()
                .display()
                .to_string(),
        );
    }
    pb.enable_steady_tick(Duration::from_millis(128));
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:} {msg}")
            .unwrap()
            .tick_strings(&[
                "⠋", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓",
                "⠋", "✓", // TODO separate finished color
            ]),
    );

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

            /*
            info!(
                "Copying {}",
                path.strip_prefix(&template_path).unwrap().display()
            );
            */

            // file stuff
            let content = fs::read(&path)?;
            let newf = current_output.join(fname);

            if !content.is_ascii() {
                debug!("Non-ascii file detected, copying directly instead of replacing text");
                fs::write(newf, content)?;
            } else {
                debug!("Replacing template text");
                let mut content = String::from_utf8(content).unwrap();
                let items = ITEMS.lock().unwrap();

                for (k, v) in items.iter() {
                    let re = Regex::new(k).unwrap(); // should prob keep a cache of regexes but they dont work in hashmaps
                    content = re.replace_all(&content, v).to_string();
                }

                fs::write(newf, content)?;
                debug!("Finished replacing template text");
            }

            continue;
        }

        // directory
        /*
        info!(
            "Copying {}",
            path.strip_prefix(&template_path).unwrap().display()
        );
        */

        let new_output = current_output.join(dir.file_name());
        let template_path2 = template_path.clone();
        let multiprogress2 = multiprogress.clone();
        handles.push(tokio::spawn(async move {
            template_async(path, new_output, template_path2, multiprogress2).await
        }));
    }

    for h in handles {
        h.await??;
    }

    pb.finish();

    debug!("Worker finished");

    Ok(())
}
