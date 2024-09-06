
use http::Response;
use serde_json;
use std::{
    borrow::Cow,
    str::FromStr,
    net::TcpStream,
};
use tracing::{event, span, Level};
use tracing_subscriber;
use tungstenite::{
    client,
    connect,
    Message,
    protocol,
    WebSocket,
    stream::MaybeTlsStream,
    
};
use url::Url;

mod messages;
use messages::{
    GetMessagesRequest,
    GetUsersRequest,
    SearchMessagesRequest,
};

mod ChatSurfer;
use ChatSurfer::messages as cs_messages;

const LOOP_LIMIT: i32 = 1;

fn get_users_message() -> String {
    let get_users_request: GetUsersRequest = GetUsersRequest {
        domainId: String::from("somedomain"),
        roomName: String::from("Test_Room")
    };

    serde_json::to_string(&get_users_request).unwrap()
}

fn build_messages_request() -> String {
    let messages_request: GetMessagesRequest = GetMessagesRequest {
        domainId: String::from("somedomain"),
        roomName: String::from("Test_Room")
    };

    serde_json::to_string(&messages_request).unwrap()
}

fn build_search_messages_request() -> String {
    let request: SearchMessagesRequest = SearchMessagesRequest {
        domainId: String::from("somedomain"),
        roomName: String::from("Test_Room"),
        keywords: vec!(String::from("test_keyword"), String::from("Austin")),
    };

    serde_json::to_string(&request).unwrap()
}

fn main() {
    // Set up the logging subscriber.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut loop_count: i32 = 0;
    let mut user_response_received: bool = false;
    let mut messages_response_received: bool = false;
    let mut search_messages_response_received: bool = false;

    let users_url = Url::parse("wss://localhost:3030/users").unwrap();
    let messages_url = Url::parse("wss://localhost:3030/messages").unwrap();
    let search_messages_url = Url::parse("wss://localhost:3030/search").unwrap();

    // Connect to the WebSocket.
    let (mut users_socket, response) = connect(users_url).expect("Can't connect");

    let (mut messages_socket, messages_response) = connect(messages_url).expect("Can't connect");

    let (mut search_messages_socket, search_messages_response) = connect(search_messages_url).expect("Can't connect");

    // High level loop to iterate over each endpoint we're testing.
    while loop_count < LOOP_LIMIT {
        //======================================================================
        // Get Messages Endpoint
        messages_socket.write_message(Message::Text(build_messages_request())).unwrap();

        while messages_response_received == false {
            event!(Level::DEBUG, "Attempting to read response from Messages endpoint:");
    
            match messages_socket.read_message() {
                Ok(message) => {
                    let response = message.to_text();

                    match response {
                        Ok(get_messages_request) => {
                            let response: cs_messages::GetChatMessagesResponse = serde_json::from_str(get_messages_request).unwrap();
                            let pretty_json = serde_json::to_string_pretty(&response).unwrap();

                            //let pretty_json = serde_json::to_string(&get_messages_request).unwrap();
                            event!(Level::DEBUG, "Received from server: {}", pretty_json);
                        }
                        _ => {
                            event!(Level::DEBUG, "Invalid response received.");
                        }
                    }
                    messages_response_received = true;                    
                },
                Err(error) => event!(Level::DEBUG, "Error receiving message: {}", error),
            };
        }
        messages_response_received = false;

        //======================================================================
        // Get Users Endpoint
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

        
        //======================================================================
        // Search Messages Endpoint
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