use std::{
    collections::HashMap,
    fmt,
};

use chrono::{ DateTime, Utc };
use http::status;

//use strum::Display;
use serde::{ Deserialize, Serialize };
use strum_macros::{ EnumString, Display };
use uuid::Uuid;

const MAX_ERROR_ARGUMENTS: usize = 5;
pub const DECIMAL_PLACES_IN_COORDINATE: usize = 2;
pub const DECIMAL_PLACES_IN_REGION_BOUNDS: usize = 2;
const COORDINATES_IN_POINT: usize = 2;
const POINTS_IN_POLYGON: usize = 4;
pub const MAX_REGIONS: usize = 1;
pub const MAX_REGION_BOUNDS: usize = 4;
pub const MAX_MESSAGE_GEOTAGS: usize = 1;
const MAX_DOMAINS: usize = 10;
const MAX_STATUS_DETAILS: usize = 100;
const MAX_ROOM_SEARCH_RESPONSE_ITEMS: usize = 100;

// Classification strings
pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";


pub const HTTP_CREATE_MESSAGE_URL: &str = "/api/chatserver/messages";

// =============================================================================
// Error Messages

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct FieldErrorSchema {
    fieldName:          String,
    message:            String,
    messageArguments:   [String; MAX_ERROR_ARGUMENTS],
    messageCode:        String,
    rejectedValue:      String
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ErrorCode400 {
    classification: String,
    code:           i32,
    fieldErrors:    Vec<FieldErrorSchema>,
    message:        String
}

pub struct ErrorCode404 {
    classification: String,
    code:           i32,
    message:        String
}

// =============================================================================
// General Messages

/// This enum lists the possible values for a Domain's network ID.
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum NetworkId {
    #[strum(serialize = "bices")]
    bices,

    #[strum(serialize = "cxk")]
    cxk,

    #[strum(serialize = "sipr")]
    sipr,

    #[strum(serialize = "jwics")]
    jwics,

    #[strum(serialize = "unclass")]
    unclass,
}

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum JoinStatus {
    #[strum(serialize = "JOINED")]
    JOINED,

    #[strum(serialize = "NOT_JOINED")]
    NOT_JOINED,
}

//==============================================================================
// struct LocationCoordinatesSchema
//==============================================================================

/// The LocationCoordinates union is used for the "coordinates" field in the
/// "Location" struct to represent either a single geographic point, or a
/// set of points to define a polygon.
#[repr(C, packed)]
#[derive(Serialize, Deserialize)]
pub struct LocationCoordinatesSchema {
    #[serde(skip)]
    r#type:                 LocationType,

    // The first entry represents the coordinates for a single point.
    point_coordinates:      [f32; COORDINATES_IN_POINT],
    
    // The second entry represents a set of points for a polygon.
    polygon_coordinates:    [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON],

}

impl fmt::Display for LocationCoordinatesSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl LocationCoordinatesSchema {
    
    pub fn new_point(seed: f32) -> [f32; COORDINATES_IN_POINT] {
        let point: [f32; COORDINATES_IN_POINT] = [seed; COORDINATES_IN_POINT];
    
        // Return the newly constructed point.
        point
    }

    pub fn new_polygon(seed: f32) -> [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] {
        let polygon: [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] = [[seed; COORDINATES_IN_POINT]; POINTS_IN_POLYGON];

        // Return the newly constructed polygon.
        polygon
    }

    pub fn new() -> LocationCoordinatesSchema {
        LocationCoordinatesSchema {
            r#type: LocationType::Point,
            point_coordinates: [0.0; COORDINATES_IN_POINT],
            polygon_coordinates: [[0.0; COORDINATES_IN_POINT]; POINTS_IN_POLYGON]
        }
    }

    pub fn init(seed: f32, r#type: &LocationType) -> LocationCoordinatesSchema {
        match r#type {
            LocationType::Point => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Point,

                    point_coordinates: LocationCoordinatesSchema::new_point(seed),

                    // Zeroize the alternate coordinate structure.
                    polygon_coordinates: [[0.0; COORDINATES_IN_POINT]; POINTS_IN_POLYGON]
                }
            }
            LocationType::Polygon => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Polygon,

                    polygon_coordinates: LocationCoordinatesSchema::new_polygon(seed),
                    
                    // Zeroize the alternate coordinate structure.
                    point_coordinates: [0.0; COORDINATES_IN_POINT]
                }
            }
        }
    } //end new

    /*
     * This method constructs a JSON string from the LocationCoordinateSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        let mut point_index: usize = 0;
        let mut polygon_index: usize = 0;
        let mut value: f32;
        let mut value_string: String;
        let mut point_string: String;
        let mut polygon_string: String;
        let json_string: String;

        //======================================================================
        // Format point_coordinates field.

        // In order to get the commas correct, we need to handle the first
        // element specially.
        value = self.point_coordinates[point_index];
        value_string = format!("{:.2}", value);

        point_string = format!("{}", value_string);
        point_index += 1;

        // Concatenate the point values into one string.
        while point_index < COORDINATES_IN_POINT {
            value = self.point_coordinates[point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{},{}", point_string, value_string);

            point_index += 1;
        } //end point loop

        // Apply the initial JSON formatting for the point_coordinates field
        // string.
        json_string = format!("{{\"point_coordinates\":[{}],", point_string);

        //======================================================================
        // Format polygon_coordinates field.

        point_index = 0;

        // In order to get the commas correct, we need to handle the first array
        // specially.
        value = self.polygon_coordinates[polygon_index][point_index];
        value_string = format!("{:.2}", value);

        point_string = format!("{}", value_string);
        point_index += 1;

        // Concatenate the point values into one string.
        while point_index < COORDINATES_IN_POINT {
            value = self.polygon_coordinates[polygon_index][point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{},{}", point_string, value_string);
            point_index += 1;
        } //end point loop

        polygon_string = format!("[{}]", point_string);
        point_index = 0;
        polygon_index += 1;

        // For each point in the polygon...
        while polygon_index < POINTS_IN_POLYGON {
            // In order to get the commas correct, we need to handle the first array
            // specially.
            value = self.polygon_coordinates[polygon_index][point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{}", value_string);
            point_index += 1;

            // Concatenate the point values into one string.
            while point_index < COORDINATES_IN_POINT {
                value = self.polygon_coordinates[polygon_index][point_index];
                value_string = format!("{:.2}", value);

                point_string = format!("{},{}", point_string, value_string);

                point_index += 1;
            } //end point loop

            polygon_string = format!("{},[{}]", polygon_string, point_string);
            point_index = 0;
            polygon_index += 1;
        } //end polygon loop

        // Complete the JSON formatting now that we constructed the string
        // for the polygon_coordinates field.
        format!("{}\"polygon_coordinates\":[{}]}}", json_string, polygon_string)
    } //end to_json
}

#[derive(Debug, PartialEq, EnumString, Display)]
#[derive(Serialize, Deserialize)]
pub enum LocationType {
    #[strum(serialize = "Point")]
    Point,

    #[strum(serialize = "Polygon")]
    Polygon,
}

/*
 * Define the default value for the LocationType enum.
 */
