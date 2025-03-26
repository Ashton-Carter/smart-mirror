use google_calendar3::{CalendarHub, hyper, hyper_rustls, oauth2};
use axum::{Json};
use std::{fs, path::PathBuf};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use chrono::{DateTime, Utc, Duration};


#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub html_link: Option<String>,
    pub status: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    pub creator: Option<EventCreator>,
    pub organizer: Option<EventOrganizer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventDateTime {
    pub date: Option<String>,
    pub date_time: Option<String>, 
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventCreator {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventOrganizer {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

pub async fn get_calendar_events() -> Vec<CalendarEvent> {
    dotenv().ok();
    let creds_path = PathBuf::from(env::var("GOOGLE_CREDENTIALS_PATH").expect("Missing GOOGLE_CREDENTIALS_PATH"));

    let creds = match fs::read_to_string(&creds_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading credentials file: {}", e);
            return vec![];
        }
    };

    let creds_json: serde_json::Value = match serde_json::from_str(&creds) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error parsing credentials file: {}", e);
            return vec![];
        }
    };

    let secret: oauth2::ApplicationSecret = match serde_json::from_value(creds_json["installed"].clone()) {
        Ok(secret) => secret,
        Err(e) => {
            eprintln!("Error extracting OAuth secret: {}", e);
            return vec![];
        }
    };

    let auth = match oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect
    )
    .persist_tokens_to_disk("token.json")
    .build()
    .await
    {
        Ok(auth) => auth,
        Err(e) => {
            eprintln!("Error during authentication: {}", e);
            return vec![];
        }
    };

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let hub = CalendarHub::new(hyper::Client::builder().build(https), auth);

    let now: DateTime<Utc> = Utc::now();
    let next_week: DateTime<Utc> = now + Duration::days(7);
    let now_str = now.to_rfc3339();
    let next_week_str = next_week.to_rfc3339();

    let calendar_list = match hub.calendar_list().list().doit().await {
        Ok((_resp, calendar_list)) => calendar_list.items.unwrap_or_default(),
        Err(e) => {
            eprintln!("Error fetching calendar list: {}", e);
            return vec![];
        }
    };

    let mut all_events = Vec::new();

    for calendar in calendar_list {
        if let Some(calendar_id) = calendar.id {
            let result = hub
                .events()
                .list(&calendar_id)
                .time_min(&now_str)
                .time_max(&next_week_str)
                .max_results(100)
                .order_by("startTime")
                .single_events(true)
                .doit()
                .await;

            match result {
                Ok((_resp, events)) => {
                    if let Some(items) = events.items {
                        for event in items {
                            

                            all_events.push(CalendarEvent {
                                id: event.id,
                                summary: event.summary,
                                description: event.description,
                                html_link: event.html_link,
                                status: event.status,
                                start: event.start.map(|s| EventDateTime {
                                    date: s.date,
                                    date_time: s.date_time,
                                    time_zone: s.time_zone,
                                }),
                                end: event.end.map(|s| EventDateTime {
                                    date: s.date,
                                    date_time: s.date_time,
                                    time_zone: s.time_zone,
                                }),
                                creator: event.creator.map(|c| EventCreator {
                                    email: c.email,
                                    display_name: c.display_name,
                                }),
                                organizer: event.organizer.map(|o| EventOrganizer {
                                    email: o.email,
                                    display_name: o.display_name,
                                }),
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching events for calendar: {}", e);
                },
            }
        }
    }

    all_events
}


pub async fn get_calendar_json() -> Json<Vec<CalendarEvent>> {
    Json(get_calendar_events().await)
}

pub async fn add_event(event: google_calendar3::api::Event) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let event_json = serde_json::to_string(&event)?;
    println!("Sending event JSON: {}", event_json);
    let creds_path = PathBuf::from(env::var("GOOGLE_CREDENTIALS_PATH").expect("Missing GOOGLE_CREDENTIALS_PATH"));

    let creds = match fs::read_to_string(&creds_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading credentials file: {}", e);
            return Err(Box::new(e));
        }
    };

    let creds_json: serde_json::Value = match serde_json::from_str(&creds) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error parsing credentials file: {}", e);
            return Err(Box::new(e));
        }
    };

    let secret: oauth2::ApplicationSecret = match serde_json::from_value(creds_json["installed"].clone()) {
        Ok(secret) => secret,
        Err(e) => {
            eprintln!("Error extracting OAuth secret: {}", e);
            return Err(Box::new(e));
        }
    };

    let auth = match oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect
    )
    .persist_tokens_to_disk("token.json")
    .build()
    .await
    {
        Ok(auth) => auth,
        Err(e) => {
            eprintln!("Error during authentication: {}", e);
            return Err(Box::new(e));
        }
    };

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let hub = CalendarHub::new(hyper::Client::builder().build(https), auth);

    let result = hub.events().insert(event, "c_02f23b407241b05d9235403f1821745ee55848bcc35019ed95ca20c3551e5d5b@group.calendar.google.com").doit().await;

    match result {
        Ok((response, event)) => {
            println!("Event successfully created!");
            println!("HTTP Status: {}", response.status());
            println!("Event Link: {}", event.html_link.unwrap_or_default());
            println!("Server Response Headers: {:?}", response.headers());
        }
        Err(e) => {
            eprintln!("Error creating event: {}", e);
        }
    }

    Ok(())
}

pub fn create_basic_event(name: String, start: String, end: String) -> google_calendar3::api::Event {

    let event = google_calendar3::api::Event {
        summary: Some(name),
        description: Some("Made by MirrorAI".to_string()),
        visibility: Some("public".to_string()),
        start: Some(google_calendar3::api::EventDateTime {
            date: Some(start),
            ..Default::default()
        }),
        end: Some(google_calendar3::api::EventDateTime {
            date: Some(end),
            ..Default::default()
        }),
        ..Default::default()
    };
    return event;
}