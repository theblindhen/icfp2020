use crate::encodings::{demodulate, modulate, vcons, vi64, vnil};
use crate::interpreter::*;
use crate::protocol::*;
use crate::sim;
use crate::value_tree::*;
use log::*;
use std::convert::TryInto;
use std::env;
use std::io::BufRead;

const APIKEY: &'static str = "91bf0ff907084b7595841e534276a415";
const DETONATION_RADIUS: i64 = 4;

fn post(url: &str, body: &ValueTree) -> Result<ValueTree, Box<dyn std::error::Error>> {
    let encoded_body = modulate(&body);

    println!("Sending:  {}", body);

    loop {
        let response = ureq::post(url).send_string(&encoded_body).into_string();
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

fn find_ship_by_id(game_response: &GameResponse, ship_id: i64) -> Option<&Ship> {
    match &game_response.game_state {
        Some(game_state) => game_state.ships.iter().find(|ship| ship.ship_id == ship_id),
        None => None,
    }
}

fn find_ship(game_response: &GameResponse, role: Role) -> Option<&Ship> {
    match &game_response.game_state {
        Some(game_state) => game_state.ships.iter().find(|ship| ship.role == role),
        None => None,
    }
}

fn find_ships(game_response: &GameResponse, role: Role) -> Vec<&Ship> {
    match &game_response.game_state {
        Some(game_state) => game_state
            .ships
            .iter()
            .filter(|ship| ship.role == role)
            .collect(),
        None => vec![],
    }
}

fn unique_positions(ships: &Vec<&Ship>) -> i64 {
    let pos_set : std::collections::hash_set::HashSet<(i64,i64)> =
        ships.into_iter()
        .map(|s| s.position)
        .collect();
    pos_set.len() as i64
}

fn our_role(game_response: &GameResponse) -> Option<Role> {
    match &game_response.static_game_info {
        Some(game_info) => Some(game_info.role),
        None => None,
    }
}

fn inverse_role(role: Role) -> Role {
    use crate::protocol::Role::*;

    match role {
        Attacker => Defender,
        Defender => Attacker,
    }
}

fn mother_ship<'a>(ships: &Vec<&'a Ship>) -> Option<&'a Ship> {
    for ship in ships {
        if ship.ship_id <= 1 {
            return Some(ship)
        }
    }
    None
}

fn resource_score(ship: &Ship) -> i64 {
    if let Some(resources) = &ship.resources {
        resources.fuel + 4*resources.cannon + 12*resources.cooling + 2*resources.clones
    } else {
        0
    }
}


fn total_resources(ships: &Vec<&Ship>) -> i64 {
    let mut sum = 0;
    for ship in ships {
        sum += resource_score(ship);
    }
    sum
}

trait AI {
    fn step_a_ship(&mut self, ship: &Ship, game_response: &GameResponse) -> Vec<Command> {
        self.step(&vec![ship], &game_response)
    }

    fn step(&mut self, ships: &Vec<&Ship>, game_response: &GameResponse) -> Vec<Command> {
        let mut cmds = vec![];
        for ship in ships {
            cmds.append(&mut self.step_a_ship(ship, &game_response));
        }
        cmds
    }
}

// struct Shoot {}
// impl Shoot {
//     fn step_a_ship(&mut self, our_ship: &Ship, game_response: &GameResponse) -> Vec<Command> {
//         fn close_enough(us: &Ship, them: &Ship) -> bool {
//             let dist =
//                 (us.position.0 - them.position.0).abs() + (us.position.1 - them.position.1).abs();

//             dist <= 32
//         }
//         fn cold(ship: &Ship) -> bool {
//             if let Some(resources) = &ship.resources {
//                 ship.heat + resources.cannon - resources.cooling + 8 <= 64
//             } else {
//                 false
//             }
//         }
//         fn little_cooling(ship: &Ship) -> bool {
//             if let Some(resources) = &ship.resources {
//                 resources.cooling <= 16
//             } else {
//                 true
//             }
//         }
//         let our_role = our_role(&game_response).unwrap();
//         let opp_ships = find_ships(&game_response, inverse_role(our_role));

//         let (gx, gy) = gravity(our_ship.position);

//         let mut cmds = vec![Command::Accelerate(our_ship.ship_id, (gx, gy))];

//         for target in opp_ships {
//             if close_enough(our_ship, target) && cold(our_ship) && little_cooling(target) {
//                 let target_gravity = gravity(target.position);
//                 let (target_vx, target_vy) = (
//                     target.velocity.0 + target_gravity.0,
//                     target.velocity.1 + target_gravity.1,
//                 );
//                 let (target_x, target_y) =
//                     (target.position.0 + target_vx, target.position.1 + target_vy);

