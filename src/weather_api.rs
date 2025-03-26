use serde::{Deserialize, Serialize};
use reqwest::Error;
use std::env;
use dotenv::dotenv;

#[derive(Deserialize, Serialize, Debug)]
pub struct WeatherResponse{
    pub location: LocationResponse,
    pub current: CurrentResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationResponse{
    pub name: String,
    pub region: String,
    pub country: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CurrentResponse{
    pub temp_f: f64,
    pub condition: Condition,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Condition{
    pub text: String,
    pub icon: String,
}


pub async fn get_weather(location: &str)->Result<WeatherResponse, Error> {
    dotenv().ok();
    let api_key = env::var("WEATHER_API_KEY").expect("Missing WEATHER_API_KEY in .env");
    let request_url = format!("http://api.weatherapi.com/v1/current.json?key={api_key}&q={location}&aqi=no");
    let client = reqwest::Client::new();
    let response: WeatherResponse = client.get(request_url).send().await?.json().await?;
    Ok(response)
}

pub async fn get_weather_json(axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>) -> axum::Json<WeatherResponse> {
    let location: &str = params.get("location").unwrap();
    axum::Json(get_weather(&location).await.unwrap())
}
