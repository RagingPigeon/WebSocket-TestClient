use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
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
use uuid::Uuid;

pub const SERVER_PORT: u16 = 7878;

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
    
    auth_request
        .headers_mut()
        .insert("Authorization", auth_token);

    match TcpStream::connect(url).await {
        Ok(stream) => {
            

            let (socket, _) = client_async(
                auth_request,
                stream
            ).await.expect("Failed to connect");

            Some(socket)
        }
        Err(e) => {
            None
        }
    }
}