//                 if let Some(resources) = &our_ship.resources {
//                     cmds.push(Command::Shoot(
//                         our_ship.ship_id,
//                         (target_x, target_y),
//                         resources.cannon,
//                     ));
//                 }
//                 break;
//             }
//         }
//         cmds
//     }
// }
// impl AI for Shoot {
//     fn start(&mut self, player_key: i64, game_response: Option<GameResponse>) -> (i64,i64,i64,i64) {
//         use crate::protocol::*;

//         match get_max_resources(game_response) {
//             Some(max_resources) => {
//                 let resource_split = (max_resources - 256 - 1 * PARAM_MULT.3) / 2;
//                 let cannon = resource_split / PARAM_MULT.1;
//                 let cooling = resource_split / PARAM_MULT.2;
//                 (256, cannon, cooling, 1)
//             },
//             None => (1,1,1,1)
//         }
//     }

//     fn step(&mut self, ship_ids: Vec<i64>, game_response: &GameResponse) -> Vec<Command> {
//     }

// }

struct Orbiting {}
impl Orbiting {

    /// std::i64::MAX if it doesn't crash
    fn goodness_of_drift_from(sv: &sim::SV, planet_radius: i64) -> i64 {
        let mut last_pos = sv.s;
        let mut dist_sum = 0;
        for pos in sv.one_orbit_positions(planet_radius, 384) {
            if sim::collided_with_planet(planet_radius, pos) {
                return dist_sum;
            }
            dist_sum += sim::max_norm(pos, last_pos);
            last_pos = pos;
        }
        std::i64::MAX
    }

    pub fn survives(sv: &sim::SV, planet_radius: i64) -> bool {
        Orbiting::goodness_of_drift_from(sv, planet_radius) == std::i64::MAX
    }

    pub fn get_best_nonzero_thrust(sv: &sim::SV, planet_radius: i64) -> (sim::XY, i64) {
        let mut best_measure = i64::MIN;
        let mut best_thrust = None;
        for &thrust in &sim::nonzero_thrusts_random() {
            let mut thrusted_sv = sv.clone();
            thrusted_sv.thrust(thrust);
            let measure =
                Orbiting::goodness_of_drift_from(&thrusted_sv, planet_radius);
            if measure > best_measure {
                best_measure = measure;
                best_thrust = Some(thrust);
            }
        }
        (best_thrust.unwrap(), best_measure)
    }
}

impl AI for Orbiting {
    fn step_a_ship(&mut self, ship: &Ship, game_response: &GameResponse) -> Vec<Command> {
        use crate::protocol::Role::*;

        fn within_detonation_range(ship1: &Ship, ship2: &Ship) -> bool {
            use crate::sim::*;

            let mut ship1_sv = SV { s: ship1.position.into(), v: ship1.velocity.into() };
            let mut ship2_sv = SV { s: ship2.position.into(), v: ship2.velocity.into() };

            ship1_sv.drift();
            ship2_sv.drift();

            (ship1_sv.s.x - ship2_sv.s.x).abs() <= DETONATION_RADIUS - 1
                && (ship1_sv.s.y - ship2_sv.s.y).abs() <= DETONATION_RADIUS - 1
        }

        fn detonation_score(ship: &Ship, other_ships: &Vec<&Ship>) -> i64 {
            let mut sum = 0;
            for other_ship in other_ships {
                if within_detonation_range(ship, other_ship) {
                    sum += resource_score(other_ship);
                }
            }
            sum
        }

        match (&game_response.static_game_info, &game_response.game_state) {
            (Some(static_game_info), Some(game_state)) => {
                let our_role = static_game_info.role;
                let opp_ships : Vec<&Ship> = game_state
                    .ships
                    .iter()
                    .filter(|ship| ship.role != our_role)
                    .collect();

                let sv = sim::SV {
                    s: ship.position.into(),
                    v: ship.velocity.into(),
                };

                // Detonate!
                if our_role == Attacker {
                    if let Some(resources) = &ship.resources {
                        // Destroy the last ship
                        if (opp_ships.len() == 1
                            && within_detonation_range(ship, opp_ships[0])) {
                            return vec![Command::Detonate(ship.ship_id)]
                        } else if (resources.fuel < 10) {
                            let our_ships = find_ships(&game_response, our_role);
                            let kill_ratio = detonation_score(&ship, &opp_ships) as f64 / total_resources(&opp_ships) as f64; 
                            let mut loss_ratio = detonation_score(&ship, &our_ships) as f64 / total_resources(&our_ships) as f64;
                            if let Some(mother) = mother_ship(&our_ships) {
                                if within_detonation_range(ship, mother) && ship.ship_id != mother.ship_id {
                                    loss_ratio = 1.;
                                }
                            };
                            if  1.1 * kill_ratio > loss_ratio {
                                return vec![Command::Detonate(ship.ship_id)]
                            }
                        }
                    }
                }

                // Orbit
                if let Some(resources) = &ship.resources {
                    if resources.fuel > 0 {
                        let mut drift_measure = {
                            use rand::seq::SliceRandom;
                            use rand::thread_rng;
                            use rand_core::RngCore;
                            let mut rng = thread_rng();
                            if ship.heat < 60 || rng.next_u32() % 100 < 90 {
                                Orbiting::goodness_of_drift_from(&sv, static_game_info.planet_radius)
                            } else {
                                // Boogie!
                                i64::MIN + 1
                            }
                        };
                        let (nonzero_thrust, nonzero_measure) =
                            Orbiting::get_best_nonzero_thrust(&sv, static_game_info.planet_radius);
                        let best_thrust = 
                            if nonzero_measure > drift_measure {
                                nonzero_thrust
                            } else {
                                sim::XY { x: 0, y: 0 }
                            };
                        if best_thrust != (sim::XY { x: 0, y: 0 }) {
                            return vec![Command::Accelerate(ship.ship_id, best_thrust.into())]
                        }
                    }
                }
            }
            _ => {
                error!("Error in survivor ai: no static game info or game state");
            }
        }
        vec![]
    }
}


