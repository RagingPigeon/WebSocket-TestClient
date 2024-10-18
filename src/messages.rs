use crate::ChatSurfer::messages::ChatMessageSchema;
use serde::{ Deserialize, Serialize };
use std::fmt;
use uuid::Uuid;

//==============================================================================
// struct Edge View JWT Token definitions
//==============================================================================

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
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub domainId:   String,
    // The name of the chatroom that we want to get all users from.
    pub roomName:   String,
}

// #############################################################################
// #############################################################################
//                               Search Message
// #############################################################################
// #############################################################################

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

//==============================================================================
// struct SearchMessagesResponse
//==============================================================================
#[derive(Serialize, Deserialize)]
pub struct SearchMessagesResponse {
    pub messages:   Vec<ChatMessageSchema>,
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
    /*
     * This method constructs a JSON string from the SendNewMessageRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}