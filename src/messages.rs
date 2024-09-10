use serde::{ Deserialize, Serialize };
use std::fmt;

pub const TOPIC_USERS: &str = "users";
pub const TOPIC_MESSAGES: &str = "messages";
pub const TOPIC_SEARCH_MESSAGES: &str = "search";

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