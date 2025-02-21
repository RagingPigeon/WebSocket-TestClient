use crate::chatsurfer::messages::{
    ChatMessageSchema,
    ErrorCode400,
    ErrorCode404,
};
use anyhow::{
    Context,
    Result,
};
use http::StatusCode;
use serde::{ Deserialize, Serialize };
use std::fmt;
use tracing::{event, Level};
use uuid::Uuid;

pub const TOPIC_USERS: &str = "/users";
pub const TOPIC_MESSAGES: &str = "/messages";
pub const TOPIC_SEARCH_MESSAGES: &str = "/search";
pub const TOPIC_SEND_MESSAGE: &str = "/send";

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };
        write!(f, "{}", string)
    }
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

    /// This method will construct an Error structure using data from
    /// the given ChatSurfer ErrorCode400 error message.
    pub fn from_400(error: ErrorCode400) -> Error {
        Error {
            classification: error.classification,
            code:           error.code,
            message:        error.message,
        }
    }

    /// This method will construct an Error structure using data from
    /// the given ChatSurfer ErrorCode404 error message.
    pub fn from_404(error: ErrorCode404) -> Error {
        Error {
            classification: error.classification,
            code:           error.code,
            message:        error.message,
        }
    }

    /// This method will construct an Error structure representing an
    /// HTTP 429 Too Many Requests error, that is unclassified, and with
    /// the message "Too many requests".
    pub fn new_429() -> Error {
        Error {
            classification: String::from("UNCLASSIFIED"),
            code:           StatusCode::TOO_MANY_REQUESTS.as_u16(),
            message:        String::from("Too many requests")
        }
    }

    /// This method constructs a JSON string from the Error's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the Error struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the Error struct to a string.")?;

        Ok(error_string)
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
        let display_string = match self.to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SendNewMessageRequest {
    /// This method attempts to construct a SendNewMessageRequest
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_json(json: String) -> Result<SendNewMessageRequest, anyhow::Error> {
        let error_struct: SendNewMessageRequest = serde_json::from_str::<SendNewMessageRequest>(&json)
            .with_context(|| format!("Unable to create SendNewMessageRequest struct from String {}", json))?;

        Ok(error_struct)
    }

    /// This method constructs a JSON string from the SendNewMessageRequest's
    /// fields.
    pub fn to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SendNewMessageRequest struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SendNewMessageRequest struct to a string.")?;

        Ok(error_string)
    }
}

/// The SendNewMessageResponse structure defines the response that will be
/// send to Edge View for a successful Send Message request.
#[derive(Serialize, Deserialize)]
pub struct SendNewMessageResponse {
    pub message: String
}

impl SendNewMessageResponse {
    pub fn new_204() -> SendNewMessageResponse {
        SendNewMessageResponse {
            message: String::from("Successful - No Response Content"),
        }
    }

    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SendNewMessageResponse struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SendNewMessageResponse struct to a string.")?;

        Ok(error_string)
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

impl GetMessagesRequest {
    /// This method attempts to construct a GetMessagesRequest
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_json(json: String) -> Result<GetMessagesRequest, anyhow::Error> {
        let error_struct: GetMessagesRequest = serde_json::from_str::<GetMessagesRequest>(&json)
            .with_context(|| format!("Unable to create GetMessagesRequest struct from String {}", json))?;

        Ok(error_struct)
    } // end try_from_json
}

/// The GetMessagesResponse structure defines the response that will be sent to
/// Edge View for a successful Get Messages request.
#[derive(Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>,
}

impl fmt::Display for GetMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetMessagesResponse {
    /// This method constructs a new GetMessagesResponse structure that
    /// doesn't contain any data.
    pub fn new() -> GetMessagesResponse {
        GetMessagesResponse {
            classification: String::new(),
            messages:       Vec::new(),
        }
    }

    pub fn try_from_json(json: String) -> Result<GetMessagesResponse, anyhow::Error> {
        let response_struct: GetMessagesResponse = serde_json::from_str::<GetMessagesResponse>(&json)
        .with_context(|| format!("Unable to create GetMessagesResponse struct from String {}", json))?;

        Ok(response_struct)
    } // end try_from_json
    
