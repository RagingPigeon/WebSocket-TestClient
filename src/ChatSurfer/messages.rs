use anyhow::{
    Context,
    Result,
};

use http::StatusCode;
use serde::{ Deserialize, Serialize };
use std::{
    collections::HashMap,
    fmt
};
use strum_macros::{ EnumString, Display };
// use tracing::{ event, Level };
// use uuid::Uuid;

use crate::service::error::CommonError;

/// The ChatSurfer API limits client requests to a certain number
/// every minute.
/// 
/// <https://chatsurfer.nro.mil/apidocs#section/(U)-Rate-Limiting>
pub const MAX_REQUESTS_PER_MINUTE: i32 = 60;




const MAX_ERROR_ARGUMENTS: usize = 1;
const COORDINATES_IN_POINT: usize = 2;
const POINTS_IN_POLYGON: usize = 4;
pub const MAX_REGIONS: usize = 1;
pub const MAX_REGION_BOUNDS: usize = 4;
pub const MAX_MESSAGE_GEOTAGS: usize = 1;

// Classification strings
pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";

/// This utility function attempts to convert the given HTTP status code and
/// message from a failed HTTP response into a structure.
/// 
/// # Parameters
/// ## status_code
/// The `status_code` parameter indicates the HTTP status code received from
/// the HTTP operation.  Since this function attempts to parse error messages,
/// this `status_code` is expected to be `4XX` or `5XX` HTTP error code.
/// Any other code will result in a CommonError structure with a generic
/// error message.
/// 
/// ## response
/// The `response` parameter contains the JSON message received from the server.
/// 
/// # Returns
/// - Ok() if successfully converted the error data into a
///   ChatSurferResponseType error variant.
/// - Err() if the `status_code` or `response` were unrecognized and could
///   not be converted into a ChatSurferResponseType error variant.  Err() will
///   contain a generic CommonError struct and error message.
pub fn parse_error_message(
    status_code:    StatusCode,
    response:       String,
) -> Result<ChatSurferResponseType, anyhow::Error> {

    match status_code {
        StatusCode::BAD_REQUEST => {
            let error_struct = ErrorCode400::try_from_string(response)
                .context("Unable to convert the response body to a ErrorCode400 struct.")?;

            Ok(ChatSurferResponseType::Failure400 { body: error_struct })
        }
        StatusCode::NOT_FOUND => {
            let error_struct = ErrorCode404::try_from_string(response)
                .context("Unable to convert the response body to a ErrorCode404 struct.")?;

            Ok(ChatSurferResponseType::Failure404 { body: error_struct })
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Ok(ChatSurferResponseType::Failure429)
        }
        _ => {
            Err(
                CommonError::from("Error - Message contents do not match any expected format.").into()
            )
        }
    }
} // end parse_error_message

// #############################################################################
// #############################################################################
//                              Error Messages
// #############################################################################
// #############################################################################

//==============================================================================
// ErrorCode400
//==============================================================================

/// This structure represents an HTTP 400 Bad Request message received
/// from ChatSurfer.
#[derive(Serialize, Deserialize)]
pub struct ErrorCode400 {
    pub classification: String,
    pub code:           u16,
    
    #[serde(rename = "fieldErrors")]
    pub field_errors:   Vec<FieldErrorSchema>,
    pub message:        String,
}

impl Default for ErrorCode400 {
    fn default() -> Self {
        ErrorCode400 {
            classification: String::from(UNCLASSIFIED_STRING),
            code:           400,
            field_errors:   Vec::new(),
            message:        String::from("Bad Request"),
        }
    }
}

/// Implement the trait fmt::Display for the struct ErrorCode400
/// so that these structs can be easily printed to consoles.
impl fmt::Display for ErrorCode400 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl ErrorCode400 {
    #[allow(dead_code)]
    pub fn test(source: String) -> ErrorCode400 {
        ErrorCode400 {
            classification: String::from(UNCLASSIFIED_STRING),
            code:           400,
            field_errors:   vec!(FieldErrorSchema::from_string(source.clone())),
            message:        source.clone(),
        }
    }
    
