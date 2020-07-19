use crate::value_tree::*;
use log::*;
use std::convert::TryInto;
use std::env;
use std::io::BufRead;

use crate::submission::*;

type ShipId = i64;

// Param1 = Fuel
// Param2 = Cannon power
// Param3 = ???
// Param4 = ???
const PARAM_MULT : (u8,u8,u8,u8) = (1, 4, 12, 2);


// OBSERVATIONS ON COMMANDS

// Shoot:
// (2, shipId, target, cannon-power?)



// OBSERVATIONS ON POST RESPONSES
// [1, 1, [256, 0, [512, 1, 64], [16, 128], [1, 1, 1, 1]], [1, [16, 128], [[[1, 0, (47, -33), (-1, 0), [1, 1, 1, 1], 0, 64, 1], []], [[0, 1, (-47, 33), (1, 0), [254, 64, 0, 1], 64, 64, 1], [[2, (47, -33), 64, 0, 4]]]]]]
//                                                                                                                                                                                X Ammo used (shots * strength)

#[derive(Debug, PartialEq, Eq)]
pub enum GameStage {
    NotStarted,
    Started,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Attacker,
    Defender,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameResponse {
    pub game_stage: GameStage,
    pub static_game_info: Option<StaticGameInfo>,
    pub game_state: Option<GameState>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StaticGameInfo {
    pub max_steps: i64,
    pub planet_radius: i64,
    pub game_radius: i64,
    pub role: Role,
    pub max_resources: i64,
    pub opponent_resources: Option<Resources>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameState {
    pub game_tick: i64,
    pub ships: Vec<Ship>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ship {
    pub role: Role,
    pub ship_id: ShipId,
    pub position: (i64, i64),
    pub velocity: (i64, i64),
    pub resources: Option<Resources>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Resources {
    pub fuel: i64,
}

pub enum Command {
    Accelerate(ShipId, (i64, i64)),
    Detonate(ShipId),
    Shoot(ShipId, (i64, i64), i64),
}

pub fn as_int(field: &str, tree: &ValueTree) -> Result<i64, Box<dyn std::error::Error>> {
    match tree {
        ValueTree::VInt(i) => {
            let res = i.try_into()?;
            Ok(res)
        }
        _ => Err(Box::from(format!("expected an int for {}", field))),
    }
}

fn parse_game_stage(tree: &ValueTree) -> Result<GameStage, Box<dyn std::error::Error>> {
    use GameStage::*;

    match as_int("game stage", tree)? {
        0 => Ok(NotStarted),
        1 => Ok(Started),
        2 => Ok(Finished),
        _ => Err(Box::from("unexpected game stage")),
    }
}

fn parse_role(tree: &ValueTree) -> Result<Role, Box<dyn std::error::Error>> {
    use Role::*;

    match as_int("role", tree)? {
        0 => Ok(Attacker),
        1 => Ok(Defender),
        _ => Err(Box::from("unexpected role")),
    }
}

pub fn parse_game_response(tree: &ValueTree) -> Result<GameResponse, Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() != 4 {
        Err(Box::from("unexpected structure of game response"))
    } else {
        Ok(GameResponse {
            game_stage: parse_game_stage(response[1])?,
            static_game_info: parse_static_game_info(response[2])?,
            game_state: parse_game_state(response[3])?,
        })
    }
}

fn parse_radius(tree: &ValueTree) -> Result<(i64, i64), Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() != 2 {
        Err(Box::from(
            "unexpected structure of radius parameters in game response",
        ))
    } else {
        Ok((as_int("planet_radius", response[0])?, as_int("game radius", response[1])?))
    }
}

fn parse_static_game_info(tree: &ValueTree) -> Result<Option<StaticGameInfo>, Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() == 0 {
        Ok(None)
    } else if response.len() != 5 {
        Err(Box::from("unexpected structure of static game info"))
    } else {
        let (planet_radius, game_radius) = parse_radius(response[3])?;
        let inner_list = to_native_list(&response[2]);

        if (inner_list.len() != 3) {
            Err(Box::from("unexpected structure of static game info"))
        } else {
            Ok(Some(StaticGameInfo {
                max_steps: as_int("steps", response[0])?,
                planet_radius: planet_radius,
                game_radius: game_radius,
                role: parse_role(response[1])?,
                max_resources: as_int("max resources", inner_list[0])?,
                opponent_resources: parse_resources(response[4])?,
            }))
        }
    }
}

fn parse_game_state(tree: &ValueTree) -> Result<Option<GameState>, Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() == 0 {
        Ok(None)
    } else if response.len() != 3 {
        Err(Box::from("unexpected structure of game state"))
    } else {
        Ok(Some(GameState {
            game_tick: as_int("tick", response[0])?,
            ships: parse_ships(response[2])?,
        }))
    }
}

fn parse_ships(tree: &ValueTree) -> Result<Vec<Ship>, Box<dyn std::error::Error>> {
    let mut ships = vec![];

    for ship in to_native_list(&tree) {
        ships.push(parse_ship(ship)?);
    }

    Ok(ships)
}

fn parse_tuple(tree: &ValueTree) -> Result<(i64, i64), Box<dyn std::error::Error>> {
    use ValueTree::*;

    match tree {
        VCons(args) => match args.as_ref() {
            (x, y) => Ok((as_int("coord 1", x)?, as_int("coord 2", y)?)),
        },
        _ => Err(Box::from("unexpected structure of tuple")),
    }
}

fn parse_ship(tree: &ValueTree) -> Result<Ship, Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() != 2 {
        Err(Box::from("unexpected structure of ship"))
    } else {
        let ship = to_native_list(&response[0]);
        
        if (ship.len() != 8) {
            Err(Box::from("unexpected structure of ship"))
        } else {
            Ok(Ship {
                role: parse_role(ship[0])?,
                ship_id: as_int("ship id", ship[1])?,
                position: parse_tuple(ship[2])?,
                velocity: parse_tuple(ship[3])?,
                resources: parse_resources(ship[4])?,
            })
        }
    }
}

