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

    loop {
        let response = ureq::post(url)
            .timeout(std::time::Duration::from_secs(30))
            .send_string(&encoded_body)
            .into_string();
        match response {
            Ok(response) => {
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

                return Ok(decoded_response);
            }
            Err(e) => error!("Error communicating with server:\n\t{}\nTrying again", e),
        }
    }
}

pub fn parse(tree: &str) -> ValueTree {
    parse_value_tree(&tree).unwrap()
}

fn join_msg(player_key: i64) -> ValueTree {
    parse(&format!("[2, {}, []]", player_key))
}

fn start_msg(player_key: i64, game_response: Option<GameResponse>) -> ValueTree {
    match get_max_resources(game_response) {
        Some(max_resources) => parse(&format!(
            "[3, {}, [{}, 0, 16, 1]]",
            player_key,
            max_resources - 2 - (12 * 16)
        )),
        None => parse(&format!("[3, {}, [1, 1, 1, 1]]", player_key)),
    }
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

trait AI {
    fn start(&mut self, player_key: i64, game_response: Option<GameResponse>) -> ValueTree;
    fn step(&mut self, game_response: GameResponse) -> Vec<Command>;
}

struct Stationary {}
struct Noop {}

impl AI for Stationary {
    fn start(&mut self, player_key: i64, game_response: Option<GameResponse>) -> ValueTree {
        start_msg(player_key, game_response)
    }

    fn step(&mut self, game_response: GameResponse) -> Vec<Command> {
        match (game_response.static_game_info, game_response.game_state) {
            (Some(static_game_info), Some(game_state)) => {
                let our_role = static_game_info.role;
                let our_ship = game_state
                    .ships
                    .iter()
                    .find(|&ship| ship.role == our_role)
                    .unwrap();

                let (gx, gy) = gravity(our_ship.position);

                vec![Command::Accelerate(our_ship.ship_id, (gx, gy))]
            }
            _ => {
                error!("Error in survivor ai: no static game info or game state");
                vec!()
            }
        }
    }
}

impl AI for Noop {
    fn start(&mut self, player_key: i64, game_response: Option<GameResponse>) -> ValueTree {
        start_msg(player_key, game_response)
    }

    fn step(&mut self, game_response: GameResponse) -> Vec<Command> {
        vec!()
    }
}

fn run_ai(
    ai: &mut dyn AI,
    url: &str,
    player_key: i64,
    initial_game_response: ValueTree,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::protocol::*;

    let mut game_response = try_parse_response(&initial_game_response);
    game_response = try_parse_response(&post(&url, &ai.start(player_key, game_response))?);

    loop {
        let cmds = match game_response {
            None => vec!(),
            Some(game_response) => ai.step(game_response),
        };
        let mut commands = vnil();
        for cmd in cmds {
            commands = vcons(flatten_command(cmd), commands);
        }

        let request = vcons(vi64(4), vcons(vi64(player_key), vcons(commands, vnil())));

        game_response = try_parse_response(&post(url, &request)?);
    }
}

pub fn main(
    server_url: &str,
    player_key: &str,
    ai: &str,
    proxy: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_key: i64 = player_key.parse().unwrap();

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let url = if proxy {
        format!("{}/aliens/send?apiKey={}", server_url, APIKEY)
    } else {
        format!("{}/aliens/send", server_url)
    };

    let initial_game_response = post(&url, &join_msg(player_key))?;

    let mut ai : Box<dyn AI> = match ai {
        "stationary" => Box::from(Stationary{}),
        _ => Box::from(Noop{}),
    };

    if interactive {
        run_interactively(&url, player_key)?
    } else {
        run_ai(&mut *ai, &url, player_key, initial_game_response)?
    }

    Ok(())
}
