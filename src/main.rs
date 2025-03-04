
#[allow(non_snake_case)]
mod chatsurfer;
mod cli;
use dotenv::dotenv;
mod edge_view;
mod service;

use clap::Parser;
use futures_util::{ SinkExt, StreamExt };
use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
mod messages;
use messages::{
    Account, EdgeViewClaims, GetMessagesRequest, GetMessagesResponse, GetUsersRequest, GetUsersResponse, RealmAccess, RealmManagement, ResourceAccess, SearchMessagesRequest, SearchMessagesResponse, SendNewMessageRequest, SendNewMessageResponse
};
use serde_json;
use std::time;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    tungstenite::{
        client::IntoClientRequest, http::HeaderValue, protocol::{CloseFrame, Message},
        protocol::frame::coding::CloseCode,
    },
    WebSocketStream,
};
use tracing::{ event, Level };
use tracing_subscriber::{ EnvFilter, fmt, prelude::* };
use uuid::Uuid;


const TEST_DOMAIN: &str = "chatsurferxmppunclass";
const TEST_ROOM: &str = "edge-view-test-room";
const JWT_ALGORITHM: jsonwebtoken::Algorithm = Algorithm::HS256;

pub struct TestCase {
    pub server_path:    String,
    pub test_name:      String,
    pub jwt_header_alg: jsonwebtoken::Algorithm,
    pub request:        String,
    pub validator:      fn(String) -> bool,
}

fn get_users_message() -> String {
    let get_users_request: GetUsersRequest = GetUsersRequest {
        domain_id: String::from(TEST_DOMAIN),
        room_name: String::from(TEST_ROOM)
    };

    serde_json::to_string(&get_users_request).unwrap()
}

fn build_messages_request() -> String {
    let messages_request: GetMessagesRequest = GetMessagesRequest {
        domain_id: String::from(TEST_DOMAIN),
        room_name: String::from(TEST_ROOM),
    };

    serde_json::to_string(&messages_request).unwrap()
}

fn build_search_messages_request() -> String {
    let search_str: &str = "test_keyword";

    let request: SearchMessagesRequest = SearchMessagesRequest {
        domain_id: String::from(TEST_DOMAIN),
        room_name: String::from(TEST_ROOM),
        keywords: vec!(String::from(search_str)),
    };

    event!(Level::DEBUG, "Searching for messages containing {}", search_str);

    serde_json::to_string(&request).unwrap()
} // end build_search_messages_request

fn build_new_message_request() -> String {
    let request: SendNewMessageRequest = SendNewMessageRequest {
        domain_id: String::from(TEST_DOMAIN),
        room_name: String::from(TEST_ROOM),
        text: String::from("I'm a new message")
    };

    request.to_json().unwrap()
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

    std::thread::sleep(time::Duration::from_millis(3000));

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
} // end ws_connect_send

async fn test_send_new_message() -> bool {
    event!(Level::INFO, "Beginning Send New Message Test.");

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/send",
        build_new_message_request()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Send New Message Test passed!");
            true
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Send New Message Test Failed!");
            false
        }
    }


} // end test_send_new_message

