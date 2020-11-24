use schema::{query_client::QueryClient, DataPoint, Range, Tag};
use tonic::transport::Channel;
use utils::query_utils::error::ClientError;

pub mod victoria;
pub mod helper_types;

pub mod schema {
    tonic::include_proto!("query");
}

pub async fn connect(addr: String) -> Result<QueryClient<Channel>, ClientError> {
    QueryClient::connect(addr)
        .await
        .map_err(ClientError::ConnectionError)
}

pub async fn query_by_range(
    schema_id: String,
    start: String,
    end: String,
    step: f32,
    addr: String,
) -> Result<Vec<DataPoint>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_range(Range {
            schema_id,
            start,
            end,
            step,
        })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().datapoints)
}

pub async fn query_by_tag(tag_id: String, addr: String) -> Result<Vec<DataPoint>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_tag(Tag { tag_id })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().datapoints)
}