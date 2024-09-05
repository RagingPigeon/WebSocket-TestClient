use messages::GetUsersRequest;
use serde_json;
use std::{borrow::Cow, str::FromStr};
use tracing::{event, span, Level};
use tracing_subscriber;
use tungstenite::{connect, Message, protocol};
use url::Url;

mod messages;

const LOOP_LIMIT: i32 = 1;

fn get_users_message() -> String {
    let get_users_request: GetUsersRequest = GetUsersRequest {
        domainId: String::from("somedomain"),
        roomName: String::from("Test_Room")
    };

    serde_json::to_string(&get_users_request).unwrap()
}

fn main() {
    // Set up the logging subscriber.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut loop_count: i32 = 0;
    let mut user_response_received: bool = false;
    let mut messages_response_received: bool = false;

    let users_url = Url::parse("wss://localhost:3030/users").unwrap();
    let messages_url = Url::parse("wss://localhost:3030/messages").unwrap();

    // Connect to the WebSocket.
    let (mut users_socket, response) = connect(users_url).expect("Can't connect");

    //let (mut messages_socket, messages_response) = connect(messages_url).expect("Can't connect");

    // High level loop to iterate over each endpoint we're testing.
    while loop_count < LOOP_LIMIT {
        users_socket.write_message(Message::Text(get_users_message())).unwrap();

        while user_response_received == false {
            event!(Level::DEBUG, "Attempting to read response from Users endpoint:");

            match users_socket.read_message() {
                Ok(message) => {
                    event!(Level::DEBUG, "Received from server: {}", message);
                    user_response_received = true;
                },
                Err(error) => event!(Level::DEBUG, "Error receiving message: {}", error),
            };
        }
        user_response_received = false;

        loop_count += 1;
    }

    loop {

    }

    let closing_code: Option<protocol::frame::CloseFrame> = Some(protocol::frame::CloseFrame {
        code: protocol::frame::coding::CloseCode::Normal,
        reason: Cow::from("hello")
    });

    match users_socket.close(closing_code) {
        Ok(_) => event!(Level::DEBUG, "Successfully closed the WebSocket!"),
        Err(error) => event!(Level::DEBUG, "Error closing the WebSocket: {}", error),
    }

        
        // messages_socket.write_message(Message::Text("Hello Messages!".into())).unwrap();

        // while messages_response_received == false {
        //     let messages_msg: Message = socket.read_message().expect("Error reading message from Messages endpoint");


        //     event!(Level::DEBUG, "Received from Messages endpoint: {}", messages_msg);
        // }
        // messages_response_received = false;


}