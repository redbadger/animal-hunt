use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use shared::{AnimalHunt, Capabilities, Core, Effect, Event, Mode, ViewModel};

async fn landing_page() -> &'static str {
    "Hello, World! Happy hunting!"
}

async fn animal_page(
    Path(animal): Path<String>,
    State(state): State<Vec<(String, String)>>,
) -> (StatusCode, String) {
    state
        .iter()
        .find(|(name, _)| name.to_lowercase() == animal.to_lowercase())
        .map(|(name, emoji)| {
            (
                StatusCode::OK,
                format!("You found a {}: {}! Congrats!", name, emoji),
            )
        })
        .unwrap_or((
            StatusCode::NOT_FOUND,
            "Sorry, I don't know that animal".to_string(),
        ))
}

async fn apple_app_site_association() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        include_str!("../public/apple-app-site-association.json"),
    )
}

#[tokio::main]
async fn main() {
    let animal_hunt: Core<Effect, AnimalHunt> = Core::new::<Capabilities>();
    animal_hunt.process_event(Event::SetMode(Mode::Configure));

    let ViewModel::Configure { known_animals: animals, error: _ } = animal_hunt.view() else {
        panic!("Expected Configure mode");
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/animal/:animal", get(animal_page))
        .route(
            "/.well-known/apple-app-site-association",
            get(apple_app_site_association),
        )
        .with_state(animals);

    // run our app with hyper, listening globally on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
