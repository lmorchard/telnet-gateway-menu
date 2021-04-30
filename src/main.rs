extern crate clap;
extern crate config;
extern crate libtelnet_rs;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches,
};
use config::Config;

mod addressbook;
mod subcommand_serve;

fn main() {
    let app = setup_app();
    let app_m = app.get_matches();
    let config = setup_config(&app_m).unwrap();
    setup_logging(&config).unwrap();
    match app_m.subcommand() {
        Some((subcommand_serve::NAME, sub_m)) => subcommand_serve::execute(&sub_m, &config),
        _ => Ok(()),
    }
    .unwrap()
}

// TODO: no idea what I'm going with these lifetimes
fn setup_app() -> App<'static> {
    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("debug")
                .short('d')
                .about("Turn debugging information on"),
        )
        .subcommand(subcommand_serve::app())
}

// TODO: this function seems awkward
fn setup_config(app_m: &ArgMatches) -> Result<config::Config, Box<dyn std::error::Error>> {
    // TODO: just chaining straight from Config::default() raises complaints of temporary references, why?
    let mut config_default = Config::default();
    let config = config_default
        .set_default("log_level", "info")?
        .set_default("addresses_filename", "addresses.toml")?
        .set_default("host", "0.0.0.0")?
        .set_default("port", "7878")?
        .merge(config::File::with_name("config").required(false))?
        .merge(config::Environment::with_prefix("APP"))?;
    if app_m.is_present("debug") {
        config.set_default("debug", true)?;
        config.set("log_level", "debug")?;
    }
    // TODO: does this really need to be cloned? anything else complains about lifetimes
    Ok(config.clone())
}

fn setup_logging(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let default_level = config.get::<String>("log_level")?;
    let env_with_defaults = env_logger::Env::default().default_filter_or(default_level);
    env_logger::Builder::from_env(env_with_defaults).init();
    Ok(())
}
