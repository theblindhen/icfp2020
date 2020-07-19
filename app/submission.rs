#![allow(unused)]

mod aplang;
mod value_tree;
mod bits2d;
mod draw;
mod encodings;
mod interpreter;
mod lexer;
mod nom_helpers;

use encodings::{vcons, vint, vnil};
use interpreter::*;
use value_tree::*;
use log::*;
use std::env;
use std::io::BufRead;

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = encodings::modulate(&body);

    println!("Sending POST request with body {} to {}", body, url);

    let response = ureq::post(url)
        .timeout(std::time::Duration::from_secs(30))
        .send_string(&encoded_body)
        .into_string()?;

    let (decoded_response, remainder) = encodings::demodulate(&response);
    if (remainder != "") {
        panic!(
            "non-empty remainder when demodulating server response: {}",
            response
        );
    }

    println!("Received POST response {}", decoded_response);

    Ok(decoded_response)
}

fn join_msg(player_key: i64) -> ValueTree {
    vcons(vint(2), vcons(vint(player_key), vcons(vnil(), vnil())))
}

fn start_msg(player_key: i64) -> ValueTree {
    let initial_params = vcons(
        vint(1),
        vcons(vint(1), vcons(vint(1), vcons(vint(1), vnil()))),
    );

    vcons(
        vint(3),
        vcons(vint(player_key), vcons(initial_params, vnil())),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key: i64 = args[2].parse().unwrap();

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);
    let url = format!("{}/aliens/send?apiKey=91bf0ff907084b7595841e534276a415", server_url);

    let _ = post(&url, &join_msg(player_key))?;
    let _ = post(&url, &start_msg(player_key))?;

    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    loop {
        println!("Write a message to send to server");

        if stdin.read_line(&mut buf).unwrap() == 0 {
            panic!("dummy")
        }

        match value_tree::parse_value_tree(&buf) {
            Some(wtree) => {
                post(&url, &wtree);
            },
            None => ()
        }
    }

    Ok(())
}