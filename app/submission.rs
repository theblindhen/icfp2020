use crate::encodings::{demodulate, modulate, vcons, vi64, vnil};
use crate::interpreter::*;
use crate::protocol::*;
use crate::value_tree::*;
use log::*;
use std::env;
use std::io::BufRead;

const APIKEY: &'static str = "91bf0ff907084b7595841e534276a415";

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

pub fn parse(tree: &str) -> ValueTree {
    parse_value_tree(&tree).unwrap()
}

fn join_msg(player_key: i64) -> ValueTree {
    parse(&format!("[2, {}, []]", player_key))
}

fn start_msg(player_key: i64) -> ValueTree {
    parse(&format!("[3, {}, [1, 1, 1, 1]]", player_key))
}

fn run_interactively(url: &str, player_key: i64) -> Result<(), Box<dyn std::error::Error>> {
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

// compute gravity in given position
fn gravity((x, y): (i64, i64)) -> (i64, i64) {
    if x.abs() == y.abs() {
        (-x.signum(), -y.signum())
    } else {
        if x.abs() > y.abs() {
            (-x.signum(), 0)
        } else {
            (0, -y.signum())
        }
    }
}

fn decide_command(game_response: Option<GameResponse>) -> Vec<Command> {
    match game_response {
        Some(game_response) => match (game_response.static_game_info, game_response.game_state) {
            (Some(static_game_info), Some(game_state)) => {
                let our_role = static_game_info.role;
                let our_ship = game_state
                    .ships
                    .iter()
                    .find(|&ship| ship.role == our_role)
                    .unwrap();

                let (gx, gy) = gravity(our_ship.position);

                vec![Command::Accelerate(our_ship.ship_id, (-gx, -gy))]
            }
            _ => vec![],
        },
        None => vec![],
    }
}

fn dummy_ai(url: &str, player_key: i64) {
    loop {}
}

fn try_parse_response(response: &ValueTree) -> Option<GameResponse> {
    use crate::protocol::*;

    match parse_game_response(&response) {
        Ok(res) => Some(res),
        Err(err) => {
            println!("could not parse game response: {}", err);
            None
        }
    }
}

fn run_ai(
    url: &str,
    player_key: i64,
    initial_game_response: ValueTree,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::protocol::*;

    let mut game_response = try_parse_response(&post(&url, &start_msg(player_key))?);

    loop {
        let mut commands = vnil();
        for cmd in decide_command(game_response) {
            commands = vcons(flatten_command(cmd), commands);
        }

        let request = vcons(vi64(4), vcons(vi64(player_key), vcons(commands, vnil())));

        game_response = try_parse_response(&post(url, &request)?);
    }
}

pub fn main(
    server_url: &str,
    player_key: &str,
    proxy: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let player_key: i64 = player_key.parse().unwrap();

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let url = if proxy {
        format!("{}/aliens/send?apiKey={}", server_url, APIKEY)
    } else {
        format!("{}/aliens/send", server_url)
    };

    let initial_game_response = post(&url, &join_msg(player_key))?;

    if interactive {
        run_interactively(&url, player_key)?
    } else {
        run_ai(&url, player_key, initial_game_response)?
    }

    Ok(())
}
