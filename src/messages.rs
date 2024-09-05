use serde::{ Deserialize, Serialize };
use serde_json::Result;
use std::fmt;

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
}