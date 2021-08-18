use std::io::Write;
use std::path::Path;
use std::fs::File;
use clap::{Arg, App, SubCommand, crate_name, crate_authors, crate_version, crate_description};
use serde_derive::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressStyle};
use colored::Colorize;

use mojiman::json::indexjson;

#[derive(Serialize, Deserialize)]
struct Config {
    repo_name: String,
    emote_size: u32,
    source_dir: String,
    public_dir: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self { Self {
        repo_name: String::from("Nitroless Repo"),
        emote_size: 48,
        source_dir: String::from("emotes"),
        public_dir: String::from("public"),
    } }
}


fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .after_help("These command-line flags will automatically update mojiman.toml, \
                     which will be created if it doesn't already exist. \
                     \n\n\
                     If no subcommand is specified, then `generate` will run, \
                     followed by `clean`.")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enable debug output"))
        .arg(Arg::with_name("repo_name")
            .short("n")
            .long("name")
            .takes_value(true)
            .help("Name of the Nitroless repo"))
        .arg(Arg::with_name("source_dir")
            .short("i")
            .long("source")
            .takes_value(true)
            .help("Directory containing the source emotes"))
        .arg(Arg::with_name("public_dir")
            .short("o")
            .long("out")
            .takes_value(true)
            .help("Directory to output public files to"))
        .arg(Arg::with_name("emote_size")
            .short("s")
            .long("size")
            .takes_value(true)
            .help("Resolution to resize emotes to"))
        .subcommand(SubCommand::with_name("generate")
            .about("Generate the repo.")
            .visible_aliases(&["gen", "g"]))
        .subcommand(SubCommand::with_name("clean")
            .about("Remove emotes from the public directory that have been removed from the source directory."))
        .get_matches();

    let loglevel: log::LevelFilter;
    if matches.is_present("verbose") {
        loglevel = log::LevelFilter::Debug;
    } else {
        loglevel = log::LevelFilter::Info;
    }

    env_logger::builder()
        .format(|buf, record| {
            let level_str = match record.level() {
                log::Level::Error => "::".red(),
                log::Level::Warn => "::".yellow(),
                log::Level::Info => "::".bright_blue(),
                log::Level::Debug => "::".bright_black(),
                log::Level::Trace => "::".black(),
            };
            writeln!(buf, "{} {}", level_str, record.args())
        })
        //.format_timestamp(None)
        .filter_level(loglevel)
        .init();

    let mut cfg: Config = confy::load_path("mojiman.toml").expect("Error loading config");
    log::debug!("Loaded mojiman.toml");


    if let Some(name) = matches.value_of("repo_name") {
        cfg.repo_name = String::from(name);
    }

    if let Some(dir) = matches.value_of("source_dir") {
        cfg.source_dir = String::from(dir);
    }

    if let Some(dir) = matches.value_of("public_dir") {
        cfg.public_dir = String::from(dir);
    }

    if let Some(size) = matches.value_of("emote_size") {
        cfg.emote_size = size.parse::<u32>().expect("emote_size must be a number");
    }

    confy::store_path("mojiman.toml", &cfg).expect("Error saving updated config");
    log::debug!("Updated mojiman.toml with current settings");


    if let Some(ref _matches) = matches.subcommand_matches("clean") {
        clean_repo(&cfg);
    } else if let Some(ref _matches) = matches.subcommand_matches("generate") {
        generate_repo(&cfg);
    } else {
        log::info!("{} Running `mojiman generate`", "==>".green());
        generate_repo(&cfg);

        log::info!("{} Running `mojiman clean`", "==>".green());
        clean_repo(&cfg);
    }
}

