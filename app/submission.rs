#![allow(unused)]

mod aplang;
mod bits2d;
mod draw;
mod encodings;
mod interpreter;
mod lexer;

use log::*;
use std::env;

fn post(url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending POST request with body {} to {}", body, url);

    let reply = ureq::post(url)
        .timeout(std::time::Duration::from_secs(30))
        .send_string(body)
        .into_string()?;
    Ok(reply)
}

fn join_msg(player_key: i64) -> String {
    use encodings::{vcons, vint, vnil};

    let join = vcons(vint(2), vcons(vint(player_key), vcons(vnil(), vnil())));
    encodings::modulate(&join)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key: i64 = args[2].parse().unwrap();

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let url = format!("{}/aliens/send", server_url);
    let reply = post(&url, &join_msg(player_key))?;

    println!("Reply: {}", reply);
    Ok(())
}
