use std::path::Path;
use std::fs::File;
use clap::{Arg, App};
use serde_derive::{Serialize, Deserialize};
use indicatif::ProgressBar;


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
    // https://github.com/env-logger-rs/env_logger/issues/47#issuecomment-607475404
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let matches = App::new("mojiman")
        .about("Generate a Nitroless repo with correctly sized emotes")
        .version("v0.1.0")
        .after_help("These command-line flags will automatically update mojiman.toml, \
                     which will be created if it doesn't already exist.")
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
        .get_matches();

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


    log::debug!("Source directory is {}", cfg.source_dir);
    log::debug!("Public directory is {}", cfg.public_dir);

    let emotes = mojiman::find_emotes(&cfg.source_dir).expect("Error finding emotes");

    log::info!("Found {} emotes", emotes.len());

    if !std::path::Path::new(&cfg.public_dir).exists() {
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
            let public_size = imagesize::size(public_path).unwrap();
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
        //resize_bar.set_style(ProgressStyle::default_bar()
        //    .progress_chars("=>-"));

        for emote in do_resize {
            let source_path = Path::new(&cfg.source_dir).join(&emote.file_name);
            let public_path = Path::new(&cfg.public_dir).join("emotes").join(&emote.file_name);
            mojiman::resize(&source_path, &public_path, cfg.emote_size)
                .expect(&format!("Error resizing {}", emote.file_name)[..]);
            resize_bar.println(format!("                                     Resized {}", emote.file_name));
            resize_bar.inc(1);
        }

        resize_bar.finish();
    } else {
        log::info!("No emotes need to be resized");
    }

    let index_json = mojiman::make_index_json(&String::from("bobamoji"), &emotes);
    serde_json::to_writer_pretty(&File::create(cfg.public_dir + "/index.json").expect("Error creating index.json"), &index_json)
        .expect("Error writing index.json");
    log::info!("Saved index.json");
}