fn clean_repo(cfg: &Config) {
    if !Path::new(&cfg.public_dir).exists() {
        log::error!("There is no public repo to clean up, run `mojiman generate` to create it.");
        return;
    }

    let public_emotes_path = Path::new(&cfg.public_dir).join("emotes");
    let public_emotes = mojiman::find_emotes(&public_emotes_path).expect("Error finding public emotes");
    let source_emotes = mojiman::find_emotes(&Path::new(&cfg.source_dir)).expect("Error finding source emotes");

    let mut orphans = Vec::new();
    for emote in &public_emotes {
        if !source_emotes.contains(emote) {
            orphans.push(emote);
        }
    }

    if orphans.len() > 0 {
        log::info!("Removing {} orphaned emotes from the public directory", orphans.len());
        for emote in &orphans {
            let emote_path = public_emotes_path.join(&emote.file_name);
            log::debug!("Removing {}", emote_path.to_str().unwrap());
            std::fs::remove_file(&emote_path).expect(&format!("Error removing {}", emote_path.to_str().unwrap())[..]);
        }
    } else {
        log::info!("There are no orphaned emotes in the public directory.");
    }
}

fn generate_repo(cfg: &Config) {
    let emotes = mojiman::find_emotes(&Path::new(&cfg.source_dir)).expect("Error finding emotes");

    log::info!("Found {} emotes", emotes.len());

    if !Path::new(&cfg.public_dir).exists() {
        std::fs::create_dir_all(Path::new(&cfg.public_dir).join("emotes"))
            .expect(&format!("Error creating {}/emotes", &cfg.public_dir)[..]);
    }

    let mut do_resize = Vec::new();

    for emote in &emotes {
        let source_path = Path::new(&cfg.source_dir).join(&emote.file_name);
        let public_path = Path::new(&cfg.public_dir).join("emotes").join(&emote.file_name);

        if public_path.is_file() {
            let is_newer = mojiman::is_newer_than(&source_path, &public_path)
                .expect(&format!("Error comparing modification date of {}", emote.file_name)[..]);
            let public_size = match imagesize::size(&public_path) {
                Ok(size) => size,
                Err(err) => {
                    log::warn!("Failed to get the size of {}, it will be regenerated", emote.file_name);
                    log::debug!("Error that caused the failure: {:?}", err);
                    do_resize.push(emote);
                    continue;
                }
            };
            let is_wrongly_sized = !((public_size.width == cfg.emote_size as usize) || (public_size.height == cfg.emote_size as usize));

            if is_newer || is_wrongly_sized {
                log::debug!("Resize {}", emote.file_name);
                do_resize.push(emote);
            } else {
                log::debug!("Keep {}", emote.file_name);
            }
        } else {
            log::debug!("Resize {}", emote.file_name);
            do_resize.push(emote);
        }
    }

    if do_resize.len() > 0 {
        log::info!("Resizing {} emotes", do_resize.len());
        let resize_bar = ProgressBar::new(do_resize.len() as u64);
        resize_bar.set_style(ProgressStyle::default_bar()
            .template("[{wide_bar}] {pos}/{len} {msg}")
            .progress_chars("=> "));

        for emote in do_resize {
            let source_path = Path::new(&cfg.source_dir).join(&emote.file_name);
            let public_path = Path::new(&cfg.public_dir).join("emotes").join(&emote.file_name);

            resize_bar.set_message(String::from(&emote.file_name));
            resize_bar.println(format!("{} Resizing {}", "::".blue(), emote.file_name));
            resize_bar.inc(1);

            mojiman::resize(&source_path, &public_path, cfg.emote_size)
                .expect(&format!("Error resizing {}", emote.file_name)[..]);
        }

        resize_bar.finish_with_message("");
    } else {
        log::info!("No emotes need to be resized");
    }

    let mut index_json_emotes: Vec<indexjson::Emote> = Vec::new();
    for e in emotes {
        index_json_emotes.push(e.into());
    }
    let index_json = indexjson::generate(&String::from("bobamoji"), &index_json_emotes);
    serde_json::to_writer_pretty(&File::create(String::from(&cfg.public_dir) + "/index.json").expect("Error creating index.json"), &index_json)
        .expect("Error writing index.json");
    log::info!("Saved index.json");
}
