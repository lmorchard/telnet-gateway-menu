#[macro_use]
extern crate lazy_static;
extern crate config;
extern crate libtelnet_rs;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process;
use std::str;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use config::Config;
use libtelnet_rs::events::TelnetEvents;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

fn main() {
    env_logger::init();
    log::info!("starting up");

    if let Err(err) = load_config() {
        log::error!("Failed to load config: {:?}", err);
        process::exit(1);
    }

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    loop {
        for stream in listener.incoming() {
            thread::spawn(|| {
                if let Err(err) = handle_connection(stream.unwrap()) {
                    log::error!("connection error {:?}", err);
                }
            });
        }
    }
}

fn load_config() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = CONFIG.write()?;
    config
        .set_default("addresses_filename", "addresses.toml")?
        .merge(config::File::with_name("Settings"))?
        .merge(config::Environment::with_prefix("APP"))?;
    Ok(())
}

fn handle_connection(mut local_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    log::info!("incoming connection {:?}", local_stream.peer_addr()?);
    write!(local_stream, "Hello, {:?}\r\n", local_stream.peer_addr()?)?;
    loop {
        let address_book = load_address_book()?;
        let entry = run_menu(&mut local_stream, &address_book)?;
        log::info!("outgoing connection {:?}", entry.address);
        write!(
            local_stream,
            "\r\nConnecting to {} - {}\r\n",
            entry.label, entry.address
        )?;
        match run_telnet_relay(&mut local_stream, &entry.address)? {
            RelayEnd::RemoteErr(err) => {
                log::error!("remote connection error - {:?}", err);
            }
            RelayEnd::LocalErr(err) => {
                log::error!("local connection error - {:?}", err);
                return Err(Box::new(err));
            }
        };
        log::info!("outgoing disconnect {:?}", entry.address);
        write!(
            local_stream,
            "\r\nDisconnected from {} - {}\r\n",
            entry.label, entry.address
        )?;

        log::info!("incoming disconnect {:?}", local_stream.peer_addr()?);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AddressBook {
    addresses: Vec<AddressBookEntry>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AddressBookEntry {
    label: String,
    address: String,
    meta: Option<HashMap<String, String>>,
}

fn load_address_book() -> Result<AddressBook, Box<dyn Error>> {
    let addresses_filename = CONFIG.read()?.get::<String>("addresses_filename")?;
    let contents = fs::read_to_string(addresses_filename)?;
    let address_book = toml::from_str::<AddressBook>(&contents)?;
    log::trace!("Loaded address book {:?}", address_book);
    Ok(address_book)
}

#[derive(Debug)]
struct DisconnectFromMenuError {}
impl fmt::Display for DisconnectFromMenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl Error for DisconnectFromMenuError {}

fn run_menu<'a>(
    local_stream: &mut TcpStream,
    address_book: &'a AddressBook,
) -> Result<&'a AddressBookEntry, Box<dyn Error>> {
    local_stream.set_read_timeout(None)?;

    loop {
        write!(local_stream, "\r\nAddress book:\r\n")?;
        write!(local_stream, "{:>3}: Logoff\r\n", 0)?;

        let mut idx = 0;
        let addresses = &address_book.addresses;
        while idx < addresses.len() {
            if let Some(entry) = addresses.get(idx) {
                write!(
                    local_stream,
                    "{:>3}: {} - {}\r\n",
                    idx + 1,
                    entry.label,
                    entry.address
                )?;
            }
            idx += 1;
        }

        let input = read_line_from_stream(local_stream, "> ")?;
        if let Ok(menu_choice) = input.trim().parse::<usize>() {
            if menu_choice == 0 {
                write!(local_stream, "\r\nGoodbye!\r\n")?;
                return Err(Box::new(DisconnectFromMenuError {}));
            }
            if let Some(choice) = address_book.addresses.get(menu_choice - 1) {
                return Ok(choice);
            }
        }
        write!(
            local_stream,
            "\r\nInvalid choice - {:?} - please try again.\r\n",
            input
        )?;
    }
}

fn read_line_from_stream(
    local_stream: &mut TcpStream,
    prompt: &str,
) -> Result<String, std::io::Error> {
    let mut buffer = [0; 1024];
    let mut input = String::new();

    local_stream.write_all(prompt.as_bytes())?;

    let mut parser = libtelnet_rs::Parser::new();

    loop {
        // Try reading a chunk of input
        let len = match local_stream.read(&mut buffer) {
            Err(err) => Err(err),
            Ok(len) if len == 0 => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "read zero (disconnected)",
            )),
            Ok(len) => Ok(len),
        }?;

        // TODO: work out when ont to echo per telnet protocol
        local_stream.write_all(&buffer[0..len])?;

        // TODO: support backspace!
        let telnet_events = parser.receive(&buffer[0..len]);
        for ev in telnet_events {
            match ev {
                TelnetEvents::IAC(iac) => log::info!("IAC {:?}", iac.into_bytes()),
                TelnetEvents::Negotiation(neg) => log::info!("Negotiation {:?}", neg.into_bytes()),
                TelnetEvents::Subnegotiation(subneg) => {
                    log::info!("Subnegotiation {:?}", subneg.into_bytes())
                }
                TelnetEvents::DataReceive(data) => log::info!("DataReceive {:?}", data),
                TelnetEvents::DataSend(data) => log::info!("DataSend {:?}", data),
                TelnetEvents::DecompressImmediate(data) => {
                    log::info!("DecompressImmediate {:?}", data)
                }
            }
        }

        // Collect this chunk of input until it contains a return
        if let Ok(data) = str::from_utf8(&buffer[0..len]) {
            input.push_str(data);
            if let Some(pos) = input.find('\r') {
                input.truncate(pos);
                return Ok(input);
            }
        }
    }
}

#[derive(Debug)]
enum RelayEnd {
    LocalErr(std::io::Error),
    RemoteErr(std::io::Error),
}

fn run_telnet_relay(
    local_stream: &mut TcpStream,
    address: &str,
) -> Result<RelayEnd, std::io::Error> {
    let mut remote_stream = TcpStream::connect(address)?;

    let timeout = Some(Duration::from_millis(10));
    local_stream.set_read_timeout(timeout)?;
    remote_stream.set_read_timeout(timeout)?;

    let mut buffer = [0; 1024];
    loop {
        if let Err(err) = relay_sockets(&mut buffer, local_stream, &mut remote_stream) {
            return Ok(RelayEnd::LocalErr(err));
        }
        if let Err(err) = relay_sockets(&mut buffer, &mut remote_stream, local_stream) {
            return Ok(RelayEnd::RemoteErr(err));
        }
    }
}

fn relay_sockets(
    buffer: &mut [u8],
    from_stream: &mut TcpStream,
    to_stream: &mut TcpStream,
) -> Result<(), std::io::Error> {
    match from_stream.read(buffer) {
        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
        Ok(len) if len == 0 => Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "read zero (disconnected)",
        )),
        Err(err) => Err(err),
        Ok(len) => match to_stream.write(&buffer[0..len]) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        },
    }
}
