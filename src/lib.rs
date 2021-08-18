use std::fs;
use std::error::Error;
use std::path::Path;
use serde_derive::Serialize;
use std::process::Command;


pub mod json;


#[derive(Debug, PartialEq, Serialize)]
pub struct Emote {
    pub name: String,

    #[serde(rename = "type")]
    pub extension: String,

    #[serde(skip_serializing)]
    pub file_name: String,
}

pub fn find_emotes<T: AsRef<Path>>(dir: T) -> Result<Vec<Emote>, Box<dyn Error>> {
    let mut emotes = Vec::new();

    for f in fs::read_dir(dir)? {
        let path = f?.path();
        let file_name = match path.file_name() {
            Some(file_name) => String::from(file_name.to_str().unwrap()),
            None => continue,
        };
        let name = match path.file_stem() {
            Some(name) => String::from(name.to_str().unwrap()),
            None => continue,
        };
        let extension = match path.extension() {
            Some(extension) => String::from(extension.to_str().unwrap()),
            None => continue,
        };
        if ["png", "jpg", "gif"].contains(&&extension[..]) {
            emotes.push(Emote {
                name,
                extension,
                file_name,
            });
        }
    }

    Ok(emotes)
}

pub fn is_newer_than<T: AsRef<Path>>(one: T, two: T) -> Result<bool, Box<dyn Error>> {
    let one_modified = fs::metadata(one)?.modified()?;
    let two_modified = fs::metadata(two)?.modified()?;
    Ok(one_modified > two_modified)
}

pub fn resize<T: AsRef<Path>>(source_path: T, output_path: T, size: u32) -> Result<(), Box<dyn Error>> {

    if source_path.as_ref().extension().unwrap().to_str().unwrap() == "gif" {
        let source_path_str = source_path.as_ref().to_str().unwrap();
        let output_path_str = output_path.as_ref().to_str().unwrap();
        Command::new("magick")
            .args(&["convert", source_path_str, "-resize", format!("{}", size).as_str(), output_path_str])
            .status().expect(format!("Failed to execute imagemagick on {}", source_path_str).as_str());

        Ok(())

    } else {
        let img = image::open(source_path)?;
        img.resize(size, size, image::imageops::Lanczos3).save(output_path)?;
        Ok(())
    }
}
