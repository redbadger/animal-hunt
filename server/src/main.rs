use axum::{extract::Path, routing::get, Router};

async fn landing_page() -> &'static str {
    "Hello, World! Happy hunting!"
}

async fn animal_page(Path(animal): Path<String>) -> String {
    format!("You found a {}! Congrats!", animal)
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/animal/:animal", get(animal_page))
        .route(
            "/apple-app-site-association",
            get(|| async { include_str!("../public/apple-app-site-association") }),
        );

    // run our app with hyper, listening globally on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
