use clap::Parser as ClapParser;
use handlebars::Handlebars;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

/// A program that automatically converts markdown files to HTML files when they are saved
#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify the directory where markdown files used for input are located (optional).
    #[clap(short, long, default_value = ".")]
    input: String,

    /// Specify the output destination for the converted HTML file (optional)
    #[clap(short, long, default_value = ".")]
    output: String,

    /// Specify the file path of the template file
    #[clap(short, long, default_value = "")]
    template: String,
}

fn read_md_file(file_path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let md = fs::read_to_string(file_path.to_str().unwrap())?;
    Ok(md)
}

fn write_html_file(
    file_path: &std::path::Path,
    html: &str,
    template: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(file_path)?;

    if template != "" {
        let template_path = Path::new(template);
        let mut handlebars = Handlebars::new();
        match handlebars.register_template_file("template", template_path) {
            Err(err) => {
                eprintln!("ERROR: {}", err.reason);
                std::process::exit(1);
            }
            _ => handlebars.render_to_write("template", &json!({ "html": html }), &mut file)?,
        }
    } else {
        write!(file, "{}", html)?;
    }
    Ok(())
}

fn markdown_to_html(input_path: &std::path::Path, output_path: &std::path::Path, template: &str) {
    let markdown_input = read_md_file(input_path).unwrap();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    write_html_file(output_path, &html_output, template).unwrap();
}

fn main() -> notify::Result<()> {
    let args = Args::parse();
    let input = &args.input;
    let output = &args.output;
    let template = &args.template;

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;

    let input_dir_path = Path::new(input);
    let output_dir_path = Path::new(output);

    if !input_dir_path.is_dir() || !output_dir_path.is_dir() {
        eprintln!("ERROR: A invalid directory is specified.");
        std::process::exit(1);
    }

    watcher.watch(input_dir_path, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => match event {
                notify::DebouncedEvent::Write(path) => {
                    let input_file_path = Path::new(&path);
                    let md_file_name = input_file_path.file_name().unwrap();
                    match Path::new(md_file_name).extension() {
                        Some(md_exntension) => {
                            if md_exntension != "md" && md_exntension != "html" {
                                eprintln!("ERROR: Only markdown files can be converted.");
                                std::process::exit(1);
                            }

                            if md_exntension == "md" {
                                let html_file_name =
                                    md_file_name.to_str().unwrap().replace(".md", ".html");
                                let output_file_path = output_dir_path.join(html_file_name);

                                markdown_to_html(
                                    input_file_path,
                                    output_file_path.as_path(),
                                    template,
                                );

                                println!("=== Generated HTML ===");
                                println!("Input file path: {:?}", input_file_path);
                                println!(
                                    "Output file path: {:?}",
                                    output_file_path.canonicalize().unwrap()
                                );
                            }
                        }
                        None => {
                            eprintln!("ERROR: Not found extension.");
                            std::process::exit(1);
                        }
                    };
                }
                _ => (),
            },
            Err(err) => println!("FILE WATCH ERROR: {:?}", err),
        };
    }
}
