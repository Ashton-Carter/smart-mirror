
use axum::{Json, response::{Response}, extract::State};
use std::env;
use reqwest::Client;

use crate::state::AppState;

use crate::chat;


#[axum::debug_handler]
pub async fn return_audio(
    State(app_state): State<AppState>, 
    Json(payload): Json<chat::ChatRequest>
) -> Result<Response, http::StatusCode> {

    let chat_str = chat::handle_chat_request(
        app_state.messages.clone(),
        Json(chat::ChatRequest{message: payload.message})
    ).await;


    println!("Chat_Str:{}", &chat_str.text);


    let v_id = env::var("VOICE_ID").expect("NO VOICE ID");
    let api_key = env::var("ELEVENLABS_API_KEY").expect("NO 11L KEY");

    let client = Client::new();
    let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", v_id);
    let body = serde_json::json!({
        "text": chat_str.text,
        "voice_settings": {
            "stability": 0.5,
            "similarity_boost": 0.7
        }
    });

    let response = client.post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .unwrap();

    let audio_bytes = response.bytes().await.unwrap();

    let response = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "audio/mpeg")
        .body(axum::body::boxed(axum::body::Body::from(audio_bytes)))
        .unwrap();

    Ok(response)

}