fn parse_resources(tree: &ValueTree) -> Result<Option<Resources>, Box<dyn std::error::Error>> {
    let response = to_native_list(&tree);

    if response.len() == 0 {
        Ok(None)
    } else if response.len() != 4 {
        Err(Box::from("unexpected resource structure"))
    } else {
        Ok(Some(Resources {
            fuel: as_int("fuel", response[0])?,
        }))
    }
}

pub fn flatten_command(cmd: Command) -> ValueTree {
    use Command::*;
    use crate::submission::*;

    match cmd {
        Accelerate(id, (vx, vy)) => parse(&format!("[0, {}, ({}, {})]", id, vx, vy)),
        Detonate(id) => parse(&format!("[1, {}]", id)),
        Shoot(id, (x, y), intensity) => parse(&format!("[2, {}, ({}, {}), {}]", id, x, y, intensity))
    }
}

pub fn get_max_resources(game_response: Option<GameResponse>) -> Option<i64> {
    game_response.and_then(|x| x.static_game_info.and_then(|y| Some(y.max_resources)))
}

#[cfg(test)]
mod test {
    use super::*;
    use super::GameStage::*;
    use super::Role::*;

    #[test]
    fn test_parse_started_game_response() {
        let response = "[1, 1, [256, 0, [512, 1, 64], [16, 128], [326, 0, 10, 1]], [0, [16, 128], [[[1, 0, (-48, -12), (0, 0), [326, 0, 10, 1], 0, 64, 2], []], [[0, 1, (48, 12), (0, 0), [1, 1, 1, 1], 0, 64, 1], []]]]]";
        let tree = crate::value_tree::parse_value_tree(response).unwrap();

        let result = parse_game_response(&tree).unwrap();
        let static_game_info = result.static_game_info.unwrap();

        assert_eq!(result.game_stage, Started);

        assert_eq!(static_game_info.game_radius, 128);
        assert_eq!(static_game_info.planet_radius, 16);
        assert_eq!(static_game_info.max_steps, 256);
        assert_eq!(static_game_info.role, Attacker);
        assert_eq!(static_game_info.max_resources, 512);
        assert_eq!(static_game_info.opponent_resources.unwrap().fuel, 326);

        let game_state = result.game_state.unwrap();

        assert_eq!(game_state.game_tick, 0);
        assert_eq!(game_state.ships.len(), 2);
        assert_eq!(game_state.ships[0].position, (-48, -12));
        assert_eq!(game_state.ships[0].role, Defender);
        assert_eq!(game_state.ships[0].ship_id, 0);
        assert_eq!(game_state.ships[0].velocity, (0, 0));

        let resources = game_state.ships[0].resources.as_ref().unwrap();

        assert_eq!(resources.fuel, 326);
    }

    #[test]
    fn test_parse_joined_game_response() {
        let response = "[1, 0, [256, 0, [512, 1, 64], [16, 128], [326, 0, 10, 1]], []]";
        let tree = crate::value_tree::parse_value_tree(response).unwrap();

        let result = parse_game_response(&tree).unwrap();
        let static_game_info = result.static_game_info.unwrap();

        assert_eq!(result.game_stage, NotStarted);

        assert_eq!(static_game_info.game_radius, 128);
        assert_eq!(static_game_info.planet_radius, 16);
        assert_eq!(static_game_info.max_steps, 256);
        assert_eq!(static_game_info.role, Attacker);
        assert_eq!(static_game_info.max_resources, 512);
        assert_eq!(static_game_info.opponent_resources.unwrap().fuel, 326);

        assert_eq!(result.game_state, None);
    }

    #[test]
    fn test_parse_finished_game_response() {
        let response = "[1, 2, [], []]";
        let tree = crate::value_tree::parse_value_tree(response).unwrap();

        let result = parse_game_response(&tree).unwrap();

        assert_eq!(result.game_stage, Finished);
        assert_eq!(result.static_game_info, None);
        assert_eq!(result.game_state, None);
    }

    #[test]
    fn test_parse_not_started_game_response() {
        let response = "[1, 0, [256, 1, [448, 1, 64], [16, 128], []], []]";
        let tree = crate::value_tree::parse_value_tree(response).unwrap();

        let result = parse_game_response(&tree).unwrap();

        assert_eq!(result.game_stage, NotStarted);
    }
}