fn initial_resources(player_key: i64, game_response: Option<GameResponse>) -> (i64,i64,i64,i64) {
    if let Some(game_response) = game_response {
        match our_role(&game_response) {
            Some(Role::Attacker) => {
                match get_max_resources(&game_response) {
                    Some(max_resources) => {
                        let fuel_min = 50;
                        let cooling_min = 4;
                        let free = max_resources - fuel_min - cooling_min * PARAM_MULT.2;
                        let clones  = ((free as f64 * 0.3)/(PARAM_MULT.3 as f64)) as i64;
                        let fuel = free - PARAM_MULT.3*clones - PARAM_MULT.2*cooling_min;
                        if fuel >= fuel_min {
                            (fuel, 0, cooling_min, clones)
                        } else {
                            error!("This isn't good!");
                            (max_resources-2, 0, 0, 1)
                        }
                              
                    }
                    None => (1,1,1,1)
                }
            }
            _ => { // Probably defender
                match get_max_resources(&game_response) {
                    Some(max_resources) => {
                        let clones = 10;
                        let fuel = 300;
                        let cooling =
                            (max_resources - (fuel * PARAM_MULT.0) - (clones * PARAM_MULT.3)) / PARAM_MULT.2;
                        (fuel, 0, cooling, clones)
                    }
                    None => (1,1,1,1)
                }
            }
        }
    } else {
        // During the orgs testing phase
        (1,1,1,1)
    }

}

