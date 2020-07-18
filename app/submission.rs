#![allow(unused)]

mod aplang;
mod bits2d;
mod draw;
mod encodings;
mod interpreter;
mod lexer;

use encodings::{vcons, vint, vnil};
use interpreter::*;
use log::*;
use std::env;

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = encodings::modulate(&body);

    println!("Sending POST request with body {:?} to {}", body, url);

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

    println!("Received POST response {:?}", response);

    Ok(decoded_response)
}

fn join_msg(player_key: i64) -> ValueTree {
    vcons(vint(2), vcons(vint(player_key), vcons(vnil(), vnil())))
}

fn start_msg(player_key: i64) -> ValueTree {
    let initial_params = vcons(
        vint(0),
        vcons(vint(0), vcons(vint(0), vcons(vint(0), vnil()))),
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
    let url = format!("{}/aliens/send", server_url);

    let _ = post(&url, &join_msg(player_key))?;
    let _ = post(&url, &start_msg(player_key))?;

    Ok(())
}