async fn test_send_new_message_repeat() -> bool {
    event!(Level::INFO, "Beginning Send New Message Repeat Test.");

    let number_of_iterations: i32 = 3;
    let mut number_of_successes: i32 = 0;

    let path = "/send";
    let client_socket = edge_view::client::ws_connect(7878, Algorithm::HS256, path).await;

    let (mut write, mut read) = client_socket.unwrap().split();


    for i in 0..number_of_iterations {
        event!(Level::DEBUG, "========================================");
        event!(Level::DEBUG, "Iteration {}", i);

        match write.send(Message::Text(build_new_message_request())).await {
            Ok(()) => {
                event!(Level::DEBUG, "Attempting to read response from {} endpoint:", path);
                match read.next().await {
                    Some(response) => {
                        event!(Level::DEBUG, "We received a response!");
    
                        match response {
                            Ok(payload) => {
                                event!(Level::DEBUG, "{}", payload);
                                number_of_successes += 1;
                            }
                            Err(e) => {
                                event!(Level::ERROR, "{}", e);   
                            }
                        }
                    }
                    None => {}
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Could not send the request: {}", e);
            }
        }
    }

    if number_of_successes == number_of_iterations {
        event!(Level::INFO, "Send New Message Repeat Test passed!");
        true
    } else {
        event!(Level::ERROR, "Send New Message Repeat Test failed!");
        false
    }

}

async fn test_get_users_repeat() -> bool {
    let number_of_iterations: i32 = 3;
    let mut number_of_successes: i32 = 0;
    let path: &str = "/users";

    event!(Level::INFO, "Beginning Get Users Repeat Test.");

    let client = edge_view::client::ws_connect(7878, Algorithm::HS256, path).await;

    let (mut write, mut read) = client.unwrap().split();

    for i in 0..number_of_iterations {
        event!(Level::DEBUG, "========================================");
        event!(Level::DEBUG, "Iteration {}", i);

        match write.send(Message::Text(get_users_message())).await {
            Ok(()) => {
                event!(Level::DEBUG, "Attempting to read response from {} endpoint:", path);
                match read.next().await {
                    Some(response) => {
                        event!(Level::DEBUG, "We received a response!");
    
                        match response {
                            Ok(payload) => {
                                event!(Level::DEBUG, "{}", payload);
                                number_of_successes += 1;
                            }
                            Err(e) => {
                                event!(Level::ERROR, "{}", e);   
                            }
                        }
                    }
                    None => {}
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Could not send the request: {}", e);
            }
        }
    }

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

    if number_of_successes == number_of_iterations {
        event!(Level::INFO, "Get Users Repeat Test passed!");
        true
    } else {
        event!(Level::ERROR, "Get Users Repeat Test failed!");
        false
    }

} // end test_get_users

async fn test_get_messages() -> bool {
    event!(Level::INFO, "Beginning Get Messages Test.");

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/messages",
        build_messages_request()).await;

    match response {
        Some(payload) => {

            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Get Messages Test passed!");
            true
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Get Messages Test Failed!");
            false
        }
    }
} // end test_get_messages

async fn test_search_messages() -> bool {
    event!(Level::INFO, "Beginning Search Messages Test.");

    let response = ws_connect_send(
        7878,
        Algorithm::HS256,
        "/search",
        build_search_messages_request()).await;

    match response {
        Some(payload) => {
            


            event!(Level::DEBUG, "{}", payload);
            event!(Level::INFO, "Search Messages Test passed!");
            true
        }
        None => {
            event!(Level::DEBUG, "No response received.");
            event!(Level::ERROR, "Search Messages Test Failed!");
            false
        }
    }
} // end test_search_messages

async fn test(stream: TcpStream) {

    
    event!(Level::DEBUG, "Creating the request");
    let socket: Option<WebSocketStream<TcpStream>> = match format!("ws://localhost:{}{}", 7878, "/users").into_client_request() {
        Ok(mut auth_request) => {

            event!(Level::DEBUG, "Building the JWT");
            match format!("Bearer {}", build_jwt(Algorithm::HS256)).parse::<HeaderValue>() {

                Ok(auth_token) => {

                    event!(Level::DEBUG, "Inserting the header");
                    auth_request
                        .headers_mut()
                        .insert("Authorization", auth_token);


                    event!(Level::DEBUG, "Establishing the WebSocket handshake with the server");
                    match client_async(auth_request, stream).await {
                        Ok((socket, response)) => {
                            event!(Level::DEBUG, "Connected and retreived the WebSocket stream");
                            Some(socket)

                        }
                        Err(e) => {
                            event!(Level::ERROR, "Could not complete the WebSocket handshake: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    event!(Level::ERROR, "Could not parse the JWT into a HeaderValue: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            event!(Level::ERROR, "Could not create the request: {}", e);
            None
        }
    };

    match socket {
        Some(socket) => {

            event!(Level::DEBUG, "Splitting the WebSocket stream.");
            let (mut write, mut read) = socket.split();

            // Send the request.
            event!(Level::DEBUG, "Sending the Get Users request.");
            let result = match write.send(Message::Text(edge_view::client::build_users_request())).await {
                Ok(()) => {
                    event!(Level::DEBUG, "Attempting to read response from {} endpoint:", "/users");
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
        }
        None => {
            event!(Level::ERROR, "Could not create the socket.");
        }
    }
}

fn create_message_validator
(
    response: String
) -> bool {
    event!(Level::DEBUG, "Create Message payload:");
    event!(Level::DEBUG, "{}", response);

    match SendNewMessageResponse::try_from_json(response) {
        Ok(response_struct) => {
            event!(Level::DEBUG, "Successfully parsed the response into a struct: {}", response_struct);
            true
        }
        Err(e) => {
            event!(Level::ERROR, "The Create Message Response was invalid, and could not be parsed into a structure: {}", e);
            false
        }
    }
} // end create_message_validator

fn get_messages_validator
(
    response:   String
) -> bool {
    event!(Level::DEBUG, "Get Messages payload:");
    event!(Level::DEBUG, "{}", response);

    match GetMessagesResponse::try_from_json(response) {
        Ok(response_struct) => {
            event!(Level::DEBUG, "Successfully parsed the response into a struct: {}", response_struct);
            true
        }
        Err(e) => {
            event!(Level::ERROR, "The Get Messages Response was invalid, and could not be parsed into a structure: {}", e);
            false
        }
    }
} // end get_messages_validator

fn get_users_validator
(
    response:   String
) -> bool {
    event!(Level::DEBUG, "Get Users payload:");
    event!(Level::DEBUG, "{}", response);

    match GetUsersResponse::try_from_json(response) {
        Ok(response_struct) => {
            event!(Level::DEBUG, "Successfully parsed the response into a struct: {}", response_struct);
            true
        }
        Err(e) => {
            event!(Level::ERROR, "The Get Users Response was invalid, and could not be parsed into a structure: {}", e);
            false
        }
    }
} // end get_users_validator

fn search_messages_validator
(
    response:   String
) -> bool {
    event!(Level::DEBUG, "Search Messages payload:");
    event!(Level::DEBUG, "{}", response);

    match SearchMessagesResponse::try_from_json(response) {
        Ok(response_struct) => {
            event!(Level::DEBUG, "Successfully parsed the response into a struct: {}", response_struct);
            true
        }
        Err(e) => {
            event!(Level::ERROR, "The Search Messages Response was invalid, and could not be parsed into a structure: {}", e);
            false
        }
    }
} // end search_messages_validator

async fn run_test<F>
(
    config:         cli::Args,
    server_path:    &str,
    test_name:      &str,
    jwt_header_alg: jsonwebtoken::Algorithm,
    request:        String,
    validator:      F,
) -> bool
where
    F: Fn(String) -> bool,
{
    event!(Level::INFO, "Beginning {} Test.", test_name);

    let response = ws_connect_send(
        config.server_port,
        jwt_header_alg,
        server_path,
        request
    ).await;

    // Check the response from sending the request to the server.
    let test_status: bool = match response {
        Some(payload) => {

            match payload.into_text() {
                Ok(payload_string) => {
                    validator(payload_string)
                }
                Err(e) => {
                    event!(Level::ERROR, "Invalid response payload format: {}", e);
                    false
                }
            }
        }
        None => {
            event!(Level::ERROR, "No response received.");
            false
        }
    };

    // Log the test status message.
    match test_status {
        true => {
            event!(Level::INFO, "{} Test Passed!", test_name);
        }
        false => {
            event!(Level::ERROR, "{} Test Failed!", test_name);
        }
    }

    test_status
} // end run_test

async fn run_test_list(
    config:     cli::Args,
    test_list:  Vec<TestCase>
) {
    let mut test_status: bool;
    let mut total_tests: i32 = 0;
    let mut passed_tests: i32 = 0;

    for test_case in test_list {

        test_status = run_test(
            config.clone(),
            test_case.server_path.as_str(),
            test_case.test_name.as_str(),
            test_case.jwt_header_alg,
            test_case.request,
            test_case.validator
        ).await;

        if true == test_status {
            passed_tests += 1;
        }

        total_tests += 1;
    }

    event!(Level::INFO, "Tests Passed: {}/{}", passed_tests, total_tests);
} // end run_test_list

pub fn process_arguments
(
    config: cli::Args
) -> Vec<TestCase> {

    let mut test_list: Vec<TestCase> = Vec::new();

    if config.test_create_message || config.test_all {
        test_list.push(
            TestCase {
                server_path:    String::from("/send"),
                test_name:      String::from("Create Message"),
                jwt_header_alg: JWT_ALGORITHM,
                request:        build_new_message_request(),
                validator:      create_message_validator
            }
        );
    }

    if config.test_get_messages || config.test_all {
        test_list.push(
            TestCase {
                server_path:    String::from("/messages"),
                test_name:      String::from("Get Messages"),
                jwt_header_alg: JWT_ALGORITHM,
                request:        build_messages_request(),
                validator:      get_messages_validator
            }
        );
    }

    if config.test_get_users || config.test_all {
        test_list.push(
            TestCase {
                server_path:    String::from("/users"),
                test_name:      String::from("Get Users"),
                jwt_header_alg: JWT_ALGORITHM,
                request:        get_users_message(),
                validator:      get_users_validator
            }
        );
    }

    if config.test_search_messages || config.test_all {
        test_list.push(
            TestCase {
                server_path:    String::from("/search"),
                test_name:      String::from("Search Messages"),
                jwt_header_alg: JWT_ALGORITHM,
                request:        build_search_messages_request(),
                validator:      search_messages_validator
            }
        );
    }

    test_list
} // end process arguments

#[tokio::main]
async fn main() {

    // Set up the logging subscriber.
    dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    
    let args = cli::Args::parse();
    let test_list = process_arguments(args.clone());

    run_test_list(args, test_list).await;

    // while let Some(completed_task) = tasks.join_next().await {
    //     match completed_task {
    //         Ok(()) => {
    //             event!(Level::DEBUG, "Task completed.");
    //         }
    //         Err(e) => {
    //             event!(Level::ERROR, "A task encountered an error: {}", e);
    //         }
    //     }
    // }


    // let (socket, _) = client_async(
    //     auth_request,
    //     stream
    // ).await.expect("Failed to connect");
}