impl Default for LocationType {
    fn default() -> Self { LocationType::Point }
}

//==============================================================================
// struct LocationSchema
//==============================================================================

/// The Location struct represent a particular geographic location relevant
/// to a particular chat message.
#[derive(Serialize, Deserialize)]
pub struct LocationSchema {
    pub coordinates:    LocationCoordinatesSchema,
    pub r#type:         LocationType
}

impl fmt::Display for LocationSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl LocationSchema {
    /*
     * This method constructs a new LocationSchema object using default
     * values.  Both the "point_coordinates" and "polygon_coordinates" arrays
     * in the LocationCoordinatesSchema field will be populated with the same
     * default value.
     * The LocationType field will be initialized to the Point value.
     */
    pub fn new() -> LocationSchema {
        LocationSchema {
            coordinates:    LocationCoordinatesSchema::new(),
            r#type:         LocationType::Point
        }
    }

    /*
     * This method constructs a new LocationSchema object using the
     * LocationType value specified by the "new_type" parameter.
     * Both the "point_coordinates" and "polygon_coordinates" arrays
     * in the LocationCoordinatesSchema field will be populated with the same
     * default value.
     */
    pub fn from_type(new_type: LocationType) -> LocationSchema {
        LocationSchema {
            coordinates:    LocationCoordinatesSchema::init(0.0, &new_type),
            r#type:         new_type
        }
    }

    pub fn init
    (
        coord_value:    f32,
        new_type:       LocationType
    ) -> LocationSchema {
        LocationSchema {
            coordinates:    LocationCoordinatesSchema::init(coord_value, &new_type),
            r#type:         new_type
        }
    }

