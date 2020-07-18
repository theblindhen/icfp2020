#![allow(unused)]

use log::*;
use std::env;

fn post(url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending request to {}...", url);

    let reply =
        ureq::post(url)
            .timeout(std::time::Duration::from_secs(30))
            .send_string(body)
            .into_string()?;
    Ok(reply)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let reply = post(server_url, player_key)?;

    println!("Reply: {}", reply);
    Ok(())
}

