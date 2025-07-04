use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use pretty_env_logger;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use thiserror::Error;
use tokio::time::Duration;
use warp::filters::ws::{Message, WebSocket};
use warp::http::StatusCode;
use warp::Filter;


const API_TOKEN: &str = "6smtr8ke3s7yq63f3zug9z3th";

#[derive(StructOpt, Debug)]
#[structopt(name = "websocket-server")]
struct Opts {
    #[structopt(short, long, default_value = "7878")]
    port: u16,
}

#[derive(Deserialize, Debug)]
struct WsRequest{
    kind: String,
    token: String,
}


#[derive(Serialize, Debug)]
struct ApiErrorResult {
    details : String,
}

#[derive(Serialize, Debug)]
struct WsResult {
    status: String,
    response: String,
}


#[derive(Error, Debug)]
enum ApiErrors {
    #[error("user not authorized")]
    NotAuthorized(String),
}


impl warp::reject::Reject for ApiErrors {}


#[tokio::main]
async fn main () {
    pretty_env_logger::init();
    let opts = Opts::from_args();

    info!("Initiating WebSocket server on port {}", opts.port);

    let health_check = warp::path!("health-check").map(||format!("Server is running!"));
    let ws = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws|{
        info!("WebSocket connection request recieved");
        ws.on_upgrade(handle_ws_client)
    });

    let ws_auth = warp::path("ws-private")
    .and(ensure_authentication().await)
    .and(warp::ws())
    .map(|_user:String, ws:warp::ws::Ws|{
        info!("WebSocket connection request recieved for private endpoint");
        ws.on_upgrade(handle_ws_client)
    });

    let routes = health_check.or(ws).or(ws_auth)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);

    warp::serve(routes)
        .run(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), opts.port))
        .await;
    info!("WebSocket server stopped");
    
}


async fn handle_rejection(err: warp::reject::Rejection) -> std::result::Result<impl warp::reply::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<ApiErrors>() {
        match e {
            ApiErrors::NotAuthorized(_error_message) => {
                code = StatusCode::UNAUTHORIZED;
                message = "Action not authorized";
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed";
    } else {
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error";
    }

    let json = warp::reply::json(&ApiErrorResult { details: message.into() });

    Ok(warp::reply::with_status(json, code))
}


async fn ensure_authentication() -> impl Filter<Extract = (String,), Error = warp::reject::Rejection> + Clone {
    warp::header::optional::<String>("Authorization").and_then(|auth_header: Option<String>| async move {
        info!("doing dummy validation of authorization header");
        if let Some(header) = auth_header {
            info!("got auth header, verifying: {}", header);
            let parts: Vec<&str> = header.split(" ").collect();
            if parts.len() == 2 && parts[0] == "Token" && parts[1] == API_TOKEN {
                return Ok("Existing user".to_string());
            }
        }

        Err(warp::reject::custom(ApiErrors::NotAuthorized(
            "not authorized".to_string(),
        )))
    })
}
async fn handle_ws_client(websocket: warp:: ws:: WebSocket){
    let(mut sender, mut receiver) = websocket.split();
    while let Some(body) = receiver.next().await{
        let message = match body 
        {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        };
        
        handle_websocket_message(message, &mut sender).await;
    }

    info!("WebSocket client disconnected");
}


async fn handle_websocket_message(message: Message, sender: &mut SplitSink<WebSocket, Message>) {
    let msg = if let Ok(s) = message.to_str(){
        s
    }else{
        error!("ping-pong");
        return;
    };

    let req: WsRequest = serde_json::from_str(msg).unwrap();
    info!("got request {} with token {}", req.kind, req.token);

    std::thread::sleep(Duration::new(1, 0));

    let response = serde_json::to_string(&WsResult {
        status: "success".to_string(),
        response: "awesome message".to_string(),
    })
    .unwrap();
    sender.send(Message::text(response)).await.unwrap();
}