    /// This method attempts to construct a ErrorCode400
    /// structure from the given String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String) -> Result<ErrorCode400, anyhow::Error> {
        let error_struct: ErrorCode400 = serde_json::from_str::<ErrorCode400>(&source)
            .with_context(|| format!("Unable to create ErrorCode400 struct from String {}", source))?;

        Ok(error_struct)
    } // end try_from_string

    /// This method constructs a JSON string from the
    /// ErrorCode400's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the ErrorCode400 struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the ErrorCode400 struct to a string.")?;

        Ok(error_string)
    } // end try_to_json
} // end ErrorCode400

//==============================================================================
// ErrorCode404
//==============================================================================

/// This structure represents an HTTP 404 Not Found message received
/// from ChatSurfer.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorCode404 {
    pub classification: String,
    pub code:           u16,
    pub message:        String
}

impl std::fmt::Display for ErrorCode404 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl std::error::Error for ErrorCode404 {}

impl ErrorCode404 {
    /// This method attempts to construct a ErrorCode404
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String) -> Result<ErrorCode404, anyhow::Error> {
        let error_struct: ErrorCode404 = serde_json::from_str::<ErrorCode404>(&source)
            .with_context(|| format!("Unable to create ErrorCode404 struct from String {}", source))?;

        Ok(error_struct)
    } // end try_from_string
    
    /// This method constructs a JSON string from the
    /// ErrorCode404's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the ErrorCode404 struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the ErrorCode404 struct to a string.")?;

        Ok(error_string)
    } // end try_to_json
}

// #############################################################################
// #############################################################################
//                              API Key Messages
// #############################################################################
// #############################################################################

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum ApiKeyStatus {
    #[strum(serialize = "ACTIVE")]
    Active,
    #[strum(serialize = "DISABLED")]
    Disabled,
    #[strum(serialize = "PENDING")]
    Pending,
}

#[derive(Serialize, Deserialize)]
pub struct GetApiResponse {
    pub classification: String,
    
    // The Distinguished Name of the certificate used to
    // create the API key.
    pub dn:             String,
    pub email:          String,
    pub key:            String,

    // The status of the API Key.
    pub status:         String,
}

/// Implement the trait fmt::Display for the struct GetApiResponse
/// so that these structs can be easily printed to consoles.
impl fmt::Display for GetApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetApiResponse {
    /// This method attempts to construct a GetApiResponse
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_json(json: String) -> Result<GetApiResponse, anyhow::Error> {
        let error_struct: GetApiResponse = serde_json::from_str::<GetApiResponse>(&json)
            .with_context(|| format!("Unable to create GetApiResponse struct from String {}", json))?;

        Ok(error_struct)
    } // end try_from_json
    
    /// This method constructs a JSON string from the
    /// GetApiResponse's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the GetApiResponse struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the GetApiResponse struct to a string.")?;

        Ok(error_string)
    }
} // end GetApiResponse

// =============================================================================
// struct SendChatMessageRequest
// =============================================================================

/// The SendChatMessageRequest structure represents an HTTP request that can
/// be sent to ChatSurfer to create a new chat message within the defined
/// chat room.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Send%20Chat%20Message>
#[derive(Serialize, Deserialize)]
pub struct SendChatMessageRequest {
    pub classification: String,

    #[serde(rename = "domainId")]
    pub domain_id:      String,
    pub message:        String,
    pub nickname:       String,

    #[serde(rename = "roomName")]
    pub room_name:      String
}

/// Implement the trait Default for the struct SendChatMessageRequest
/// so that we can fall back on default values.
impl Default for SendChatMessageRequest {
    fn default() -> SendChatMessageRequest {
        SendChatMessageRequest {
            classification: String::from(UNCLASSIFIED_STRING),
            domain_id:      String::new(),
            message:        String::new(),
            nickname:       String::from("Edge View"),
            room_name:      String::new()
        }
    }
}

/// Implement the trait fmt::Display for the struct SendChatMessageRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SendChatMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SendChatMessageRequest {
    /// This method constructs a JSON string from the
    /// SendChatMessageRequest's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SendChatMessageRequest struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SendChatMessageRequest struct to a string.")?;

        Ok(error_string)
    }
} //end SendChatMessageRequest

