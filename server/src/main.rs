mod pages;

use axum::{routing::get, Router};
use shared::{AnimalHunt, Capabilities, Core, Effect, Event, Mode, ViewModel};

use pages::{animal_page, apple_app_site_association, landing_page};

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
        .route(
            "/robots.txt",
            get(|| async { "User-agent: *\nDisallow: /" }),
        )
        .with_state(animals);

    // run our app with hyper, listening globally on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
