use crate::edge_view;
use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
use futures_util::{ SinkExt, StreamExt };
use crate::messages;
use messages::{
    Account,
    EdgeViewClaims,
    GetMessagesRequest,
    GetUsersRequest,
    RealmAccess,
    RealmManagement,
    ResourceAccess,
    SearchMessagesRequest,
    SendNewMessageRequest,
};
use std::{thread, time};
use thread_id;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    tungstenite::{
        client::IntoClientRequest, http::HeaderValue, protocol::{CloseFrame, Message},
        protocol::frame::coding::CloseCode,
    },
    WebSocketStream,
};
use tracing::{event, Level};
use uuid::Uuid;

pub const SERVER_PORT: u16 = 7878;
const TEST_DOMAIN: &str = "chatsurferxmppunclass";
const TEST_ROOM: &str = "edge-view-test-room";

pub fn debug(message: String) {
    event!(Level::DEBUG, "Thread {}: {}", thread_id::get(), message);
}

pub fn error(message: String) {
    event!(Level::ERROR, "Thread {}: {}", thread_id::get(), message);
}

fn build_test_claim() -> EdgeViewClaims {
    EdgeViewClaims {
        exp:                    jsonwebtoken::get_current_timestamp() + time::Duration::from_secs(3600).as_secs(),
        iat:                    jsonwebtoken::get_current_timestamp(),
        auth_time:              jsonwebtoken::get_current_timestamp(),
        jti:                    String::from("e5f3e658-629a-42ff-a63f-20a50afa61d6"),
        iss:                    String::from("https://app.fmvedgeview.net/keycloak/auth/realms/fmv"),
        aud:                    None,
        sub:                    String::from("6e4b6e86-030b-41ed-90ab-c05325526a06"),
        typ:                    String::from("Bearer"),
        azp:                    String::from("edge-view-ui"),
        nonce:                  String::from(Uuid::new_v4()),
        session_state:          String::from(Uuid::new_v4()),
        acr:                    String::from("1"),
        allowed_origins:        vec![
            String::from("http://0.0.0.0"),
            String::from("https://app.fmvedgeview.net"),
        ],
        realm_access:           RealmAccess {
            roles:              vec![
                String::from("authenticated user"),
            ],
        },
        resource_access:        ResourceAccess {
            realm_management:   RealmManagement {
                roles:          vec![String::from("view-users"),],
            },
            account:            Account {
                roles:          vec![String::from("view-profile"),],
            },
        },
        scope:                  String::from("openid email profile"),
        sid:                    Uuid::new_v4(),
        email_verified:         true,
        name:                   String::from("Austin Farrell"),
        preferred_username:     String::from("austin.farrell@ninehilltech.com"),
        given_name:             String::from("Austin"),
        family_name:            String::from("Farrell"),
        email:                  String::from("austin.farrell@ninehilltech.com"),
    }
}

pub fn build_users_request() -> String {
    let get_users_request: GetUsersRequest = GetUsersRequest {
        domain_id: String::from(TEST_DOMAIN),
        room_name: String::from(TEST_ROOM)
    };

    serde_json::to_string(&get_users_request).unwrap()
} // end build_users_request

fn build_jwt(alg: Algorithm) -> String {
    let header = Header::new(alg);
    let claims = build_test_claim();

    // Construct the JWT.
    let jwt = encode(
        &header,
        &claims,
        &EncodingKey::from_secret("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzq/jsj5MTmOA9sW4YBJpv16yLPvznKLj3UqNXQ17WhukP5wu6GQyHMUSqNV8CAqGEA8TJpoQcpTCs8iaKxpfF1yORKdeuvCa/aJZpOw6TwsJZa1OWLONyJnOuPeZZNDUn+D7as+tS9ws7UP3AtROO8hkMS7+B3C90eXTWhZnkzEDSfDmfUxPMvYH/5yGUI4AtzbAGPMwiDOXOguXUSkV5TP7RXTZqrgHp3yvzBsbaWtjW9r4tfzXRHuGFXhlEgBdsBIzupaXrpfqIjHQXDhJ1NnI6KOQUTDi5t3VOhfZ8z6WXMPdqi/pvyzTenAshvoTR2rEti6KyLqwTdW6y1KFVQIDAQAB".as_ref())).unwrap();

    jwt
} // end build_jwt

