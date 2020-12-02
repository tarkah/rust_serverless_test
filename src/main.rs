use lambda_http::{handler, http::Method, lambda, Context, IntoResponse, Request, RequestExt};
use serde::{Deserialize, Serialize};
use sqlx::{aurora::AuroraConnectOptions, AuroraConnection, ConnectOptions};

use std::env;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("rust_test", log::LevelFilter::Debug)
        .with_module_level("sqlx::query", log::LevelFilter::Debug)
        .init()?;

    lambda::run(handler(wrapper)).await?;

    Ok(())
}

fn log_error(error: &Error) {
    log::error!("{}", error);
}

async fn wrapper(request: Request, ctx: Context) -> Result<impl IntoResponse, Error> {
    let method = request.method();

    let result = match *method {
        Method::GET => get(request, ctx).await,
        Method::POST => post(request, ctx).await,
        _ => {
            return Ok(serde_json::to_value(BadRequest {
                error: "Invalid method".to_string(),
            })?);
        }
    };

    match result {
        Ok(response) => Ok(response),
        Err(e) => {
            log_error(&e);

            Ok(serde_json::to_value(BadRequest {
                error: format!("{}", e),
            })?)
        }
    }
}

async fn post(request: Request, _: Context) -> Result<serde_json::Value, Error> {
    log::info!("Post received");

    let message: Message = request.payload()?.ok_or("Empty body")?;

    log::info!("Message is: {}", &message.message);

    Ok(serde_json::to_value(message)?)
}

async fn get(_: Request, _: Context) -> Result<serde_json::Value, Error> {
    log::info!("Get received");

    let addons = test_rds().await?;

    Ok(serde_json::to_value(addons)?)
}

async fn test_rds() -> Result<AddonResponse, Error> {
    let resource_arn = env::var("AURORA_DB_RESOURCE_ARN")?;
    let secret_arn = env::var("AURORA_DB_SECRET_ARN")?;
    let region = env::var("AURORA_DB_REGION")?;

    let mut connection: AuroraConnection = AuroraConnectOptions::new()
        .region(&region)
        .resource_arn(&resource_arn)
        .secret_arn(&secret_arn)
        .database("addons")
        .log_statements(log::LevelFilter::Debug)
        .connect()
        .await?;

    let addons = sqlx::query_as::<_, Addon>(
        "
    SELECT id,
        repository,
        repository_name,
        source,
        description,
        homepage,
        image_url,
        owner_image_url,
        owner_name,
        total_download_count,
        updated_at           
    FROM addon
    ",
    )
    .fetch_all(&mut connection)
    .await?;

    let count = addons.len();

    Ok(AddonResponse { addons, count })
}

#[derive(Deserialize, Serialize)]
struct Message {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct BadRequest {
    error: String,
}

#[derive(Serialize)]
struct AddonResponse {
    addons: Vec<Addon>,
    count: usize,
}

#[derive(Serialize, sqlx::FromRow)]
struct Addon {
    id: i64,
    repository: String,
    repository_name: String,
    source: String,
    description: Option<String>,
    homepage: Option<String>,
    image_url: Option<String>,
    owner_image_url: Option<String>,
    owner_name: Option<String>,
    total_download_count: i64,
    updated_at: String,
}