// =============================================================================
// GetChatMessagesResponse
// =============================================================================

/// The GetChatMessagesResponse structure defines the response we
/// expect to receive from a successful Get Chat messages By Room request.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20Chat%20Messages%20By%20Room>
#[derive(Serialize, Deserialize)]
pub struct GetChatMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>,

    #[serde(rename = "domainId")]
    pub domain_id:      String,
    pub private:        bool,
    
    #[serde(rename = "roomName")]
    pub room_name:      String,
}

impl fmt::Display for GetChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetChatMessagesResponse {
    #[allow(dead_code)]
    pub fn test(source: String) -> GetChatMessagesResponse {
        GetChatMessagesResponse {
            classification: String::from("UNCLASSIFIED"),
            messages:       vec!(),
            domain_id:      source.clone(),
            private:        false,
            room_name:      source,
        }
    }

    pub fn try_from_string(source: String) -> Result<GetChatMessagesResponse, anyhow::Error> {
        let error_struct: GetChatMessagesResponse = serde_json::from_str::<GetChatMessagesResponse>(&source)
            .with_context(|| format!("Unable to create GetChatMessagesResponse struct from String {}", source))?;

        Ok(error_struct)
    }

    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the GetChatMessagesResponse struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the GetChatMessagesResponse struct to a string.")?;

        Ok(error_string)
    }
} // end GetChatMessagesResponse

// =============================================================================
// SearchChatMessagesRequest
// =============================================================================

/// The SearchChatMessagesRequest structure represents an HTTP request that can
/// be sent to ChatSurfer to search for chat messages based on the given
/// search criteria.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesRequest {
    pub cursor:             Option<String>,
    
    #[serde(rename = "filesOnly")]
    pub files_only:         Option<bool>,
    
    #[serde(rename = "highlightResults")]
    pub highlight_results:  Option<bool>,
    
    #[serde(rename = "keywordFilter")]
    pub keyword_filter:     Option<KeywordFilter>,
    pub limit:              Option<i32>,
    pub location:           Option<LocationSchema>,
    
    #[serde(rename = "locationFilter")]
    pub location_filter:    Option<bool>,
    
    #[serde(rename = "mentionFilter")]
    pub mention_filter:     Option<MentionFilter>,
    
    #[serde(rename = "requestGeoTags")]
    pub request_geo_tags:   Option<bool>,
    
    #[serde(rename = "roomFilter")]
    pub room_filter:        Option<DomainFilterDetail>,
    
    #[serde(rename = "senderFilter")]
    pub sender_filter:      Option<DomainFilterDetail>,
    pub sort:               Option<SortFilter>,
    
    #[serde(rename = "threadIdFilter")]
    pub thread_id_filter:   Option<ThreadIdFilter>,
    
    #[serde(rename = "timeFilter")]
    pub time_filter:        Option<TimeFilterRequest>,
    
    #[serde(rename = "userIdFilter")]
    pub user_id_filter:     Option<UserIdFilter>,

    //#[serde(rename = "userHighClassification")]
    //pub userHighClassification:   String,
}

#[allow(clippy::derivable_impls)]
impl Default for SearchChatMessagesRequest {
    fn default() -> Self {
        SearchChatMessagesRequest {
            cursor:             None,
            files_only:         None,
            highlight_results:  None,
            keyword_filter:     None,
            limit:              None,
            location:           None,
            location_filter:    None,
            mention_filter:     None,
            request_geo_tags:   None,
            room_filter:        None,
            sender_filter:      None,
            sort:               None,
            thread_id_filter:   None,
            time_filter:        None,
            user_id_filter:     None,
            //userHighClassification:   String::from("Test"),
        }
    }
}

/// Implement the trait fmt::Display for the struct SearchChatMessagesRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SearchChatMessagesRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SearchChatMessagesRequest {
    
    pub const NUM_RESULTS: i32 = 10;

    /// This method constructs a JSON string from the SearchChatMessagesRequest's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SearchChatMessagesRequest struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SearchChatMessagesRequest struct to a string.")?;

        Ok(error_string)
    }
}

// =============================================================================
// SearchChatMessagesResponse
// =============================================================================

