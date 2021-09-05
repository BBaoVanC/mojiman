use std::convert::From;
use serde_derive::{Serialize, Deserialize};
use crate::Emote as mojiman_emote;

#[derive(Deserialize, Serialize)]
pub struct Emote {
    pub name: String,

    #[serde(rename = "type")] // because `type` is a keyword
    pub typ: String,
}

impl From<mojiman_emote> for Emote {
    fn from(emote: mojiman_emote) -> Self {
        Emote {
            name: emote.name,
            typ: ".".to_owned() + &emote.extension,
        }
    }
}

pub fn generate(name: &String, emotes: &Vec<Emote>) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "path": "emotes",
        "emotes": emotes,
    })
}
