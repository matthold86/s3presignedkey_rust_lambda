use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
use serde_json::json;
use std::error::Error;
use tracing::info;
use chrono::Utc;

extern crate rusoto_core;
extern crate rusoto_s3;

use std::error::Error;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3Client, S3};

fn generate_presigned_url(bucket_name: &str, object_name: &str, expiration: i64) -> Result<String, Box<dyn Error>> {
    let region = Region::default();
    let s3_client = S3Client::new(region);

    let request = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: object_name.to_owned(),
        ..Default::default()
    };

    let url = s3_client.get_presigned_url(&request, expiration);

    Ok(url)
}

async fn lambda_handler(event: Request, _: Context) -> Result<Response, Box<dyn Error>> {
    match event.method().as_str() {
        "OPTIONS" => {
            let mut response = Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type, X-Amz-Date, Authorization, X-Api-Key")
                .body(json!({"message": "CORS preflight response"}).to_string())?;
            Ok(response)
        }
        _ => {
            let body = event.body();
            let body_str = match body {
                Some(body) => String::from_utf8_lossy(body.as_ref()).to_string(),
                None => String::new(),
            };
            let parsed_body: serde_json::Value = serde_json::from_str(&body_str)?;

            let bar_name = parsed_body["barName"].as_str().unwrap_or_default();
            let drink_name = parsed_body["drinkName"].as_str().unwrap_or_default();

            let bucket_name = "cocktail-recommendations";
            let object_name = format!("cocktail-pictures/{}/{}.jpg", bar_name, drink_name);
            let expiration = 3600;

            let presigned_url = generate_presigned_url(bucket_name, &object_name, expiration)?;

            let response_body = json!({ "url": presigned_url });

            let mut response = Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(response_body.to_string())?;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda!(lambda_handler);
    Ok(())
}