
use axum::{Json};
use serde::{Deserialize, Serialize};
use std::env;
use reqwest::Client;


#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    pub command: String,
    pub parameters: serde_json::Value,
    pub text: String,
}

static SYSTEM_MSG: &str = r#"You are a helpful AI for a smart mirror. Possible commands:
- "get_events": parameters={} => will return the next week of events to you(use if user asks for events)
- "play_song": parameters={"song":"..."} => aggregator will embed a YT link
- "add_event": parameters={"event_name":"...","date":"yyyy-mm-dd",} => aggregator calls Google Calendar
- "get_weather": parameters={"location":"..."} => aggregator fetches weather
- "none" => no special action

Important: The next command after get_events or get_weather should be none!

Always respond in strict JSON: {"command":"...","parameters":{...},"text":"..."}
Your output is given to a text to speech so please write it in a voice-friendly manner.
The text section is the only sections that is given to text_to_speech"#;


pub async fn handle_chat_request(messages: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>, Json(payload): Json<ChatRequest>) -> Json<ChatResponse> {
    dotenv::dotenv().ok();
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Missing OPENAI_API_KEY in .env");
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "No API key configured.".to_string(),
            });
        }
    };

    let messages_clone = {
        let mut messages_lock = messages.lock().unwrap();
        messages_lock.push(to_json_message("user", format!("{} \nREMEMBER RESPOND IN JSON ONLY", &payload.message).as_str()));
        messages_lock.clone()
    };

    // println!("\n\n\nPRE GPT VECTOR\n{:?}", &messages_clone);
    
    println!("PRE GPT RESPONSE: {:?}", &messages_clone);
    let client = Client::new();
    let response = match client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-4o",
            "messages": messages_clone,
            "temperature": 0.5
        }))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to contact OpenAI: {}", err);
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "Error contacting AI.".to_string(),
            });
        }
    };

    // println!("\n\n\n\nAI RESPONSE:{:?}", response);

    let json_val = match response.json::<serde_json::Value>().await {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to parse AI response: {}", err);
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "Could not parse AI response.".to_string(),
            });
        }
    };

    let ai_reply = match json_val["choices"].as_array() {
        Some(choices) if !choices.is_empty() => {
            choices[0]["message"]["content"].as_str().unwrap_or_default()
        },
        _ => {
            eprintln!("No valid 'choices' found in AI response: {:?}", json_val);
            ""
        }
    };

    println!("AI Reply: {}", ai_reply);

    let parsed_response: ChatResponse = serde_json::from_str(ai_reply).unwrap_or(ChatResponse {
        command: "none".to_string(),
        parameters: serde_json::json!({}),
        text: "I didn't understand that.".to_string(),
    });

    {
        let mut messages_lock = messages.lock().unwrap();
        messages_lock.push(to_json_message("assistant", &parsed_response.text.clone()));
    }

    

    if &parsed_response.command == "none" {
        return Json(parsed_response);
    }
    handle_command(parsed_response, messages.clone()).await

}


pub async fn handle_command(payload: ChatResponse, messages: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>) -> Json<ChatResponse> {
    dotenv::dotenv().ok();
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Missing OPENAI_API_KEY in .env");
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "No API key configured.".to_string(),
            });
        }
    };



    let return_str = match payload.command.as_str() {
        "get_weather" => {
            weather_string(&payload.parameters["location"].to_string()).await
        },
        "get_events" => {
            events_string().await
        },
        "add_event" => {
            let name = payload.parameters.get("event_name").unwrap().to_string()
                .trim_matches('"').to_string();

            let date = payload.parameters.get("date").unwrap().to_string()
              .trim_matches('"').to_string();
            println!("{}, {}", name, date);
        
            crate::calendar_api::add_event(crate::calendar_api::create_basic_event(name, date.clone(), date)).await.expect("COULDNT ADD EVENT");
            "Added Event".to_string()
        },
        _=> {
            eprintln!("NO MATCHING COMMANDS: {}", payload.command.as_str());
            "None".to_string()
        }, 
    };


    let user_msg = format!("Returned_Value:{}\nREMEMBER RESPOND IN JSON ONLY\n", &return_str);

    let messages_clone = {
        let mut messages_lock = messages.lock().unwrap();
        messages_lock.push(to_json_message("system", &user_msg));
        messages_lock.clone() 
    };


    let client = Client::new();
    let response = match client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-4o",
            "messages": messages_clone,
            "temperature": 0.5
        }))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to contact OpenAI: {}", err);
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "Error contacting AI.".to_string(),
            });
        }
    };

    
    let json_val = match response.json::<serde_json::Value>().await {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to parse AI response: {}", err);
            return Json(ChatResponse {
                command: "none".to_string(),
                parameters: serde_json::json!({}),
                text: "Could not parse AI response.".to_string(),
            });
        }
    };

    
    let ai_reply = json_val["choices"]
    .as_array()
    .and_then(|choices| choices.get(0))
    .and_then(|choice| choice["message"]["content"].as_str())
    .unwrap_or("No response from AI.");

    let parsed_response: ChatResponse = serde_json::from_str(ai_reply).unwrap_or(ChatResponse {
        command: "none".to_string(),
        parameters: serde_json::json!({}),
        text: "I didn't understand that.".to_string(),
    });

    {
        let mut messages_lock = messages.lock().unwrap();
        messages_lock.push(to_json_message("assistant", &parsed_response.text.clone()));
    }
    
    //if &parsed_response.command == "none" {
        return Json(parsed_response);
    //}

    //handle_command(parsed_response, &input).await
}

async fn weather_string(location: &str)-> String{
    let weather = crate::weather_api::get_weather(&location).await;
    String::from(&format!("Location:{}\nWeather:{} Degrees Farenheit\n", &weather.as_ref().unwrap().location.name, &weather.as_ref().unwrap().current.temp_f))
}

async fn events_string() -> String {
    let events = crate::calendar_api::get_calendar_events().await;
    let mut event_total = String::new();
    for event in events {
        let summary = event.summary.as_ref().unwrap();

        let date = match event.start.as_ref(){
            Some(start) => {
                match start.date.as_ref() {
                    Some(date) => {
                        date
                    }
                    None => {
                        start.date_time.as_ref().unwrap()
                    }
                }
            }
            None => {
                eprintln!("No Start!");
                "NO_START"
            }
        };
        
        event_total.push_str(&format!("Summary:{}\nDate:{}\n\n",
            summary,
            date
            )    
        )
    }
    event_total
}

pub fn to_json_message(role: &str, message: &str)->serde_json::Value{
    let to_json: serde_json::Value;
    if message == "sys" {
        to_json = serde_json::json!({ "role": "system", "content": &SYSTEM_MSG });
    } else {
        to_json = serde_json::json!({ "role": role, "content": message });
    }
    to_json
}