    /*
     * This method constructs a JSON string from the LocationSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        format!("{{\"coordinates\":{},\"type\":{}}}", self.coordinates, self.r#type)
    }
}

//==============================================================================
// struct RegionSchema
//==============================================================================

/// The Region struct describes a notable geographic area with identifying
/// information.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct RegionSchema {
    pub abbreviation:   String,
    pub bounds:         [f32; MAX_REGION_BOUNDS],
    pub description:    String,
    pub name:           String,
    pub regionType:     String
}

impl fmt::Display for RegionSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl RegionSchema {
    /*
     * This method constructs a new RegionSchema object with default values.
     */
    pub fn new() -> RegionSchema {
        RegionSchema {
            abbreviation:   String::from(""),
            bounds:         [0.0; MAX_REGION_BOUNDS],
            description:    String::from(""),
            name:           String::from(""),
            regionType:     String::from(""),
        }
    }

    /*
     * This method constructs a new RegionSchema object for testing using the
     * given floating point value as an initial value.
     */
    pub fn new_test(seed: f32) -> RegionSchema {
        RegionSchema {
            abbreviation:   String::from("us"),
            bounds:         [seed; MAX_REGION_BOUNDS],
            description:    String::from(format!(
                                "This region {} is for testing.",
                                seed)),
            name:           String::from(format!("Test region {}", seed)),
            regionType:     String::from("Country")
        }
    }

