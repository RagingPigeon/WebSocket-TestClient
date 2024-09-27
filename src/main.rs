
// use http::{
//     HeaderValue,
//     Response,
//     Request,
// };
use futures::stream::{
    SplitSink,
    SplitStream
};
use futures_util::{future, pin_mut, SinkExt, StreamExt};
use jsonwebtoken::{
    Algorithm,
    decode,
    DecodingKey,
    encode,
    EncodingKey,
    Header,
    Validation,
};
use serde::{ Deserialize, Serialize };
use serde_json;
use std::{
    fmt,
    time,
    borrow::Cow,
    str::FromStr,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    connect_async,
    tungstenite::{
        Error,
        Result,
        protocol::Message,
        client::IntoClientRequest,
        http::HeaderValue,
    },
    MaybeTlsStream,
    WebSocketStream,
    //net::TcpStream,
};
use tracing::{event, span, Level};
use tracing_subscriber;
// use tungstenite::{
//     client,
//     connect,
//     protocol,
//     WebSocket,
//     stream::MaybeTlsStream,
    
// };
use url::Url;
use uuid::Uuid;
mod messages;
use messages::{
    Account, EdgeViewClaims, GetMessagesRequest, GetUsersRequest, GetUsersResponse, RealmAccess, RealmManagement, ResourceAccess, SearchMessagesRequest, SendNewMessageRequest
};

mod ChatSurfer;
use ChatSurfer::messages as cs_messages;

const LOOP_LIMIT: i32 = 1;
const TEST_DOMAIN: &str = "chatsurferxmppunclass";
const TEST_ROOM: &str = "Test_Room";

