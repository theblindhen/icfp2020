#![allow(unused)]

mod aplang;
mod encodings;
mod lexer;
mod interpreter;
mod bits2d;
mod draw;
mod nom_helpers;

use crate::aplang::*;
use crate::encodings::*;
use crate::draw::*;

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
    steps: Option<String>, // Format: "(x,y) (x,y) ... (x,y)"

    #[structopt(long)]
    allzero: bool,

    #[structopt(long)]
    make_join_message_with_key: Option<i64>,
}

// Parenthesised, comma-separated point
pub fn parse_point_paren(s : &str) -> Option<Point> {
    let words: Vec<_> = s[1..s.len()-1].split(',').collect();
    match &words[..] {
        [x, y] => {
            match (x.parse(), y.parse()) {
                (Ok(x), (Ok(y))) => Some(Point(x,y)),
                _ => None
            }
        },
        _ => None
    }
}

// Space-separated list of parenthesised points
pub fn parse_points(s : &str) -> Option<Vec<Point>> {
    let words: Vec<_> = s.split_whitespace().collect();
    let rev : Option<Vec<Point>> = words.into_iter().map(parse_point_paren).collect();
    match rev {
        None => None,
        Some(mut rev) => {
            rev.reverse();
            Some(rev)
        }
    }
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
        use encodings::{vcons, vnil, vint};
        let join = vcons(vint(2), vcons(vint(key), vcons(vnil(), vnil())));
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
    let mut points : Vec<Point> =
        match opt.steps {
            None => vec![],
            Some(steps) => {
                match parse_points(&steps) {
                    Some(points) => points,
                    _ => vec![]
                }
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
    let mut screen_offset = Point(0,0);
    loop {
        let point =
            match points.pop() {
                Some(p) => p,
                None => {
                    if opt.allzero {
                        Point(0,0)
                    } else {
                        match point_from_terminal(screen_offset.0, screen_offset.1) {
                            None => return,
                            Some(new_point) => {
                                new_point
                            }
                        }
                    }
                }
            };
        round += 1;
        println!("ROUND {}", round);
        let (new_state, screens) = interpreter::interact(prg_var, &mut env.clone(), &state, point);
        let overlay = draw::Overlay::new(screens);
        println!("State\n{}\n{}", state, encodings::modulate(&state));
        println!("Sent point: ({}, {})", point.0, point.1);
        // println!("Overlays:\n{}", round, overlay);
        // overlay.dump_image()
        overlay.dump_image(&format!("imgs/round_{:03}.png", round));
        state = new_state;
        screen_offset = Point(overlay.xstart, overlay.ystart);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
