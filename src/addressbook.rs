use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path;

pub fn load_address_book(config: &config::Config) -> Result<AddressBook, Box<dyn Error>> {
    let addresses_filename = config.get::<String>("addresses_filename")?;
    let addresses_path = path::Path::new(&addresses_filename);
    if !addresses_path.exists() {
        fs::write(addresses_path, DEFAULT_ADDRESSES)?;
    }
    let contents = fs::read_to_string(addresses_path)?;
    let address_book = toml::from_str::<AddressBook>(&contents)?;
    log::trace!("Loaded address book {:?}", address_book);
    Ok(address_book)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressBook {
    pub addresses: Vec<AddressBookEntry>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AddressBookEntry {
    pub label: String,
    pub address: String,
    pub meta: Option<HashMap<String, String>>,
}

pub static DEFAULT_ADDRESSES: &str = r#"
[[addresses]]
label = "Particles"
address = "particlesbbs.dyndns.org:6400"

[[addresses]]
label = "Level29"
address = "bbs.fozztexx.com:23"

[[addresses]]
label = "Xibalba BBS"
address = "xibalba.l33t.codes:44510"

[[addresses]]
label = "The Basement BBS"
address = "basementbbs.ddns.net:9000"

[[addresses]]
label = "Part-Time"
address = "ptbbs.ddns.net:8000"

[[addresses]]
label = "Southern Amis"
address = "southernamis.ddns.net:23"
"#;
