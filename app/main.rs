#![allow(unused)]

mod aplang;
mod encodings;
mod lexer;
mod interpreter;
mod bits2d;
mod draw;

use crate::aplang::*;

// Deserializing
use serde::Deserialize;
use serde_json::Value;

use structopt::StructOpt;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;

use log::*;

// Struct for command line parsing 
#[derive(StructOpt, Debug)]
#[structopt()]
struct MyOpt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    #[structopt(default_value = "1000", short, long)]
    timeout: u32,

    #[structopt(short, long)]
    protocol: String,
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
fn http_json(url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending request to {}...", url);

    let reply =
        ureq::post(url)
            .timeout(std::time::Duration::from_secs(30))
            .send_string(body)
            .into_string()?;
    Ok(reply)
}

fn file_json(path: PathBuf) -> Result<Vec<Repo>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // Read the JSON contents of the file as an instance of `User`.
    let json = serde_json::from_reader(reader)?;
    Ok(json)
}

type RGBA = (u8,u8,u8,u8);

const COLORS : [RGBA; 12] =
    [
      (106,168, 82,255),
      (224, 73, 87,255),
      (125, 97,186,255),
      (255,122, 66,255),
      ( 40,120,181,255),
      (245,223,113,255),
      (194,180,234,255),
      (249,207,221,255),
      (177,223,243,255),
      (198,227,171,255),
      (244,234,150,255),
      (255,199,174,255),
    ];

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
        0 => level = log::LevelFilter::Warn,
        1 => level = log::LevelFilter::Info,
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

    let program =
        match &opt.protocol[..] {
            "galaxy" => lexer::lex("galaxy.txt"),
            "statelessdraw" => lexer::oneliner("ap ap c ap ap b b ap ap b ap b ap cons 0 ap ap c ap ap b b cons ap ap c cons nil ap ap c ap ap b cons ap ap c cons nil nil"),
            "statefuldraw" => lexer::oneliner("ap ap b ap b ap ap s ap ap b ap b ap cons 0 ap ap c ap ap b b cons ap ap c cons nil ap ap c cons nil ap c cons"),
            _ => panic!("Unknown protocol '{}'", opt.protocol)
        };
    let (prg_var, env) = interpreter::parse_program(&program);
    let mut env = env;
    let mut point = draw::Point(0, 0);
    let mut state = interpreter::initial_state();
    let mut round = 0;
    loop {
        round += 1;
        println!("ROUND {}", round);
        let (new_state, screens) = interpreter::interact(prg_var, &mut env, &state, point);
        for (i, screen) in screens.into_iter().enumerate() {
            println!("Screen {}_{}:\n{}\n", round, i, screen);
            screen.dump_image(&format!("imgs/screen_{}_{}.png", round, i), COLORS[i])
        }
        match draw::point_from_terminal() {
            None => return,
            Some(new_point) => {
                point = new_point;
                state = new_state;
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