/// The SearchChatMessagesResponse structure represents the response we
/// expect to receive from ChatSurfer upon a successful Search Chat messages
/// request.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesResponse {
    pub classification:     String,
    pub messages:           Option<Vec<ChatMessageSchema>>,

    #[serde(rename = "nextCursorMark")]
    pub next_cursor_mark:   Option<String>,

    #[serde(rename = "searchTimeFiler")]
    pub search_time_filter: TimeFilterResponse,
    pub total:              i32,
}

/// Implement the trait fmt::Display for the struct SearchChatMessagesResponse
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SearchChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SearchChatMessagesResponse {
    /// This method attempts to construct a SearchChatMessagesResponse
    /// structure from the given String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String)
        -> Result<SearchChatMessagesResponse, anyhow::Error> {

        let error_struct: SearchChatMessagesResponse = serde_json::from_str::<SearchChatMessagesResponse>(&source)
            .with_context(|| format!("Unable to create SearchChatMessagesRequest struct from String {}", source))?;

        Ok(error_struct)
    } // end try_from_string
    
    /// This method constructs a JSON string from the
    /// SearchChatMessagesResponse's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the SearchChatMessagesResponse struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the SearchChatMessagesResponse struct to a string.")?;

        Ok(error_string)
    }
} // end SearchChatMessagesResponse

/// This enumeration defines the types of responses we can receive
/// from ChatSurfer.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20API%20Key>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Send%20Chat%20Message>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20Chat%20Messages%20By%20Room>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
pub enum ChatSurferResponseType {
    SendChatMessage,
    GetChatMessages     { body: GetChatMessagesResponse },
    SearchChatMessages  { body: SearchChatMessagesResponse },
    Failure400          { body: ErrorCode400 },
    Failure404          { body: ErrorCode404 },
    Failure429,
}

// #############################################################################
// #############################################################################
//                           Supporting Structures
// #############################################################################
// #############################################################################
//==============================================================================
// ChatMessageSchema
//==============================================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessageSchema {
    pub classification: String,
    
    #[serde(rename = "domainId")]
    pub domain_id:      String,
    
    #[serde(rename = "geoTags")]
    pub geo_tags:       Option<Vec<GeoTagSchema>>,
    pub id:             String,
    
    #[serde(rename = "roomName")]
    pub room_name:      String,
    pub sender:         String,
    pub text:           String,
    
    #[serde(rename = "threadId")]
    pub thread_id:      Option<String>,
    pub timestamp:      String,
    
    #[serde(rename = "userId")]
    pub user_id:        String,
    pub private:        bool,
}

impl fmt::Display for ChatMessageSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl ChatMessageSchema {
    // Allow this function to be dead code, since we may use it in the
    // future.
    #[allow(dead_code)]
    pub fn try_from_json(json: String)
        -> Result<ChatMessageSchema, anyhow::Error> {

        let error_struct: ChatMessageSchema = serde_json::from_str::<ChatMessageSchema>(&json)
            .with_context(|| format!("Unable to create ChatMessageSchema struct from String {}", json))?;

        Ok(error_struct)
    }

    // Allow this function to be dead code, since we may use it in the
    // future.
    #[allow(dead_code)]
    pub fn test(source: String, seed: f32) -> ChatMessageSchema {
        ChatMessageSchema {
            classification: String::from("UNCLASSIFIED"),
            domain_id:      source.clone(),
            geo_tags:       Some(vec!(GeoTagSchema::test(source.clone(), seed))),
            id:             source.clone(),
            room_name:      source.clone(),
            sender:         source.clone(),
            text:           source.clone(),
            thread_id:      Some(source.clone()),
            timestamp:      source.clone(),
            user_id:        source.clone(),
            private:        false,
        }
    }
    
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the ChatMessageSchema struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the ChatMessageSchema struct to a string.")?;

        Ok(error_string)
    }
} // end ChatMessageSchema

//==============================================================================
// FieldErrorSchema
//==============================================================================
#[derive(Default, Serialize, Deserialize)]
pub struct FieldErrorSchema {
    #[serde(rename = "fieldName")]
    pub field_name:         String,
    pub message:            String,
    
