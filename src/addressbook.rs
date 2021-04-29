use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub fn load_address_book(config: &config::Config) -> Result<AddressBook, Box<dyn Error>> {
    let addresses_filename = config.get::<String>("addresses_filename")?;
    let contents = fs::read_to_string(addresses_filename)?;
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