const TEST_JWT: &str = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJvOWl5NU0xTm5vVjB2YTF2cl9aVnRfaW9SZHRFcThaWVFYRVdPMEh6a19ZIn0.eyJleHAiOjE3MjcyMDM2OTcsImlhdCI6MTcyNzIwMzU3NywiYXV0aF90aW1lIjoxNzI3MjAzNTc2LCJqdGkiOiJlNWYzZTY1OC02MjlhLTQyZmYtYTYzZi0yMGE1MGFmYTYxZDYiLCJpc3MiOiJodHRwczovL2FwcC5mbXZlZGdldmlldy5uZXQva2V5Y2xvYWsvYXV0aC9yZWFsbXMvZm12IiwiYXVkIjpbInJlYWxtLW1hbmFnZW1lbnQiLCJhY2NvdW50Il0sInN1YiI6IjZlNGI2ZTg2LTAzMGItNDFlZC05MGFiLWMwNTMyNTUyNmEwNiIsInR5cCI6IkJlYXJlciIsImF6cCI6ImVkZ2Utdmlldy11aSIsIm5vbmNlIjoiMTI0YjU5NTItYzFjYy00MWRlLWE5ZTAtZTUzOTljNGM3M2JjIiwic2Vzc2lvbl9zdGF0ZSI6IjJmMDFhMGI4LTE4Y2MtNDJiYS1hMzkyLWQzNGNmMTdiNWI5ZCIsImFjciI6IjEiLCJhbGxvd2VkLW9yaWdpbnMiOlsiaHR0cDovLzAuMC4wLjAiLCIwLjAuMC4wIiwiaHR0cHM6Ly9hcHAuZm12ZWRnZXZpZXcubmV0IiwiaHR0cDovLzAuMC4wLjAvKiIsImxvY2FsaG9zdC8qIiwiKiIsImh0dHA6Ly9sb2NhbGhvc3QiXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbInN0cmVhbSBtYW5hZ2VyIiwiYXV0aGVudGljYXRlZCB1c2VyIiwiRk1WIHVzZXIgYWRtaW4iLCJhZG1pbiIsImdyZS1ub2RlLW1hbmFnZXIiXX0sInJlc291cmNlX2FjY2VzcyI6eyJyZWFsbS1tYW5hZ2VtZW50Ijp7InJvbGVzIjpbImltcGVyc29uYXRpb24iLCJtYW5hZ2UtdXNlcnMiLCJ2aWV3LXVzZXJzIiwidmlldy1hdXRob3JpemF0aW9uIiwicXVlcnktZ3JvdXBzIiwicXVlcnktdXNlcnMiXX0sImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJ2aWV3LWFwcGxpY2F0aW9ucyIsIm1hbmFnZS1hY2NvdW50LWxpbmtzIiwiZGVsZXRlLWFjY291bnQiLCJ2aWV3LXByb2ZpbGUiXX19LCJzY29wZSI6Im9wZW5pZCBlbWFpbCBwcm9maWxlIiwic2lkIjoiMmYwMWEwYjgtMThjYy00MmJhLWEzOTItZDM0Y2YxN2I1YjlkIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5hbWUiOiJKb2huIERvZSIsInByZWZlcnJlZF91c2VybmFtZSI6ImFkbWluQG5pbmVoaWxsdGVjaC5jb20iLCJnaXZlbl9uYW1lIjoiSm9obiIsImZhbWlseV9uYW1lIjoiRG9lIiwiZW1haWwiOiJhZG1pbkBuaW5laGlsbHRlY2guY29tIn0.qT9RVgggWe4-KiNgWvbVDi6zNIFhk33TbWp4hhNI20_15uXQz-B-eGuy82ybzeH0JX4d7P1hAWLXKM6Zb7WD690UyVfqYAtCA13u1dzHxE_3GzkXf4gs6vCTHGw2r2WEu0XqTZUylUl4g6jB0HQ1EUFb6ehGvtSX9KCoMKLSQYO1QtpyXW7cl0HLqRFhKt6O0zoLtb_kgIR5ccL7_eLGUlSGcg3EPatOJRdaObnytnSU2HbAPESrAFdsj-ZFVjpuNS06cB-63hZoMQqV9hTglFt8-YflsEpL0UNAiKD0efRY7I6NSRF7LXDSRT1ZMOzZxv9a7Ah2Xvi_Ftt55srOeA";

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
        keywords: vec!(String::from("test_keyword"), String::from("Austin")),
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
                        Err(e) => None
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
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut loop_count: i32 = 0;
    let mut search_messages_response_received: bool = false;

    let search_messages_url = Url::parse("wss://localhost:7878/search").unwrap();
    //let search_messages_url = Url::parse("wss://aac90a4180f5e47b9ba591836c7f4829-839660979210620c.elb.us-gov-west-1.amazonaws.com/search").unwrap();


    // Connect to the WebSocket.

    //let (mut messages_socket, messages_response) = connect(messages_url).expect("Can't connect");

    //let (mut search_messages_socket, search_messages_response) = connect(search_messages_url).expect("Can't connect");

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
    
    // High level loop to iterate over each endpoint we're testing.
    while loop_count < LOOP_LIMIT {
        
        
    
        //     match messages_socket.read_message() {
        //         Ok(message) => {
        //             let response = message.to_text();

        //             match response {
        //                 Ok(get_messages_request) => {
        //                     let response: cs_messages::GetChatMessagesResponse = serde_json::from_str(get_messages_request).unwrap();
        //                     let pretty_json = serde_json::to_string_pretty(&response).unwrap();

        //                     //let pretty_json = serde_json::to_string(&get_messages_request).unwrap();
        //                     event!(Level::DEBUG, "Received from server: {}", pretty_json);
        
        
        
        
        // search_messages_socket.write_message(Message::Text(build_search_messages_request())).unwrap();

        // while search_messages_response_received == false {
        //     event!(Level::DEBUG, "Attempting to read response from Search Messages endpoint:");
    
        //     match search_messages_socket.read_message() {
        //         Ok(message) => {
        //             event!(Level::DEBUG, "Received from server: {}", message);
        //             search_messages_response_received = true;
        //         },
        //         Err(error) => event!(Level::DEBUG, "Error receiving message: {}", error),
        //     };
        // }
        // search_messages_response_received = false;

        

        loop_count += 1;
    }

    // let closing_code: Option<protocol::frame::CloseFrame> = Some(protocol::frame::CloseFrame {
    //     code: protocol::frame::coding::CloseCode::Normal,
    //     reason: Cow::from("hello")
    // });

    // match users_socket.close(closing_code) {
    //     Ok(_) => event!(Level::DEBUG, "Successfully closed the WebSocket!"),
    //     Err(error) => event!(Level::DEBUG, "Error closing the WebSocket: {}", error),
    // }
}