    /// This method constructs a JSON string from the GetMessagesResponse's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the GetMessagesResponse struct, and return
        // any error with a context message.
        let struct_string: String = serde_json::to_string(self)
            .context("Unable to convert the GetMessagesResponse struct to a string.")?;

        Ok(struct_string)
    }
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
#[derive(Clone, Serialize, Deserialize)]
pub struct GetUsersRequest {
    #[serde(rename = "domainId")]
    pub domain_id: String,

    // The name of the chatroom that we want to get all users from.
    #[serde(rename = "roomName")]
    pub room_name: String
}

impl fmt::Display for GetUsersRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetUsersRequest {
    /// This method constructs a JSON string from the GetUsersRequest's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the GetUsersRequest struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the GetUsersRequest struct to a string.")?;

        Ok(error_string)
    }

    /// This method attempts to construct a GetUsersRequest
    /// structure from the given JSON String parameter.
    pub fn try_from_json(json: String) -> Result<GetUsersRequest, anyhow::Error> {
        
        let response_struct: GetUsersRequest = serde_json::from_str::<GetUsersRequest>(&json)
            .with_context(|| format!("Unable to create GetUsersRequest struct from String {}", json))?;

        Ok(response_struct)
    } // end try_from_json
} // end GetUsersRequest

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
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetUsersResponse {

    pub fn new() -> GetUsersResponse {
        GetUsersResponse {
            user_names: Vec::new()
        }
    }

    pub fn try_from_json(json: String) -> Result<GetUsersResponse, anyhow::Error> {
        let response_struct: GetUsersResponse = serde_json::from_str::<GetUsersResponse>(&json)
            .with_context(|| format!("Unable to create GetUsersResponse struct from String {}", json))?;

        Ok(response_struct)
    } // end try_from_json

    /// This method constructs a JSON string from the GetUsersResponse's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        let struct_string: String = serde_json::to_string(self)
            .context("Unable to convert the GetUsersResponse struct to a string.")?;

        Ok(struct_string)
    }
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

impl SearchMessagesRequest {

    /// This method attempts to construct a SearchMessagesRequest
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_json(json: String) -> Result<SearchMessagesRequest, anyhow::Error> {

        let error_struct: SearchMessagesRequest = serde_json::from_str::<SearchMessagesRequest>(&json)
            .with_context(|| format!("Unable to create SearchMessagesRequest struct from String {}", json))?;

        Ok(error_struct)
    } // end try_from_json
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

impl SearchMessagesResponse {
    /// This method constructs a JSON string from the GetUsersResponse's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SearchMessagesResponse struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SearchMessagesResponse struct to a string.")?;

        Ok(error_string)
    }
}

// #############################################################################
// #############################################################################
//                            EdgeViewResponseTypes                             
// #############################################################################
// #############################################################################
pub enum EdgeViewResponseTypes {
    SendNewMessage  { body: SendNewMessageResponse },
    GetMessages     { body: GetMessagesResponse },
    GetUsers        { body: GetUsersResponse },
    SearchMessages  { body: SearchMessagesResponse },
    Error           { body: Error },
}

impl EdgeViewResponseTypes {
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        let response_string: String = match self {
            EdgeViewResponseTypes::SendNewMessage { body } => {
                body.try_to_json()
                    .context("Failed to convert the SendNewMessageResponse variant of the EdgeViewResponseTypes enum to a string.")?
            }
            EdgeViewResponseTypes::GetMessages { body } => {
                body.try_to_json()
                    .context("Failed to convert the GetMessagesResponse variant of the EdgeViewResponseTypes enum to a string.")?
            }
            EdgeViewResponseTypes::GetUsers { body } => {
                body.try_to_json()
                    .context("Failed to convert the GetUsersResponse variant of the EdgeViewResponseTypes enum to a string.")?
            }
            EdgeViewResponseTypes::SearchMessages { body } => {
                body.try_to_json()
                    .context("Failed to convert the SearchMessagesResponse variant of the EdgeViewResponseTypes enum to a string.")?
            }
            EdgeViewResponseTypes::Error { body } => {
                body.try_to_json()
                    .context("Failed to convert the Error variant of the EdgeViewResponseTypes enum to a string.")?
            }
        };

        Ok(response_string)
    } // end try_to_json
}