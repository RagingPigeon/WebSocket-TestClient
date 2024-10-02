
#[allow(non_snake_case)]
mod ChatSurfer;
use futures_util::{ SinkExt, StreamExt };
use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
mod messages;
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
use serde_json;
use std::time;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    tungstenite::{
        protocol::Message,
        client::IntoClientRequest,
        http::HeaderValue,
    },
};
use tracing::{ event, Level };
use tracing_subscriber;
use uuid::Uuid;


const TEST_DOMAIN: &str = "chatsurferxmppunclass";
const TEST_ROOM: &str = "Test_Room";

fn get_users_message() -> String {
    let get_users_request: GetUsersRequest = GetUsersRequest {
        domainId: String::from(TEST_DOMAIN),
        roomName: String::from(TEST_ROOM)
    };

    serde_json::to_string(&get_users_request).unwrap()
}

fn build_messages_request() -> String {
    let messages_request: GetMessagesRequest = GetMessagesRequest {
        domainId: String::from(TEST_DOMAIN),
        roomName: String::from(TEST_ROOM),
    };

    serde_json::to_string(&messages_request).unwrap()
}

fn build_search_messages_request() -> String {
    let request: SearchMessagesRequest = SearchMessagesRequest {
        domainId: String::from(TEST_DOMAIN),
        roomName: String::from(TEST_ROOM),
        keywords: vec!(String::from("test_keyword")),
    };

    serde_json::to_string(&request).unwrap()
} // end build_search_messages_request

fn build_new_message_request() -> String {
    let request: SendNewMessageRequest = SendNewMessageRequest {
        domainId: String::from(TEST_DOMAIN),
        roomName: String::from(TEST_ROOM),
        text: String::from("I'm a new message")
    };

    request.to_json()
} // end build_new_message_request

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

async fn ws_connect_send
(
    server_port:    u16,
    jwt_alg:        Algorithm,
    path:           &str,
    message:        String,
) -> Option<Message> {
    let url = ("localhost", server_port);
    let auth_token: HeaderValue = format!("Bearer {}", build_jwt(jwt_alg)).parse().unwrap();
    
    let mut auth_request = format!("ws://localhost:{}{}",
            server_port,
            path)
        .into_client_request()
        .unwrap();

    auth_request
        .headers_mut()
        .insert("Authorization", auth_token);

    let stream = TcpStream::connect(url).await.unwrap();

    let (socket, _) = client_async(
        auth_request,
        stream
    ).await.expect("Failed to connect");

    let (mut write, mut read) = socket.split();

    // Send the request.
    match write.send(Message::Text(message)).await {
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
    }
} // end ws_connect_send

async fn test_send_new_message() {

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/send",
        build_new_message_request()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Send New Message Test passed!");
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Send New Message Test Failed!");
        }
    }
} // end test_send_new_message

async fn test_get_users() {

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/users",
        get_users_message()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Get Users Test passed!");
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Get Users Test Failed!");
        }
    }
} // end test_get_users

async fn test_get_messages() {
    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/messages",
        build_messages_request()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Get Messages Test passed!");
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Get Messages Test Failed!");
        }
    }
} // end test_get_messages

async fn test_search_messages() {
    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/search",
        build_search_messages_request()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Search Messages Test passed!");
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Search Messages Test Failed!");
        }
    }
} // end test_search_messages

#[tokio::main]
async fn main() {
    // Set up the logging subscriber.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    //======================================================================
    // Send New Message Endpoint
    test_send_new_message().await;

    //======================================================================
    //Get Users Endpoint
    test_get_users().await;
    
    //======================================================================
    // Get Messages Endpoint
    test_get_messages().await;

    //======================================================================
    // Search Messages Endpoint
    test_search_messages().await;
}