    /*
     * This method constructs a JSON string from the RegionSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

//==============================================================================
// struct GeoTagSchema
//==============================================================================

/// The GeoTag struct allows context information to be added to a chat message.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GeoTagSchema {
    pub anchorEnd:      i64,
    pub anchorStart:    i64,
    pub anchorText:     String,
    pub confidence:     f32,
    pub location:       LocationSchema,
    pub regions:        [RegionSchema; MAX_REGIONS],
    pub r#type:         String
}

impl fmt::Display for GeoTagSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GeoTagSchema {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

//==============================================================================
// struct ChatMessageSchema
//==============================================================================

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ChatMessageSchema {
    pub classification: String,
    pub domainId:       String,
    pub geoTags:        [GeoTagSchema; MAX_MESSAGE_GEOTAGS],
    pub id:             Uuid,
    pub roomName:       String,
    pub sender:         String,
    pub text:           String,
    pub threadId:       Uuid,
    pub timestamp:      String,
    pub userId:         Uuid
}

impl fmt::Display for ChatMessageSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl ChatMessageSchema {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub struct ScrubbedChatMessage {
    classification: String,
    messageId:      Uuid,
    scrubDate:      DateTime<Utc>
}

// =============================================================================
// struct GetChatMessagesResponse
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct GetChatMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>
}

impl fmt::Display for GetChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GetChatMessagesResponse {
    pub fn new() -> GetChatMessagesResponse {
        GetChatMessagesResponse {
            classification: String::from(UNCLASSIFIED_STRING),
            messages:       Vec::new()
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// Domain Messages
#[derive(Serialize, Deserialize)]
pub struct ChatDomainSchema {
    classification: String,
    id:             String,
    name:           String,
    networkId:      String
}

pub struct GetChatDomainsResponse {
    classification: String,
    domains:        [ChatDomainSchema; MAX_DOMAINS]
}

// =============================================================================
// Room Messages

#[allow(non_snake_case)]
pub struct RoomSearchResponseRoomItemSchema {
    classification:         String,
    description:            String,
    displayName:            String,
    domainAbbreviation:     String,
    domainId:               String,
    domainName:             String,
    firstJoinedDate:        DateTime<Utc>,
    firstSeenDate:          DateTime<Utc>,
    isMembersOnly:          bool,
    isPasswordProtected:    bool,
    isPersistent:           bool,
    joinStatus:             JoinStatus,
    lastJoinedDate:         DateTime<Utc>,
    lastSeenDate:           DateTime<Utc>,
    networkId:              String,
    roomName:               String,
    statusDetail:           [String; MAX_STATUS_DETAILS],
    topic:                  String
}

#[allow(non_snake_case)]
pub struct SearchRoomsResponse {
    classification: String,
    rooms:          [RoomSearchResponseRoomItemSchema; MAX_ROOM_SEARCH_RESPONSE_ITEMS],
    totalRoomCount: i64
}

// =============================================================================
// struct KeywordFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct KeywordFilter {
    pub query: String
}

/*
 * Implement the trait fmt::Display for the struct KeywordFilter
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for KeywordFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl KeywordFilter {
    /*
     * This method constructs a JSON string from the KeywordFilter's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct TimeFilter
// =============================================================================
/*
 * This struct contains fields that can be used as filters when searching
 * for chat messages within a ChatSurfer chat room.
 * 
 * Each field in this struct is considered an optional parameter from
 * ChatSurfer's perspective.  So when determining the validity of a search
 * request, these fields should be allowed to be ignored.
 */
#[derive(Serialize, Deserialize)]
pub struct TimeFilter {
    endDateTime:        String, //This string needs to be in DateTime format.
    lookBackDuration:   String,
    startDateTime:      String, //This string needs to be in DateTime format.
}

impl Default for TimeFilter {
    fn default() -> Self {
        TimeFilter {
            endDateTime: String::new(),
            lookBackDuration: String::new(),
            startDateTime: String::new(),
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct TimeFilter
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for TimeFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl TimeFilter {
    pub fn is_valid(&self) -> bool {
        let valid_end = match self.endDateTime.parse::<DateTime<Utc>>() {
            Ok(_) => true,
            Err(_) => false
        };

        let valid_start = match self.startDateTime.parse::<DateTime<Utc>>() {
            Ok(_) => true,
            Err(_) => false
        };

        #[allow(unused_braces)]
        (valid_end && valid_start)
    }

    /*
     * This method constructs a JSON string from the TimeFilter's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct RoomFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
struct RoomFilterDomainProperties {
    properties: Vec<String>,
}

impl RoomFilterDomainProperties {
    pub fn from_vec(new_properties: Vec<String>) -> RoomFilterDomainProperties {
        RoomFilterDomainProperties {
            properties: new_properties
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RoomFilter {
    domains: HashMap<String, RoomFilterDomainProperties>,
}

impl RoomFilter {
    // pub fn add_domain
    // (
    //     &self,
    //     domainId:   &str,
    //     names:      Vec<String>
    // ) {
    //     self.domains.insert(String::from(domainId), names);
    // }

    // pub fn new() -> RoomFilter {
    //     RoomFilter {
    //         domains:    HashMap::new(),
    //     }
    // }
}

// =============================================================================
// struct SearchChatMessagesRequest
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesRequest {
    pub keywordFilter:  KeywordFilter,
    pub limit:          i32,
    //roomFilter:     RoomFilter,
    //pub timeFilter:     TimeFilter,
}

/*
 * Implement the trait fmt::Display for the struct SearchChatMessagesRequest
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SearchChatMessagesRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SearchChatMessagesRequest {
    
    pub fn from_string(json: String) -> SearchChatMessagesRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }
    
    pub fn from_str(json: &str) -> SearchChatMessagesRequest {
        serde_json::from_str(json).unwrap()
    }

    /*
     * This method constructs a JSON string from the SearchChatMessagesRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct SearchChatMessagesResponse
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesResponse {
    pub classification:     String,
    pub messages:           Vec<ChatMessageSchema>,
    pub nextCursorMark:     String,
    pub searchTimeFiler:    TimeFilter,
    pub total:              i32,
}

/*
 * Implement the trait fmt::Display for the struct SearchChatMessagesResponse
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SearchChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SearchChatMessagesResponse {
    /*
     * This method constructs a JSON string from the
     * SearchChatMessagesResponse's fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct SendChatMessageRequest
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct SendChatMessageRequest {
    pub classification: String,
    pub domainId:       String,
    pub message:        String,
    pub nickname:       String,
    pub roomName:       String
}

/*
 * Implement the trait Default for the struct SendChatMessageRequest
 * so that we can fall back on default values.
 */
impl Default for SendChatMessageRequest {
    fn default() -> SendChatMessageRequest {
        SendChatMessageRequest {
            classification: String::from(UNCLASSIFIED_STRING),
            domainId:       String::new(),
            message:        String::new(),
            nickname:       String::from("Edge View"),
            roomName:       String::new()
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct SendChatMessageRequest
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SendChatMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SendChatMessageRequest {
    /*
     * This constant represents the HTTP status code that ChatSurfer returns
     * upon a successful Create Chat Message operation.
     */
    pub const SUCCESSFUL: http::status::StatusCode = status::StatusCode::NO_CONTENT;

    /*
     * This method constructs a JSON string from the
     * SendChatMessageRequest's fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
} //end SendChatMessageRequest

#[derive(Serialize, Deserialize)]
pub enum CreateMessageResponse {
    Success204(),
    Failure400(ErrorCode400),
    Failure429()
}