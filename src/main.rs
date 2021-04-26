use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();
    log::info!("starting up");

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    loop {
        for stream in listener.incoming() {
            thread::spawn(|| {
                let address_book = read_address_book();
                if let Err(err) = handle_connection(stream.unwrap(), &address_book) {
                    log::error!("connection error {:?}", err);
                }
            });
        }
    }
}

fn read_address_book<'a>() -> Vec<(&'a str, &'a str)> {
    let address_book: Vec<(&str, &str)> = vec![
        ("Level29", "bbs.fozztexx.com:23"),
        ("Particles", "particlesbbs.dyndns.org:6400"),
    ];
    return address_book;
}

fn handle_connection(
    mut local_stream: TcpStream,
    address_book: &Vec<(&str, &str)>,
) -> Result<(), std::io::Error> {
    log::info!("incoming connection {:?}", local_stream.peer_addr()?);

    write!(local_stream, "Hello, {:?}\r\n", local_stream.peer_addr()?)?;

    loop {
        if let Some((label, address)) = run_menu(&mut local_stream, address_book)? {
            log::info!("outgoing connection {:?}", address);
            write!(
                local_stream,
                "\r\nConnecting to {} - {}\r\n",
                label, address
            )?;
            match run_telnet_relay(&mut local_stream, address)? {
                RelayEnd::RemoteErr(err) => {
                    log::error!("remote connection error - {:?}", err);
                }
                RelayEnd::LocalErr(err) => {
                    log::error!("local connection error - {:?}", err);
                    return Err(err);
                }
            };
            log::info!("outgoing disconnect {:?}", address);
            write!(
                local_stream,
                "\r\nDisconnected from {} - {}\r\n",
                label, address
            )?;
        }

        log::info!("incoming disconnect {:?}", local_stream.peer_addr()?);
    }
}

fn run_menu<'a>(
    local_stream: &mut TcpStream,
    address_book: &'a Vec<(&'a str, &'a str)>,
) -> Result<Option<&'a (&'a str, &'a str)>, std::io::Error> {
    let mut buffer = [0; 1024];
    local_stream.set_read_timeout(None)?;
    loop {
        write!(local_stream, "\r\nAddress book:\r\n")?;
        write!(local_stream, "{:>3}: {}\r\n", 0, "Logoff")?;
        let mut idx = 0;
        while idx < address_book.len() {
            if let Some((label, address)) = address_book.get(idx) {
                write!(local_stream, "{:>3}: {} - {}\r\n", idx + 1, label, address)?;
            }
            idx += 1;
        }
        write!(local_stream, "> ")?;

        let len = match local_stream.read(&mut buffer) {
            Err(err) => Err(err),
            Ok(len) if len == 0 => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "read zero (disconnected)",
            )),
            Ok(len) => Ok(len),
        }?;

        if let Ok(input) = str::from_utf8(&buffer[0..len]) {
            if let Ok(menu_choice) = input.trim().parse::<usize>() {
                if menu_choice == 0 {
                    write!(local_stream, "\r\nGoodbye!\r\n")?;
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        "logoff",
                    ));
                }
                if let Some(choice) = address_book.get(menu_choice - 1) {
                    return Ok(Some(choice));
                }
            }
        }

        write!(local_stream, "\r\nInvalid choice, please try again.\r\n")?;
    }
}

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
