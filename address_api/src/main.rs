use axum::{
    extract::Query,
    http::Method,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressSuggestion {
    pub street: String,
    pub city: String,
    pub postcode: String,  // Changed from zip_code to postcode for UK
    pub country: String,
    pub combined: String,  // New field for combined display
}

impl AddressSuggestion {
    pub fn new(street: &str, city: &str, postcode: &str, country: &str) -> Self {
        let combined = format!("{}, {}, {} {}", street, city, postcode, country);
        Self {
            street: street.to_string(),
            city: city.to_string(),
            postcode: postcode.to_string(),
            country: country.to_string(),
            combined,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AddressQuery {
    query: String,
}

// Hardcoded list of London address suggestions
fn get_all_suggestions() -> Vec<AddressSuggestion> {
    vec![
        // Central London
        AddressSuggestion::new("10 Downing Street", "London", "SW1A 2AA", "UK"),
        AddressSuggestion::new("221B Baker Street", "London", "NW1 6XE", "UK"),
        AddressSuggestion::new("30 St Mary Axe", "London", "EC3A 8BF", "UK"), // The Gherkin
        AddressSuggestion::new("20 Fenchurch Street", "London", "EC3M 3BY", "UK"), // The Walkie Talkie
        AddressSuggestion::new("122 Leadenhall Street", "London", "EC3V 4AB", "UK"), // The Cheesegrater
        
        // West End
        AddressSuggestion::new("1 Piccadilly Circus", "London", "W1J 0DA", "UK"),
        AddressSuggestion::new("15 Regent Street", "London", "SW1Y 4LR", "UK"),
        AddressSuggestion::new("28 Oxford Street", "London", "W1D 2AU", "UK"),
        AddressSuggestion::new("40 Bond Street", "London", "W1S 2QP", "UK"),
        AddressSuggestion::new("55 Carnaby Street", "London", "W1F 9QL", "UK"),
        
        // City of London
        AddressSuggestion::new("1 Poultry", "London", "EC2R 8EJ", "UK"),
        AddressSuggestion::new("25 Old Street", "London", "EC1V 9HL", "UK"),
        AddressSuggestion::new("42 Threadneedle Street", "London", "EC2R 8AY", "UK"),
        AddressSuggestion::new("60 Lombard Street", "London", "EC3V 9EA", "UK"),
        AddressSuggestion::new("88 Wood Street", "London", "EC2V 7RS", "UK"),
        
        // Canary Wharf
        AddressSuggestion::new("1 Canada Square", "London", "E14 5AB", "UK"),
        AddressSuggestion::new("25 Bank Street", "London", "E14 5JP", "UK"),
        AddressSuggestion::new("40 Marsh Wall", "London", "E14 9TP", "UK"),
        AddressSuggestion::new("10 Upper Bank Street", "London", "E14 5BB", "UK"),
        AddressSuggestion::new("5 Churchill Place", "London", "E14 5HU", "UK"),
        
        // South Bank
        AddressSuggestion::new("1 Southwark Bridge", "London", "SE1 9HL", "UK"),
        AddressSuggestion::new("20 Blackfriars Road", "London", "SE1 8NW", "UK"),
        AddressSuggestion::new("35 London Bridge Street", "London", "SE1 9SG", "UK"),
        AddressSuggestion::new("50 Southwark Street", "London", "SE1 1UN", "UK"),
        AddressSuggestion::new("65 Borough High Street", "London", "SE1 1LL", "UK"),
        
        // East London
        AddressSuggestion::new("1 Brick Lane", "London", "E1 6QL", "UK"),
        AddressSuggestion::new("25 Commercial Street", "London", "E1 6LP", "UK"),
        AddressSuggestion::new("40 Spitalfields", "London", "E1 6EW", "UK"),
        AddressSuggestion::new("55 Shoreditch High Street", "London", "E1 6JJ", "UK"),
        AddressSuggestion::new("70 Old Street", "London", "EC1V 9BD", "UK"),
    ]
}

async fn get_suggestions(Query(params): Query<AddressQuery>) -> axum::Json<Vec<AddressSuggestion>> {
    let query = params.query.to_lowercase();
    let suggestions = get_all_suggestions()
        .into_iter()
        .filter(|addr| {
            addr.street.to_lowercase().contains(&query)
                || addr.city.to_lowercase().contains(&query)
                || addr.postcode.to_lowercase().contains(&query)
                || addr.combined.to_lowercase().contains(&query)
        })
        .take(5)  // Limit to first 5 results
        .collect();

    axum::Json(suggestions)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any)
        .allow_headers(Any);

    // Build our application with a route
    let app = Router::new()
        .route("/api/suggestions", get(get_suggestions))
        .layer(cors);

    // Run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
} 