    #[serde(rename = "messageArguments")]
    pub message_arguments:  Vec<String>,
    
    #[serde(rename = "messageCode")]
    pub message_code:       String,
    
    #[serde(rename = "rejectedValue")]
    pub rejected_value:     String
}

impl FieldErrorSchema {
    pub fn from_string(source: String) -> FieldErrorSchema {
        FieldErrorSchema {
            field_name:         source.clone(),
            message:            source.clone(),
            message_arguments:  vec!(source.clone()),
            message_code:       source.clone(),
            rejected_value:     source.clone(),
        }
    }
}

//==============================================================================
// NetworkId
//==============================================================================
/// This enum lists the possible values for a Domain's network ID.
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum NetworkId {
    #[strum(serialize = "bices")]
    Bices,

    #[strum(serialize = "cxk")]
    Cxk,

    #[strum(serialize = "sipr")]
    Sipr,

    #[strum(serialize = "jwics")]
    Jwics,

    #[strum(serialize = "unclass")]
    Unclass,
}

//==============================================================================
// JoinStatus
//==============================================================================
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum JoinStatus {
    #[strum(serialize = "JOINED")]
    Joined,

    #[strum(serialize = "NOT_JOINED")]
    NotJoined,
}

//==============================================================================
// LocationCoordinatesSchema
//==============================================================================
/// The LocationCoordinates union is used for the "coordinates" field in the
/// "Location" struct to represent either a single geographic point, or a
/// set of points to define a polygon.
#[derive(Clone, Serialize, Deserialize)]
pub struct LocationCoordinatesSchema {
    #[serde(rename = "type")]
    r#type:                 LocationType,

    // The first entry represents the coordinates for a single point.
    point_coordinates:      Vec<f32>,
    
    // The second entry represents a set of points for a polygon.
    polygon_coordinates:    Vec<Vec<f32>>,

}

impl fmt::Display for LocationCoordinatesSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl LocationCoordinatesSchema {

    #[allow(dead_code)]
    pub fn test(seed: f32) -> LocationCoordinatesSchema {
        LocationCoordinatesSchema {
            r#type:                 LocationType::Point,
            point_coordinates:      vec!(seed), // copy
            polygon_coordinates:    vec!(vec!(seed)), // copy
        }
    }
    
    /// This method constructs a JSON string from the LocationCoordinateSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the LocationCoordinatesSchema struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the LocationCoordinatesSchema struct to a string.")?;

        Ok(error_string)
    } //end try_to_json
} // end LocationCoordinatesSchema

//==============================================================================
// LocationType
//==============================================================================
#[derive(Debug, PartialEq, EnumString, Display)]
#[derive(Clone, Serialize, Deserialize)]
pub enum LocationType {
    Point,
    Polygon,
}

/// Define the default value for the LocationType enum.
impl Default for LocationType {
    fn default() -> Self {
        LocationType::Point
    }
} // end LocationType

#[derive(Clone, Serialize, Deserialize)]
pub struct PointLocation {

}

#[derive(Clone, Serialize, Deserialize)]
pub struct PolygonLocation {
    #[serde(rename = "type")]
    r#type: String,
    coordinates: Vec<Vec<f32>>,
}

impl PolygonLocation {
    pub fn new(new_coordinates: Vec<Vec<f32>>) -> PolygonLocation {
        PolygonLocation {
            r#type:         String::from("Polygon"),
            coordinates:    new_coordinates
        }
    }

    pub fn test(seed: f32) -> PolygonLocation {
        PolygonLocation {
            r#type:         String::from("Polygon"),
            coordinates:    vec!(vec!(seed)), // copy
        }
    }

