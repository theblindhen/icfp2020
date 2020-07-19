#![allow(unused)]

mod aplang;
mod value_tree;
mod encodings;
mod lexer;
mod interpreter;
mod bits2d;
mod draw;
mod nom_helpers;
mod protocol;
mod submission;
mod gui;

use crate::aplang::*;
use crate::encodings::*;
use crate::draw::*;

use structopt::StructOpt;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;

use log::*;

const DEFAULT_AI : &'static str = "stationary";

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

    #[structopt(long)]
    modulate: Option<String>,

    #[structopt(long)]
    demodulate: Option<String>,

    #[structopt(long)]
    proxy: bool,

    #[structopt(long)]
    interactive: bool,

    #[structopt(long)]
    gui: Option<i32>,

    #[structopt(name = "SERVER_URL_AND_PLAYER_KEY")]
    url_and_key: Vec<String>,

    #[structopt(default_value = DEFAULT_AI,long)]
    ai: String,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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


    ////////////////////////////////////////
    // RUN MODE 1: SUBMISSION
    ////////////////////////////////////////
    let ai =
        if opt.ai == "stationary" || opt.ai == "noop" {
            &opt.ai
        } else {
            error!("Unknown AI requested '{}'. Using default '{}'", opt.ai, DEFAULT_AI );
            DEFAULT_AI
        };
    match &opt.url_and_key[..] {
        [server_url, player_key] =>
            return submission::main(server_url, player_key, &ai, opt.proxy, opt.interactive),
        [] => (),
        _ => panic!("Bad args"),
    }



    ////////////////////////////////////////
    // RUN MODE 2: MODULATE/DEMODULATE
    ////////////////////////////////////////
    if let Some(input) = opt.modulate {
        let modulated = encodings::modulate(&value_tree::parse_value_tree(&input).unwrap());
        println!("{}", modulated);
        return Ok(());
    }

    if let Some(input) = opt.demodulate {
        let demodulated = encodings::demodulate(&input);
        println!("{}", demodulated.0);
        return Ok(());
    }


    /////////////////////////////////////////////
    // RUN MODE 3: RUN GALAXY (OR OTHER PROTOCOL)
    /////////////////////////////////////////////
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

    if let Some(scale) = opt.gui {
        return gui::gui(prg_var, env, state, scale)
    }

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
                            None => return Ok(()),
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
