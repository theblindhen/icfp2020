use crate::encodings::{demodulate, modulate, vcons, vi64, vnil};
use crate::interpreter::*;
use crate::protocol::*;
use crate::value_tree::*;
use log::*;
use std::env;
use std::io::BufRead;
use std::convert::TryInto;

const APIKEY: &'static str = "91bf0ff907084b7595841e534276a415";

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = modulate(&body);

    println!("Sending:  {}", body);

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

fn multiplayer_msg() -> ValueTree {
    parse(&format!("[1, 0]"))
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
                vec![]
            }
        }
    }
}

impl AI for Noop {
    fn start(&mut self, player_key: i64, game_response: Option<GameResponse>) -> ValueTree {
        start_msg(player_key, game_response)
    }

    fn step(&mut self, game_response: GameResponse) -> Vec<Command> {
        vec![]
    }
}

fn run_ai(ai: &mut dyn AI, url: &str, player_key: i64) -> Result<(), Box<dyn std::error::Error>> {
    use crate::protocol::*;

    let initial_game_response = post(&url, &join_msg(player_key))?;
    let mut game_response = try_parse_response(&initial_game_response);
    game_response = try_parse_response(&post(&url, &ai.start(player_key, game_response))?);

    loop {
        println!("Game response was:\n{:?}\ny", game_response);
        let cmds = match game_response {
            None => vec![],
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

fn player_key(tree: &ValueTree)-> Result<i64, Box<dyn std::error::Error>>  {
    let lst = to_native_list(tree);
    assert!(lst.len() == 2);
    crate::protocol::as_int("player key", lst[1])
}

fn initiate_multiplayer_game(url: &str) -> Result<(i64, i64), Box<dyn std::error::Error>> {
    let resp_tree = &post(&url, &multiplayer_msg())?;
    let resp = to_native_list(resp_tree);
    assert!(resp.len() == 2);

    let player_keys = to_native_list(resp[1]);
    assert!(player_keys.len() == 2);

    Ok((player_key(player_keys[0])?, player_key(player_keys[1])?))
}

fn run_ais(
    mut ai1: Box<dyn AI + Send>,
    mut ai2: Box<dyn AI + Send>,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let (player_key1, player_key2) = initiate_multiplayer_game(&url)?;
    println!("initiate multiplayer game; player keys: {} and {}", player_key1, player_key2);

    let url1 = String::from(url);
    let thr1 = std::thread::spawn(move || {
        let err = run_ai(&mut *ai1, &url1, player_key1);
        println!("an error occured while running AI 1: {:?}", err);
    });
    let url2 = String::from(url);
    let thr2 = std::thread::spawn(move || {
        let err = run_ai(&mut *ai2, &url2, player_key2);
        println!("an error occured while running AI 2: {:?}", err);
    });

    thr1.join();
    thr2.join();

    Ok(())
}

fn get_ai(ai_str: Option<String>) -> Option<Box<dyn AI + Send>> {
    match ai_str {
        Some(ai_str) => match ai_str.as_ref() {
            "stationary" => Some(Box::from(Stationary {})),
            "noop" => Some(Box::from(Noop {})),
            _ => {
                println!("unknown ai {}, using default", ai_str);
                Some(Box::from(Stationary {}))
            }
        },
        None => None,
    }
}

pub fn main(
    server_url: &str,
    player_key: &str,
    ai1: Option<String>,
    ai2: Option<String>,
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

    if interactive {
        post(&url, &join_msg(player_key))?;
        run_interactively(&url, player_key)?
    } else {
        let mut ai1 = get_ai(ai1).unwrap_or(Box::from(Stationary {}));
        let mut ai2 = get_ai(ai2);

        match ai2 {
            Some(mut ai2) => run_ais(ai1, ai2, &url)?,
            None => run_ai(&mut *ai1, &url, player_key)?,
        }
    }

    Ok(())
}
