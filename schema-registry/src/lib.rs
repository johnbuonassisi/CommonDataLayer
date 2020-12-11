#![feature(drain_filter)]

use ::rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use ::rpc::schema_registry::{Empty, Id, ValueToValidate};
use error::RegistryClientError;
use serde_json::Value;
use tonic::{transport::Channel, Request};
use types::storage::vertices::View;
use uuid::Uuid;

pub mod db;
pub mod error;
pub mod replication;
pub mod rpc;
pub mod schema;
pub mod types;

pub async fn connect_to_registry(
    addr: String,
) -> Result<SchemaRegistryClient<Channel>, RegistryClientError> {
    SchemaRegistryClient::connect(addr)
        .await
        .map_err(RegistryClientError::ConnectionError)
}

pub async fn validate_data_with_schema(
    schema_id: Uuid,
    json: &Value,
    schema_registry_addr: String,
) -> Result<(), RegistryClientError> {
    let mut client = connect_to_registry(schema_registry_addr).await?;
    let request = Request::new(ValueToValidate {
        schema_id: schema_id.to_string(),
        value: serde_json::to_string(json).map_err(RegistryClientError::JsonError)?,
    });

    let errors = client.validate_value(request).await?.into_inner().errors;

    if errors.is_empty() {
        Ok(())
    } else {
        Err(RegistryClientError::InvalidData(errors))
    }
}

pub async fn get_view_of_data(
    view_id: Uuid,
    data: &Value,
    schema_registry_addr: String,
) -> Result<Value, RegistryClientError> {
    let mut client = connect_to_registry(schema_registry_addr).await?;
    let request = Request::new(Id {
        id: view_id.to_string(),
    });
    let view = client.get_view(request).await?.into_inner();

    let path = jmespatch::compile(&view.jmespath).map_err(RegistryClientError::JmespathError)?;
    let mapped = path
        .search(data)
        .map_err(RegistryClientError::JmespathError)?;

    serde_json::to_value(&mapped).map_err(|_err| RegistryClientError::JmespathReturnedMalformedJson)
}

pub async fn promote_to_master(storage_addr: String) -> Result<String, RegistryClientError> {
    let mut client = connect_to_registry(storage_addr).await?;
    let response = client.promote_to_master(Request::new(Empty {})).await?;

    Ok(response.into_inner().name)
}

pub async fn heartbeat(storage_addr: String) -> Result<(), RegistryClientError> {
    let mut client = connect_to_registry(storage_addr).await?;
    client.heartbeat(Request::new(Empty {})).await?;

    Ok(())
}