    pub fn world_coordinates() -> Vec<Vec<f32>> {
        vec!(
            vec!(90.0, 180.0),
            vec!(90.0, -180.0),
            vec!(-90.0, -180.0),
            vec!(-90.0, 180.0),
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LocationTypes {
    Point { location: PointLocation },
    Polygon { location: PolygonLocation },
}

//==============================================================================
// LocationSchema
//==============================================================================
/// The Location struct represent a particular geographic location relevant
/// to a particular chat message.
#[derive(Clone, Serialize, Deserialize)]
pub struct LocationSchema {
    pub aoi:    LocationTypes,

    #[serde(rename = "type")]
    pub r#type: LocationType
}

impl fmt::Display for LocationSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl LocationSchema {
    pub fn new_polygon() -> LocationSchema {
        LocationSchema {
            r#type: LocationType::Polygon,
            aoi:    LocationTypes::Polygon {
                location: PolygonLocation::new(
                    PolygonLocation::world_coordinates()
                )
            }
        }
    }

    pub fn test(seed: f32) -> LocationSchema {
        LocationSchema {
            aoi:    LocationTypes::Polygon { location: PolygonLocation::test(seed) },
            r#type: LocationType::Point,
        }
    }
    
    /// This method constructs a JSON string from the LocationSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the LocationSchema struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the LocationSchema struct to a string.")?;

        Ok(error_string)
    }
} // end LocationSchema

//==============================================================================
// struct RegionSchema
//==============================================================================
/// The Region struct describes a notable geographic area with identifying
/// information.
#[derive(Clone, Serialize, Deserialize)]
pub struct RegionSchema {
    pub abbreviation:   String,
    pub bounds:         Vec<f32>,
    pub description:    String,
    pub name:           String,

    #[serde(rename = "regionType")]
    pub region_type:    String,
}

impl fmt::Display for RegionSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl RegionSchema {

    pub fn test(source: String, seed: f32) -> RegionSchema {
        RegionSchema {
            abbreviation:   source.clone(),
            bounds:         vec!(seed),
            description:    source.clone(),
            name:           source.clone(),
            region_type:    source.clone(),
        }
    }
    
    /// This method constructs a JSON string from the RegionSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {        
        // Attempt to serialize the RegionSchema struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the RegionSchema struct to a string.")?;

        Ok(error_string)
    }
} // end RegionSchema

//==============================================================================
// struct GeoTagSchema
//==============================================================================
/// The GeoTag struct allows context information to be added to a chat message.
#[derive(Clone, Serialize, Deserialize)]
pub struct GeoTagSchema {
    #[serde(rename = "anchorEnd")]
    pub anchor_end:     i64,

    #[serde(rename = "anchorStart")]
    pub anchor_start:   i64,
    
    #[serde(rename = "anchorText")]
    pub anchor_text:    String,
    pub confidence:     f32,
    pub location:       LocationSchema,
    pub regions:        Vec<RegionSchema>,
    pub r#type:         String
}

impl fmt::Display for GeoTagSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GeoTagSchema {
    pub fn test(source: String, seed: f32) -> GeoTagSchema {
        GeoTagSchema {
            anchor_end:     0,
            anchor_start:   0,
            anchor_text:    source.clone(),
            confidence:     0.0,
            location:       LocationSchema::test(seed),
            regions:        vec!(RegionSchema::test(source.clone(), seed)),
            r#type:         source.clone(),
        }
    }

    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the GeoTagSchema struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the GeoTagSchema struct to a string.")?;

        Ok(error_string)
    }
} // end GeoTagSchema

// =============================================================================
// struct KeywordFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct KeywordFilter {
    pub query: String
}

/// Implement the trait fmt::Display for the struct KeywordFilter
/// so that these structs can be easily printed to consoles.
impl fmt::Display for KeywordFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl KeywordFilter {
    /// ChatSurfer expects keyword filters to be formatted as one
    /// string.  This method will take the given vector of Strings
    /// and combine them into a single String query.
    /// 
    /// This method returns an Option.  ChatSurfer specifies the
    /// KeywordFilter as optional, so this return type makes it
    /// easy to embed the result of this method into a request.
    pub fn try_from_vec(keywords: Vec<String>) -> Result<KeywordFilter, anyhow::Error> {
        let mut keywords = keywords.clone();

        // We have at least one keyword to work with.  Set that as
        // the first word in the string.
        match keywords.pop() {
            Some(mut combined_keywords) => {
                
                // Check to see if there are any other keywords to incorporate.
                if keywords.len() > 1 {
                    for keyword in keywords {
                        combined_keywords = format!(
                            "{} {}",
                            combined_keywords,
                            keyword
                        );
                    }
                }

                Ok(KeywordFilter { query: combined_keywords })
            }
            None => {
                Err(CommonError {
                    message: String::from("No keywords to use.")
                }.into())
            }
        }
    } // end try_from_vec

