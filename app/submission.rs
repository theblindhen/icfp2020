#![allow(unused)]

use log::*;
use std::env;

fn post(url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending POST request with body {} to {}...", body, url);

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

    let url = format!("{}/aliens/send", server_url);
    let body = format!("(2, {}, nil)", player_key);
    let reply = post(&url, &body)?;

    println!("Reply: {}", reply);
    Ok(())
}
