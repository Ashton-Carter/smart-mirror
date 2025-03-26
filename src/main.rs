mod weather_api;
mod calendar_api;
mod chat;
mod text_to_speech;

use axum::{Router, routing::{get, post}, http::Method};
use tower_http::cors::{CorsLayer, AllowMethods, AllowHeaders, Any};

use std::{sync::{Arc, Mutex}};

mod state;
use state::AppState;

#[tokio::main]
async fn main() {
    
    
    let app_state = AppState {
        messages: Arc::new(Mutex::new(Vec::<serde_json::Value>::new())),
    };
    {
        let message_to_add = chat::to_json_message("sys", "sys");
        let curr_date = format!("Current Date:{}", chrono::prelude::Local::now());
        let curr_date_system = chat::to_json_message("system", curr_date.as_str());
        let mut messages_lock = app_state.messages.lock().unwrap();
        messages_lock.push(message_to_add);
        messages_lock.push(curr_date_system);
    }
    let cors_layer = CorsLayer::new()
        .allow_methods(AllowMethods::list(vec![Method::GET, Method::POST, Method::OPTIONS]))
        .allow_headers(AllowHeaders::list(vec![
            http::header::HeaderName::from_static("content-type")
        ]))
        .allow_origin(Any);

    let router = Router::new()
        .route("/weather", get(weather_api::get_weather_json))
        .route("/calendar", get(calendar_api::get_calendar_json)) 
        .route("/chat", post(text_to_speech::return_audio))
        .layer(cors_layer)
        .with_state(app_state.clone()); 

    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 3000);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}





