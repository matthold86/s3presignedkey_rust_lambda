use lambda_http::{run, service_fn, Request, Response};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::error::Error;

extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_xray;

use rusoto_core::Region;
use rusoto_core::request::HttpClient;
use rusoto_s3::{PutObjectRequest};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_credential::{ChainProvider, ProvideAwsCredentials, StaticProvider};
use rusoto_xray::{XRayRecorder, PutTraceSegmentsRequest, XRay};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payload {
    pub bar_name: String,
    pub drink_name: String,
}


async fn generate_presigned_url(bucket_name: &str, object_name: &str, expiration: u64) -> Result<String, Box<dyn Error>> {
    let region = if let Ok(url) = std::env::var("AWS_ENDPOINT_URL") {
        Region::Custom {
            name: std::env::var("AWS_REGION").unwrap_or_else(|_| "custom".to_string()),
            endpoint: url,
        }
    } else {
        Region::default()
    };

    let provider = ChainProvider::new();
    let credentials = provider.credentials().await?;

    let options = PreSignedRequestOption {
        expires_in: std::time::Duration::from_secs(expiration),
    };

    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: object_name.to_owned(),
        ..Default::default()
    };

    let url = req.get_presigned_url(&region, &credentials, &options);
    Ok(url.to_string())
}

async fn lambda_handler(event: Request) -> Result<Response<String>, Box<dyn Error>> {
    match event.method().as_str() {
        "OPTIONS" => {
            let response = Response::builder()
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
            let s = std::str::from_utf8(body).expect("invalid utf-8 sequence");
            let payload = match serde_json::from_str::<Payload>(s) {
                Ok(item) => item,
                Err(err) => {
                    let resp = Response::builder()
                        .status(400)
                        .header("content-type", "application/json")
                        .body(err.to_string().into())
                        .map_err(Box::new)?;
                    return Ok(resp);
                }
            };
            let bar_name = payload.bar_name.clone();
            let drink_name = payload.drink_name.clone();

            let bucket_name = "cocktail-recommendations";
            let object_name = format!("cocktail-pictures/{}/{}.jpg", bar_name, drink_name);

            println!("{}", bucket_name);
            println!("{}", object_name);

            let expiration = 3600;

            let presigned_url = generate_presigned_url(bucket_name, &object_name, expiration).await?;

            let response_body = serde_json::to_string(&presigned_url)?;

            let response = Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(response_body.into())?;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(),  Box<dyn Error>> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Initialize the X-Ray recorder
    let recorder = XRayRecorder::default();

    // Initialize the X-Ray client
    let client = HttpClient::new().unwrap();
    let provider = StaticProvider::new(access_key_id.to_string(), secret_access_key.to_string(), None, None);
    let xray_client = XRayClient::new_with_client(client, provider, region);

    let response = recorder.record_trace(|recorder| {
        lambda_handler(event, &recorder, &xray_client).await
    }).await?;

    // Submit trace segments to X-Ray
    let trace_segments = recorder.trace_segments();
    let trace_segment_docs = trace_segments.iter().map(|segment| segment.document.clone()).collect();
    let put_trace_segments_req = PutTraceSegmentsRequest { trace_segment_documents: trace_segment_docs };
    xray_client.put_trace_segments(put_trace_segments_req).await?;

    println!("Response: {:?}", response);
    
    Ok(())
}