    /// This method constructs a JSON string from the KeywordFilter's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the KeywordFilter struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the KeywordFilter struct to a string.")?;

        Ok(error_string)
    }
} // end KeywordFilter

// =============================================================================
// MentionType
// =============================================================================
#[derive(Deserialize, EnumString, Serialize)]
pub enum MentionType {
    #[strum(serialize = "USER")]
    User,
}

// =============================================================================
// Mention
// =============================================================================
/// This struct contains fields for searching for chat messages that
/// contain identifiers of mentioned users.
#[derive(Serialize, Deserialize)]
pub struct Mention {
    #[serde(rename = "mentionType")]
    pub mention_type:   MentionType,
    pub value:          String,
}

// =============================================================================
// MentionFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct MentionFilter {
    pub mentions:   Vec<Mention>,
}

// =============================================================================
// DomainFilterProperties
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct DomainFilterProperties {
    pub properties: Vec<String>,
}

// =============================================================================
// DomainFilterDetail
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct DomainFilterDetail  {
    // This field is a map of Domain IDs to an array of room names
    // or sender names.
    pub domains: HashMap<String, DomainFilterProperties>,
}

// =============================================================================
// SortDirection
// =============================================================================
#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortDirection {
    #[strum(serialize = "ASC")]
    Asc,
    #[strum(serialize = "DESC")]
    Desc,
}

// =============================================================================
// SortField
// =============================================================================
#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortField {
    #[strum(serialize = "DOMAIN")]
    Domain,
    #[strum(serialize = "RELEVANCE")]
    Relevance,
    #[strum(serialize = "ROOM")]
    Room,
    #[strum(serialize = "SENDER")]
    Sender,
    #[strum(serialize = "TIME")]
    Time,
}

// =============================================================================
// SortFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct SortFilter {
    pub orders: Vec<(SortDirection, SortField)>,
}

// =============================================================================
// ThreadIdFilter
// =============================================================================
/// This struct contains fields for filtering chat message searches
/// based on the message thread those messages belong to.
#[derive(Serialize, Deserialize)]
pub struct ThreadIdFilter {
    #[serde(rename = "threadIds")]
    pub thread_ids: Vec<String>,
}

// =============================================================================
// TimeFilterRequest
// =============================================================================
/// This struct contains fields that can be used as filters when searching
/// for chat messages within a ChatSurfer chat room.
/// 
/// Each field in this struct is considered an optional parameter from
/// ChatSurfer's perspective.  So when determining the validity of a search
/// request, these fields should be allowed to be ignored.
#[derive(Serialize, Deserialize)]
pub struct TimeFilterRequest {
    #[serde(rename = "endDateTime")]
    end_date_time:      Option<String>, //This string needs to be in DateTime format.

    #[serde(rename = "lookBackDuration")]
    look_back_duration: Option<String>,
    
    #[serde(rename = "startDateTime")]
    start_date_time:    Option<String>, //This string needs to be in DateTime format.
}

impl Default for TimeFilterRequest {
    fn default() -> Self {
        TimeFilterRequest {
            end_date_time:      Some(String::new()),
            look_back_duration: Some(String::new()),
            start_date_time:    Some(String::new()),
        }
    }
}

/// Implement the trait fmt::Display for the struct TimeFilterRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for TimeFilterRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl TimeFilterRequest {
    
    /// This method constructs a JSON string from the TimeFilterRequest's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the TimeFilterRequest struct, and return
        // any error with a context message.
        let error_string: String = serde_json::to_string(self)
            .context("Unable to convert the TimeFilterRequest struct to a string.")?;

        Ok(error_string)
    }
}

// =============================================================================
// UserIdFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct UserIdFilter {
    #[serde(rename = "userIds")]
    pub user_ids:    Vec<String>,
}

// =============================================================================
// TimeFilterResponse
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct TimeFilterResponse {
    #[serde(rename = "endDateTime")]
    pub end_date_time:  String,
}