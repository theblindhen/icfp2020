use crate::encodings::{vcons, vi64, vnil, modulate, demodulate};
use crate::interpreter::*;
use log::*;
use std::env;
use std::io::BufRead;
use crate::value_tree::*;

const APIKEY : &'static str = "91bf0ff907084b7595841e534276a415";

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = modulate(&body);

    println!("Sending: {}", body);

    let response = ureq::post(url)
        .timeout(std::time::Duration::from_secs(30))
        .send_string(&encoded_body)
        .into_string()?;

    if (response == "") {
        panic!("received empty response from server");
    }

    let (decoded_response, remainder) = demodulate(&response);
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
    parse_value_tree(&tree).unwrap()
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

pub fn main(server_url: &str, player_key: &str, proxy: bool, interactive: bool) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let player_key: i64 = player_key.parse().unwrap();

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let url =
        if proxy {
            format!("{}/aliens/send?apiKey={}", server_url, APIKEY)
        } else {
            format!("{}/aliens/send", server_url)
        };

    let _ = post(&url, &join_msg(player_key))?;
    let _ = post(&url, &start_msg(player_key))?;

    if interactive {
        run_interactively(&url)
    } else {
        run_ai(&url, player_key)
    }

    Ok(())
}
