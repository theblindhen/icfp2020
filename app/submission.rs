#![allow(unused)]

mod aplang;
mod bits2d;
mod draw;
mod encodings;
mod interpreter;
mod lexer;
mod nom_helpers;
mod value_tree;

use encodings::{vcons, vi64, vnil};
use interpreter::*;
use log::*;
use std::env;
use std::io::BufRead;
use value_tree::*;

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = encodings::modulate(&body);

    println!("Sending: {}", body);

    let response = ureq::post(url)
        .timeout(std::time::Duration::from_secs(30))
        .send_string(&encoded_body)
        .into_string()?;

    if (response == "") {
        panic!("received empty response from server");
    }

    let (decoded_response, remainder) = encodings::demodulate(&response);
    if (remainder != "") {
        panic!(
            "non-empty remainder when demodulating server response: {}",
            response
        );
    }

    println!("Received: {}", decoded_response);

    Ok(decoded_response)
}

fn parse(tree: &str) -> ValueTree {
    value_tree::parse_value_tree(&tree).unwrap()
}

fn join_msg(player_key: i64) -> ValueTree {
    parse(&format!("[2, {}, []]", player_key))
}

fn start_msg(player_key: i64) -> ValueTree {
    parse(&format!("[3, {}, [1, 1, 1, 1]]", player_key))
}

fn run_interactively(url: &str) {
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    loop {
        println!("Write a message to send to server");

        if stdin.read_line(&mut buf).unwrap() == 0 {
            panic!("dummy")
        }

        post(url, &parse(&buf)); 
    }
}

fn run_ai(url: &str, player_key: i64) {
    loop {
        post(url, &parse(&format!("[4, {}, []]", player_key)));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key: i64 = args[2].parse().unwrap();
    let interactive = {
        if args.len() > 3 {
            Some(&args[3])
        } else {
            None
        }
    };

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);
    let url = match interactive {
        Some(api_key) => format!("{}/aliens/send?apiKey={}", server_url, api_key),
        None => format!("{}/aliens/send", server_url),
    };

    let _ = post(&url, &join_msg(player_key))?;
    let _ = post(&url, &start_msg(player_key))?;

    match interactive {
        Some(_) => run_interactively(&url),
        None => run_ai(&url, player_key),
    }

    Ok(())
}