#[derive(Default)]
struct CloneController {
    turns: u32,

}
impl AI for CloneController {
    fn step(&mut self, ships: &Vec<&Ship>, game_response: &GameResponse) -> Vec<Command> {
        let mut clones = ships.len();
        self.turns += 1;
        let our_role = our_role(&game_response).unwrap();

        let mut orbiter = Orbiting{};
        let mut cmds = vec![];
        // let mut cmds = orbiter.step(ships, &game_response);
        if let Some(sgi) = &game_response.static_game_info {
            for ship in ships {
                let mut moved = false;
                if let Some(resources) = &ship.resources {
                    if resources.clones > 1 && ship.heat <= 60 {
                        if our_role == Role::Defender {
                            const FIRST_CLONE : u32 = 0;
                            const WAIT_MORE_CLONES : u32 = 10; // TODO: Think
                            if (clones == 1 && self.turns > FIRST_CLONE) || (self.turns > WAIT_MORE_CLONES) {
                                let sv = sim::SV {
                                    s: ship.position.into(),
                                    v: ship.velocity.into(),
                                };
                                let (nonzero_thrust, nonzero_measure)
                                    = Orbiting::get_best_nonzero_thrust(&sv, sgi.planet_radius);
                                if nonzero_measure == i64::MAX {// Will we survive?
                                    clones += 1;
                                    cmds.push(Command::Clone {
                                        ship_id: ship.ship_id,
                                        fuel: resources.fuel/2,
                                        cannon: resources.cannon/2, //TODO: Better to keep cannons on one ship?
                                        cooling: resources.cooling/2,
                                        clones: resources.clones/2, //Is at least 1
                                    });
                                    cmds.push(Command::Accelerate(ship.ship_id, nonzero_thrust.into()));
                                    moved = true;
                                }
                            }
                        } else {
                            // Attacker
                            const WAIT_DRONE_CLONES : u32 = 10; // TODO: Think
                            // Is there a decent move so we can get away from our clone?
                            let sv = sim::SV {
                                s: ship.position.into(),
                                v: ship.velocity.into(),
                            };
                            if sim::survives_drift(&sv, sgi.planet_radius) > 25 { // Will the clone survive?
                                let (nonzero_thrust, nonzero_measure)
                                    = Orbiting::get_best_nonzero_thrust(&sv, sgi.planet_radius);
                                if nonzero_measure == i64::MAX {// Will we survive?
                                    if (self.turns > WAIT_DRONE_CLONES) {
                                        clones += 1;
                                        cmds.push(Command::Clone {
                                            ship_id: ship.ship_id,
                                            fuel: 2,
                                            cannon: 0,
                                            cooling: 0,
                                            clones: 1,
                                        });
                                        cmds.push(Command::Accelerate(ship.ship_id, nonzero_thrust.into()));
                                        moved = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if !moved {
                    cmds.append(&mut orbiter.step(&vec![ship], &game_response));
                }
            }
        }
        cmds
    }
}



fn run_ai(player : i32, ai: &mut dyn AI, url: &str, player_key: i64) -> Result<(), Box<dyn std::error::Error>> {
    use crate::protocol::*;

    let initial_game_response = post(&url, &join_msg(player_key))?;
    let mut game_response = try_parse_response(&initial_game_response);
    let resources = {
        let (f,ca,co,cl) = initial_resources(player_key, game_response);
        parse(&format!("[3, {}, [{}, {}, {}, {}]]", player_key, f, ca, co ,cl))
    };
    game_response = try_parse_response(&post(&url, &resources)?);

    loop {
        if player == 0 {
            if let Some(game_response) = &game_response {
                println!("Game response was:\n{:?}\ny", game_response);
                let def_ships = find_ships(&game_response, Role::Defender);
                let att_ships = find_ships(&game_response, Role::Attacker);
                info!("Number of Defenders: {} at {} positions \nNumber of Attackers: {} at {} positions",
                        def_ships.len(), unique_positions(&def_ships),
                        att_ships.len(), unique_positions(&att_ships),);
                info!("Defender Resources: {}\nAttacker Resources: {}",
                         total_resources(&def_ships),
                         total_resources(&att_ships));
                info!("Attacker Mother ship: {:?}\n\n",
                         mother_ship(&att_ships));
            }
        }
        let cmds = match game_response {
            None => vec![],
            Some(game_response) => {
                let our_role = our_role(&game_response).unwrap();
                let ships = find_ships(&game_response, our_role);
                ai.step(&ships, &game_response)
            },
        };
        let mut commands = vnil();
        for cmd in cmds {
            commands = vcons(flatten_command(cmd), commands);
        }

        let request = vcons(vi64(4), vcons(vi64(player_key), vcons(commands, vnil())));

        game_response = try_parse_response(&post(url, &request)?);
    }
}

fn player_key(tree: &ValueTree) -> Result<i64, Box<dyn std::error::Error>> {
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
    println!(
        "initiate multiplayer game; player keys: {} and {}",
        player_key1, player_key2
    );

    let url1 = String::from(url);
    let thr1 = std::thread::spawn(move || {
        let err = run_ai(0, &mut *ai1, &url1, player_key1);
        println!("an error occured while running AI 1: {:?}", err);
    });
    let url2 = String::from(url);
    let thr2 = std::thread::spawn(move || {
        let err = run_ai(1, &mut *ai2, &url2, player_key2);
        println!("an error occured while running AI 2: {:?}", err);
    });

    thr1.join();
    thr2.join();

    Ok(())
}

fn get_ai(ai_str: Option<String>) -> Option<Box<dyn AI + Send>> {
    match ai_str {
        Some(ai_str) => match ai_str.as_ref() {
            "orbiting" => Some(Box::from(Orbiting {})),
            // "shoot" => Some(Box::from(Shoot {})),
            "clones" => Some(Box::from(CloneController::default())),
            _ => {
                println!("unknown ai {}, using orbiting", ai_str);
                Some(Box::from(Orbiting {}))
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
        let mut ai1 = get_ai(ai1).unwrap_or(Box::from(CloneController::default()));
        let mut ai2 = get_ai(ai2);

        match ai2 {
            Some(mut ai2) => run_ais(ai1, ai2, &url)?,
            None => run_ai(0, &mut *ai1, &url, player_key)?,
        }
    }

    Ok(())
}
