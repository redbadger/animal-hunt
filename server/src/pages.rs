mod templates;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use maud::html;

use templates::{animal_template, layout};

pub async fn landing_page() -> Html<String> {
    Html(
        layout(
            "Hello",
            html! {
                h1 { "Hunting for some animals, are you?" }
            },
        )
        .into_string(),
    )
}

pub async fn animal_page(
    Path(animal): Path<String>,
    State(state): State<Vec<(String, String)>>,
) -> (StatusCode, Html<String>) {
    state
        .iter()
        .find(|(name, _)| name.to_lowercase() == animal.to_lowercase())
        .map(|(name, emoji)| {
            (
                StatusCode::OK,
                Html(layout(name, animal_template(name, emoji)).into_string()),
            )
        })
        .unwrap_or((
            StatusCode::NOT_FOUND,
            Html(
                layout(
                    "Not found",
                    html! {
                        h1 { "Sorry, I don't know that animal" }
                    },
                )
                .into_string(),
            ),
        ))
}

pub async fn apple_app_site_association() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        include_str!("../public/apple-app-site-association.json"),
    )
}
