#![allow(unused)]

mod aplang;
mod value_tree;
mod encodings;
mod lexer;
mod interpreter;
mod bits2d;
mod draw;
mod nom_helpers;

use crate::aplang::*;
use crate::encodings::*;

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
    protocol: Option<String>,

    #[structopt(short, long)]
    state: Option<String>,

    #[structopt(long)]
    first: Option<String>,

    #[structopt(long)]
    allzero: bool,

    #[structopt(long)]
    make_join_message_with_key: Option<i64>,
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

    if let Some(key) = opt.make_join_message_with_key {
        use encodings::{vcons, vnil, vi64};
        let join = vcons(vi64(2), vcons(vi64(key), vcons(vnil(), vnil())));
        let modulated = encodings::modulate(&join);
        println!("{}", modulated);
        return;
    }

    let program =
        match &opt.protocol.expect("Specify a protocol")[..] {
            "galaxy" => lexer::lex("galaxy.txt"),
            "statelessdraw" => lexer::oneliner("ap ap c ap ap b b ap ap b ap b ap cons 0 ap ap c ap ap b b cons ap ap c cons nil ap ap c ap ap b cons ap ap c cons nil nil"),
            "statefuldraw" => lexer::oneliner("ap ap b ap b ap ap s ap ap b ap b ap cons 0 ap ap c ap ap b b cons ap ap c cons nil ap ap c cons nil ap c cons"),
            other => panic!("Unknown protocol '{}'", other)
        };
    let (prg_var, env) = interpreter::parse_program(&program);
    let mut env = env;
    let mut point =
        match opt.first {
            None => draw::point_from_terminal(0, 0).unwrap(),
            Some(sints) =>
                match draw::parse_two_ints(&sints[1..sints.len()-1]) {
                    Some(point) => point,
                    None => draw::point_from_terminal(0, 0).unwrap(),
                }
        };
    let mut state =
        match opt.state {
            None => interpreter::initial_state(),
            Some(state_str) => {
                let (vtree, _) = encodings::demodulate(&state_str);
                vtree
            }
        };
    let mut round = 0;
    loop {
        round += 1;
        println!("ROUND {}", round);
        let (new_state, screens) = interpreter::interact(prg_var, &mut env.clone(), &state, point);
        let overlay = draw::Overlay::new(screens);
        println!("State\n{}\n{}", state, encodings::modulate(&state));
        println!("Sent point: ({}, {})", point.0, point.1);
        // println!("Overlays:\n{}", round, overlay);
        // overlay.dump_image()
        overlay.dump_image(&format!("imgs/round_{:03}.png", round));
        if opt.allzero {
            point = draw::Point(0,0)
        } else {
            match draw::point_from_terminal(overlay.xstart, overlay.ystart) {
                None => return,
                Some(new_point) => {
                    point = new_point;
                }
            }
        }
        state = new_state;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
