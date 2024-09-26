use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
use serde::{ Deserialize, Serialize };
use std::fmt;
use uuid::Uuid;

pub const TOPIC_USERS: &str = "/users";
pub const TOPIC_MESSAGES: &str = "/messages";
pub const TOPIC_SEARCH_MESSAGES: &str = "/search";
pub const TOPIC_SEND_MESSAGE: &str = "/send";

//==============================================================================
// struct Edge View JWT Token definitions
//==============================================================================
#[allow(non_snake_case)]
pub struct EdgeViewJWTHeader {
    pub alg:    Algorithm,
    pub typ:    String,
}

impl EdgeViewJWTHeader {
    pub fn new() -> EdgeViewJWTHeader {
        EdgeViewJWTHeader {
            alg:    Algorithm::HS256,
            typ:    "JWT".to_string(),
        }
    }

    pub fn to_header(&self) -> Header {
        let mut new_header = Header::new(self.alg);
        new_header.typ = Some(self.typ.clone());

        new_header
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles:  Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmManagement {
    pub roles:  Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub roles:  Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub realm_management:   RealmManagement,
    pub account:            Account,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeViewClaims {
    // Expiration time in seconds.
    pub exp:                u64,
    // Issued at time in seconds.
    pub iat:                u64,
    // Time when authentication occurred in seconds.
    pub auth_time:          u64,
    // JTI is a UUID to indicate against replay attacks.
    pub jti:                String,
    // Token issuer, who created the token.
    pub iss:                String,
    // Audience, who the token is intended for.
    pub aud:                Option<Vec<String>>,
    // Subject, whom the token refers to.
    pub sub:                String,
    pub typ:                String,
    // Authorized party, the party to which this token was issued.
    pub azp:                String,
    pub nonce:              String,
    pub session_state:      String,
    pub acr:                String,
    pub allowed_origins:    Vec<String>,
    pub realm_access:       RealmAccess,
    pub resource_access:    ResourceAccess,
    pub scope:              String,
    pub sid:                Uuid,
    pub email_verified:     bool,
    pub name:               String,
    pub preferred_username: String,
    pub given_name:         String,
    pub family_name:        String,
    pub email:              String,
}

impl fmt::Display for EdgeViewClaims {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}



//==============================================================================
// struct GetMessagesRequest
//==============================================================================

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub domainId:   String,
    // The name of the chatroom that we want to get all users from.
    pub roomName:   String,
}

impl GetMessagesRequest {
    pub fn from_string(json: String) -> GetMessagesRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> GetMessagesRequest {
        serde_json::from_str(json).unwrap()
    }
    
    pub fn is_valid(&self) -> bool {
        #[allow(unused_braces)]
        (!self.domainId.is_empty() && !self.roomName.is_empty())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn try_from_str(json: &str) -> Result<GetMessagesRequest, &'static str> {
        match serde_json::from_str(json) {
            Ok(new_object) => {
                Ok(new_object)
            }
            Err(e) => {
                Err("Could not create a GetMessageRequest object from the given str")
            }
        }
    }
}

//==============================================================================
// struct SearchMessagesRequest
//==============================================================================
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SearchMessagesRequest {
    pub domainId:   String,
    pub roomName:   String,
    pub keywords:   Vec<String>,
}

impl SearchMessagesRequest {
 
    pub fn from_string(json: String) -> SearchMessagesRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> SearchMessagesRequest {
        serde_json::from_str(json).unwrap()
    }
}

//==============================================================================
// struct GetUsersRequest
//==============================================================================

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GetUsersRequest {
    pub domainId: String,
    // The name of the chatroom that we want to get all users from.
    pub roomName: String
}

impl fmt::Display for GetUsersRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "{{
            \n\t\"domainId\": \"{},\"
            \n\t\"roomName\": \"{},\"
            \n}}",
            self.domainId,
            self.roomName)
    }
}

impl GetUsersRequest {
 
    pub fn from_string(json: String) -> GetUsersRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> GetUsersRequest {
        serde_json::from_str(json).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        #[allow(unused_braces)]
        (!self.domainId.is_empty() && !self.roomName.is_empty())
    }
}

//==============================================================================
// struct GetUsersResponse
//==============================================================================
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GetUsersResponse {
    pub userNames: Vec<String>
}

impl fmt::Display for GetUsersResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GetUsersResponse {
 
    pub fn from_string(json: String) -> GetUsersResponse {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> GetUsersResponse {
        serde_json::from_str(json).unwrap()
    }

    pub fn new() -> GetUsersResponse {
        GetUsersResponse {
            userNames: Vec::new()
        }
    }

    /*
     * This method constructs a JSON string from the GetUsersResponse's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

//==============================================================================
// struct SendNewMessageRequest
//==============================================================================
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SendNewMessageRequest {
    pub domainId:   String,
    // The name of the chatroom that we want to get all users from.
    pub roomName:   String,
    pub text:       String,
}

impl fmt::Display for SendNewMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SendNewMessageRequest {
 
    pub fn from_string(json: String) -> SendNewMessageRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> SendNewMessageRequest {
        serde_json::from_str(json).unwrap()
    }

    pub fn new() -> SendNewMessageRequest {
        SendNewMessageRequest {
            domainId:   String::new(),
            roomName:   String::new(),
            text:       String::new()
        }
    }

    /*
     * This method constructs a JSON string from the SendNewMessageRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}