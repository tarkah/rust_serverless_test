use lambda_http::{handler, http::Method, lambda, Context, IntoResponse, Request, RequestExt};
use rusoto_rds_data::{ExecuteStatementRequest, RdsData, RdsDataClient};
use rusoto_signature::Region;
use serde::{Deserialize, Serialize};

use std::env;
use std::str::FromStr;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("rust_test", log::LevelFilter::Debug)
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

    let client = RdsDataClient::new(Region::from_str(&region)?);

    let statement = ExecuteStatementRequest {
        resource_arn,
        secret_arn,
        database: Some("addons".to_string()),
        sql: "SELECT id, repository, total_download_count from addon".to_string(),
        ..Default::default()
    };

    let records = client
        .execute_statement(statement)
        .await?
        .records
        .ok_or("No records returned")?;

    let mut addons = vec![];

    for row in records {
        let mut id = None;
        let mut repository = None;
        let mut total_download_count = None;

        for (idx, field) in row.iter().enumerate() {
            match idx {
                0 => id = field.long_value,
                1 => repository = field.string_value.clone(),
                2 => total_download_count = field.long_value,
                _ => {}
            }
        }

        if let (Some(id), Some(repository), Some(total_download_count)) =
            (id, repository, total_download_count)
        {
            let addon = Addon {
                id,
                repository,
                total_download_count,
            };

            addons.push(addon);
        }
    }

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

#[derive(Serialize)]
struct Addon {
    id: i64,
    repository: String,
    total_download_count: i64,
}