pub async fn ws_connect(
    server_port:    u16,
    jwt_alg:        Algorithm,
    path:           &str,
) -> Option<WebSocketStream<TcpStream>> {

    let url = ("localhost", server_port);
    let auth_token: HeaderValue = format!("Bearer {}", build_jwt(jwt_alg)).parse().unwrap();

    let mut auth_request = format!("ws://localhost:{}{}",
            server_port,
            path)
        .into_client_request()
        .unwrap();
    
    event!(Level::TRACE, "Authorization header: {:?}", auth_token);

    auth_request
        .headers_mut()
        .insert("Authorization", auth_token);

    match TcpStream::connect(url).await {
        Ok(stream) => {
            
            let (socket, _) = client_async(
                auth_request,
                stream
            ).await.expect("Failed to connect");

            std::thread::sleep(time::Duration::from_millis(3000));

            Some(socket)
        }
        Err(e) => {
            error(format!("Could not connect to server: {}", e));
            None
        }
    }
} // end ws_connect

async fn ws_connect_send(
    server_port:    u16,
    jwt_alg:        Algorithm,
    path:           &str,
    message:        String,
) -> Option<Message> {

    let socket = ws_connect(server_port, jwt_alg, path).await;

    match socket {
        Some(socket) => {
            let (mut write, mut read) = socket.split();

            // Send the request.
            let result = match write.send(Message::Text(message)).await {
                Ok(()) => {
                    event!(Level::DEBUG, "Attempting to read response from {} endpoint:", path);
                    match read.next().await {
                        Some(response) => {
                            event!(Level::DEBUG, "We received a response!");
        
                            match response {
                                Ok(payload) => Some(payload),
                                Err(e) => {
                                    event!(Level::ERROR, "{}", e);
                                    None
                                }
                            }
                        }
                        None => None
                    }
                }
                Err(e) => {
                    event!(Level::ERROR, "Could not send the request: {}", e);
                    None
                }
            };
        
            let close_frame = CloseFrame {
                code: CloseCode::Normal,
                reason: std::borrow::Cow::Owned(String::from("Complete"))
            };
        
            match write.send(Message::Close(Some(close_frame))).await {
                Ok(()) => {
                    event!(Level::DEBUG, "Successfully sent the closing frame.");
                }
                Err(e) => {
                    event!(Level::ERROR, "Could not send the closing frame: {}", e);
                }
            }
        
            result
        }
        None => {
            error(format!("No WebSocket connection."));
            None
        }
    }
} // end ws_connect_send

pub async fn spin_client(endpoint: String) {

    match edge_view::client::ws_connect(
        edge_view::client::SERVER_PORT,
        Algorithm::HS256,
        endpoint.as_str()
    ).await {
        Some(client) => {
            event!(Level::DEBUG, "We successfully connected to the server!  Moving into the spin loop");

            loop {
                // We will stay here forever to keep the server connection
                // live.
                thread::sleep(time::Duration::from_secs(10));
                debug(format!("spinning on {}", endpoint));
            }
        }
        None => {
            error(format!("An error occurred connecting to the server. Killing the thread."));
        }
    }
} // end spin_client

pub async fn test_get_users() {
    event!(Level::INFO, "Beginning Get Users Test.");

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/users",
        build_users_request()).await;

    match response {
        Some(payload) => {

            debug(format!("{}", payload));
            event!(Level::INFO, "Get Users Test passed!");
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            error(format!("Get Users Test Failed!"));
        }
    }
} // end test_get_users

pub async fn test_get_users_and_listen() {
    event!(Level::INFO, "Beginning Get Users and Listen Test.");

    let socket = ws_connect(7878, Algorithm::HS256, "/users").await;

    if let Some(mut socket) = socket {

        if let Ok(()) = socket.send(Message::Text(build_users_request())).await {

            while let Some(update) = socket.next().await {

                match update {

                    Ok(Message::Text(payload)) => {
        
                        event!(Level::DEBUG, "{}", payload);
                    }
                    Ok(Message::Close(_)) => {
                        event!(Level::DEBUG,
                            "{}: Received a Closing frame.",
                            std::process::id()
                        );
                        break;
                    }
                    Ok(_) => {
                        event!(Level::DEBUG,
                            "{}: We received an unknown message. Ignoring.",
                            std::process::id()
                        );
                    }
                    Err(e) => {
                        event!(Level::ERROR,
                            "{}: An error occurred receiving from the WebSocket: {:#?}",
                            std::process::id(),
                            e
                        );
                    }
                }
            }
        }
    }
}