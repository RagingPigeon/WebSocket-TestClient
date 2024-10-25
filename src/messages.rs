use crate::chatsurfer::messages::{
    ChatMessageSchema,
    ErrorCode400,
};
//use http::StatusCode;
use serde::{ Deserialize, Serialize };
use std::fmt;
use tracing::{event, Level};
use uuid::Uuid;

// #############################################################################
// #############################################################################
//                              Error Messages
// #############################################################################
// #############################################################################

/// The Error structure represents a common error message that will be sent
/// to Edge View when a request cannot be completed.  Following a common
/// error message scheme will keep things simple on the Edge View side.
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub classification: String,
    pub code:           u16,
    pub message:        String,
}

impl Error {
    /// This method will construct an Error structure that is
    /// unclassified, with a generic server error code, with the
    /// given message string.
    pub fn new_unclassified_message(message: &str) -> Error {
        Error {
            classification: String::from("UNCLASSIFIED"),
            code:           500,
            message:        String::from(message)
        }
    }
} // end Error

// #############################################################################
// #############################################################################
//                         Edge View Authentication
// #############################################################################
// #############################################################################

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

// #############################################################################
// #############################################################################
//                                Get Message
// #############################################################################
// #############################################################################

//==============================================================================
// struct GetMessagesRequest
//==============================================================================

/// The GetMessagesRequest structure represents a request that Edge View
/// sends to this chatsurfer-connect service to retrieve all of the
/// ChatSurfer chat messages within a specified chat room.
#[derive(Serialize, Deserialize)]
pub struct GetMessagesRequest {
    #[serde(rename = "domainId")]
    pub domain_id:   String,

    // The name of the chatroom that we want to get all users from.
    #[serde(rename = "roomName")]
    pub room_name:   String,
}

/// The GetMessagesResponse structure defines the response that will be sent to
/// Edge View for a successful Get Messages request.
#[derive(Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>,
}
/// The GetMessagesResponseTypes enumeration defines the types of responses
/// that we well send back to Edge View for the Get Messages endpoint.
pub enum GetMessagesResponseTypes {
    GetMessagesResponse { response: GetMessagesResponse },
    Error               { response: Error },
}

// #############################################################################
// #############################################################################
//                               Search Message
// #############################################################################
// #############################################################################

//==============================================================================
// struct SearchMessagesRequest
//==============================================================================

/// The SearchMessagesRequest structure defines the message we expect to
/// receive from Edge View to search a specified ChatSurfer chat room
/// for chat messages that contain the specified keywords.
#[derive(Serialize, Deserialize)]
pub struct SearchMessagesRequest {
    #[serde(rename = "domainId")]
    pub domain_id:   String,

    #[serde(rename = "roomName")]
    pub room_name:   String,
    pub keywords:   Vec<String>,
}

//==============================================================================
// struct SearchMessagesResponse
//==============================================================================

/// The SearchMessagesResponse structure defines the response that will be
/// sent to Edge View for a successful Search Messages request.
#[derive(Serialize, Deserialize)]
pub struct SearchMessagesResponse {
    pub messages:   Vec<ChatMessageSchema>,
}

// #############################################################################
// #############################################################################
//                                Get Users
// #############################################################################
// #############################################################################

//==============================================================================
// struct GetUsersRequest
//==============================================================================

/// The GetUsersRequest structure defines the message we expect to receive
/// from Edge View to gather all the user names of participants within
/// the specified ChatSurfer chat room.
#[derive(Serialize, Deserialize)]
pub struct GetUsersRequest {
    #[serde(rename = "domainId")]
    pub domain_id: String,

    // The name of the chatroom that we want to get all users from.
    #[serde(rename = "roomName")]
    pub room_name: String
}

impl fmt::Display for GetUsersRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "{{
            \n\t\"domain_id\": \"{},\"
            \n\t\"room_name\": \"{},\"
            \n}}",
            self.domain_id,
            self.room_name)
    }
}

//==============================================================================
// struct GetUsersResponse
//==============================================================================

/// The GetUsersResponse structure defines the response that will be sent to
/// Edge View for a successful Get Users request.
#[derive(Serialize, Deserialize)]
pub struct GetUsersResponse {
    #[serde(rename = "userNames")]
    pub user_names: Vec<String>
}

impl fmt::Display for GetUsersResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GetUsersResponse {
    /*
     * This method constructs a JSON string from the GetUsersResponse's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// The GetUserResponseTypes enumeration defines the types of responses
/// that we can send back to Edge View for the Get Users endpoint.
pub enum GetUserResponseTypes {
    GetUsersResponse    { response: GetUsersResponse },
    Error               { response: Error },
}

// #############################################################################
// #############################################################################
//                              Create Messages
// #############################################################################
// #############################################################################

//==============================================================================
// struct SendNewMessageRequest
//==============================================================================

/// The SendNewMessageRequest structure deines the message we expect to
/// receive from Edge View to send a chat message to the specified
/// ChatSurfer chat room.
#[derive(Serialize, Deserialize)]
pub struct SendNewMessageRequest {
    #[serde(rename = "domainId")]
    pub domain_id:  String,
    
    // The name of the chatroom that we want to get all users from.
    #[serde(rename = "roomName")]
    pub room_name:  String,
    pub text:       String,
}

impl fmt::Display for SendNewMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SendNewMessageRequest {
    /*
     * This method constructs a JSON string from the SendNewMessageRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// The SendNewMessageResponse structure defines the response that will be
/// send to Edge View for a successful Send Message request.
#[derive(Serialize, Deserialize)]
pub struct SendNewMessageResponse {
    pub message: String
}
/// The SendNewMessageResponseTypes enumeration defines the types of
/// responses that we can send back to Edge View for the Send Message
/// endpoint.
pub enum SendNewMessageResponseTypes {
    SendNewMessageResponse  { response: SendNewMessageResponse },
    Error                   { response: Error },
}