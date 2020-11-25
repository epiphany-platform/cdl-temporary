use error::ClientError;
use schema::{query_client::QueryClient, ObjectIds, SchemaId};
use std::collections::HashMap;
use tonic::transport::Channel;

pub mod druid;
pub mod ds;
pub mod error;
pub mod psql;

pub mod schema {
    tonic::include_proto!("query");
}

pub async fn connect(addr: String) -> Result<QueryClient<Channel>, ClientError> {
    QueryClient::connect(addr)
        .await
        .map_err(ClientError::ConnectionError)
}

pub async fn query_multiple(
    object_ids: Vec<String>,
    addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_multiple(ObjectIds { object_ids })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().values)
}

pub async fn query_by_schema(
    schema_id: String,
    addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().values)
}

pub async fn query_raw(
    _raw_msg: String,
    _addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    // dummy mockup

    Ok(HashMap::new())
}
