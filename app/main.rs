// Deserializing
use serde::Deserialize;
use serde_json::Value;

use structopt::StructOpt;
use std::path::PathBuf;

use log::*;

// Struct for command line parsing 
#[derive(StructOpt, Debug)]
#[structopt()]
struct MyOpt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    /// Run in interactive mode
    #[structopt(short, long)]
    interactive: bool,

    #[structopt(default_value = "1000", short, long)]
    timeout: u32,

    /// Input file
    #[structopt(name = "MAP", default_value = "my.map", parse(from_os_str))]
    file: PathBuf,
}


// Example of a reduced, regular type-safe structure for holding a subset of the JSON
// The Deserialize `derive` requires the `serde` dependency.
#[derive(Deserialize, Debug)]
struct Repo {
    name: String,
    id: i32,
    description: String
}

// Demonstrates sending an HTTP request and decoding the response as JSON.
fn http_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending request...");

    let json : Value =
        ureq::get("http://jsrn.dk/advanced.json")
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .into_json_deserialize().unwrap();

    let repos : Vec<Repo> = serde_json::from_value(json)?;
    println!("{:#?}", repos);

    Ok(())
}

fn main() {
    // Parse command line arguments according to the struct
    let opt = MyOpt::from_args();

    // Set up logging 
    // There are five macros similar to "println!":
    // error!, warn!, info!, debug! and trace!
    // In order of decreasing priority.
    // Use -v, -vv, -vvv to specify logging level.
    let level;
    match opt.verbose {
        0 => level = log::LevelFilter::Off,
        1 => level = log::LevelFilter::Warn,
        2 => level = log::LevelFilter::Info,
        _ => level = log::LevelFilter::Trace,
    }
    env_logger::builder()
        .filter_level(level)
        .init();

    error!("You are seeing errors");
    warn!("You are seeing warnings");
    info!("You are seeing info");
    debug!("You are seeing debug stuff");
    trace!("You are reading everything");



    // let level_filter = match opt.verbose {
    //     1 => LevelFilter::Debug,
    //     2 => LevelFilter::Trace,
    //     _ => LevelFilter::Info,
    // };

    if opt.interactive { 
        println!("\nYou asked for INTERACTIVE mode");
    } else {
        println!("\nYou asked for NON-INTERACTIVE mode");
    }





    match http_json() {
        Err(_) => println!("\nThere was an error with the HTTP request or JSON parsing"),
        Ok(()) => ()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
