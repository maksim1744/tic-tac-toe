use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use rand::prelude::*;
use serde::Deserialize;
use serde_json::map::Map;
use serde_json::Value;

mod state;
mod tests;

use state::*;

const PREC_LEVELS: usize = 3;

#[derive(Deserialize)]
struct TttRequest {
    state: State,
    strategy: String,
}

fn eval_state(state: State, mem: &mut HashMap<u64, GameResult>) -> GameResult {
    let encoded = state.as_u64();
    if let Some(result) = mem.get(&encoded) {
        return *result;
    }
    let result = state.get_result();
    if result != GameResult::Draw {
        mem.insert(encoded, result);
        return result;
    }
    let moves = state.get_moves();
    if moves.is_empty() {
        mem.insert(encoded, GameResult::Draw);
        return GameResult::Draw;
    }
    let mut other_states: Vec<State> = Vec::new();
    let mut result = GameResult::Win(3 - state.current_player);
    for mv in moves.iter() {
        let mut next_state = state.clone();
        next_state.make_move(mv);
        let cur = next_state.get_result();
        if cur == GameResult::Draw {
            other_states.push(next_state);
        } else if cur == GameResult::Win(state.current_player) {
            mem.insert(encoded, cur);
            return cur;
        }
    }
    for next_state in other_states.into_iter() {
        let cur = eval_state(next_state, mem);
        if cur == GameResult::Win(state.current_player) {
            result = cur;
            break;
        } else if cur == GameResult::Draw {
            result = cur;
        }
    }
    mem.insert(encoded, result);
    result
}

fn get_best_move(
    state: &State,
    moves: Vec<Move>,
    global_mem: &Arc<Mutex<HashMap<u64, GameResult>>>,
) -> Move {
    let mut winning: Vec<Move> = Vec::new();
    let mut drawing: Vec<Move> = Vec::new();
    let mut losing: Vec<Move> = Vec::new();
    let mut mem: HashMap<u64, GameResult> = HashMap::new();
    for mv in moves.into_iter() {
        let mut next_state = state.clone();
        next_state.make_move(&mv);
        let result = global_mem
            .lock()
            .unwrap()
            .get(&next_state.as_u64())
            .copied();
        let result = match result {
            Some(x) => x,
            None => eval_state(next_state, &mut mem),
        };
        match result {
            GameResult::Win(player) => {
                if player == state.current_player {
                    winning.push(mv);
                } else {
                    losing.push(mv);
                }
            }
            GameResult::Draw => {
                drawing.push(mv);
            }
        }
    }
    if !winning.is_empty() {
        winning[thread_rng().gen::<usize>() % winning.len()]
    } else if !drawing.is_empty() {
        drawing[thread_rng().gen::<usize>() % drawing.len()]
    } else {
        losing[thread_rng().gen::<usize>() % losing.len()]
    }
}

async fn process_request(
    req: Request<Body>,
    global_mem: Arc<Mutex<HashMap<u64, GameResult>>>,
) -> Result<Response<Body>, Infallible> {
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            let mut response = Response::new(Body::empty());
            *response.body_mut() = Body::from("Unknown error");
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    };
    let request: TttRequest = match serde_json::from_slice(&body) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            let mut response = Response::new(Body::empty());
            *response.body_mut() = Body::from("Error while parsing json");
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    };

    let strategy = request.strategy;
    let mut state = request.state;

    let mut response: Value = Value::Object(Map::new());

    let result = state.get_result();
    let moves = state.get_moves();
    if result != GameResult::Draw || moves.is_empty() {
        match result {
            GameResult::Draw => {
                response["result"] = Value::String("draw".to_string());
            }
            GameResult::Win(player) => {
                response["result"] = Value::String("win".to_string() + &player.to_string());
            }
        };
        return Ok(Response::new(Body::from(
            serde_json::to_string(&response).unwrap(),
        )));
    }

    let best_move = match strategy.as_str() {
        "random" => moves[thread_rng().gen::<usize>() % moves.len()],
        _ => get_best_move(&state, moves, &global_mem),
    };
    state.make_move(&best_move);
    let result = state.get_result();
    response["move"] = serde_json::to_value(best_move).unwrap();
    if result != GameResult::Draw || state.get_moves().is_empty() {
        match result {
            GameResult::Draw => {
                response["result"] = Value::String("draw".to_string());
            }
            GameResult::Win(player) => {
                response["result"] = Value::String("win".to_string() + &player.to_string());
            }
        };
    }
    Ok(Response::new(Body::from(
        serde_json::to_string(&response).unwrap(),
    )))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let mut mp: HashMap<u64, GameResult> = HashMap::new();
    {
        let mut tmp: HashMap<u64, GameResult> = HashMap::new();
        let mut current: Vec<State> = Vec::new();
        current.push(State::new());
        for _ in 0..PREC_LEVELS {
            let mut next: Vec<State> = Vec::new();
            for state in current.iter() {
                let moves = state.get_moves();
                for mv in moves.into_iter() {
                    let mut next_state = state.clone();
                    next_state.make_move(&mv);
                    mp.insert(
                        next_state.as_u64(),
                        eval_state(next_state.clone(), &mut tmp),
                    );
                    next.push(next_state);
                }
            }
            std::mem::swap(&mut current, &mut next);
        }
    }
    eprintln!("Global mem ready, size: {}", mp.len());
    let global_mem = Arc::new(Mutex::new(mp));

    let make_svc = make_service_fn(move |_| {
        let inner = Arc::clone(&global_mem);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let inner = Arc::clone(&inner);
                process_request(req, inner)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
