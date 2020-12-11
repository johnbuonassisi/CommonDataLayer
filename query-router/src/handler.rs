use crate::{
    cache::{AddressCache, SchemaTypeCache},
    error::Error,
};
use log::trace;
use schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Body {
    Range {
        from: String,
        to: String,
        step: String,
    },
    Empty {},
}

pub async fn query_single(
    object_id: Uuid,
    schema_id: Uuid,
    address_cache: Arc<AddressCache>,
    schema_type_cache: Arc<SchemaTypeCache>,
    request_body: Body,
) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("Received /single/{} (SCHEMA_ID={})", object_id, schema_id);

    let address = address_cache.get_address(schema_id).await?;
    let schema_type = schema_type_cache.get_schema_type(schema_id).await?;

    let values = match (schema_type, request_body) {
        (SchemaType::DocumentStorage, _) => {
            let mut values = query_service::query_multiple(vec![object_id.to_string()], address)
                .await
                .map_err(Error::QueryError)?;

            values
                .remove(&object_id.to_string())
                .ok_or(Error::SingleQueryMissingValue)
        }

        (SchemaType::Timeseries, Body::Range { from, to, step }) => {
            let timeseries =
                query_service_ts::query_by_range(object_id.to_string(), from, to, step, address)
                    .await
                    .map_err(Error::QueryError)?;

            Ok(timeseries.as_bytes().to_vec())
        }

        (SchemaType::Timeseries, Body::Empty {}) => Err(Error::SingleQueryMissingValue),
    }?;

    Ok(warp::reply::with_header(
        values,
        "Content-Type",
        "application/json",
    ))
}

pub async fn query_multiple(
    object_ids: String,
    schema_id: Uuid,
    address_cache: Arc<AddressCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    trace!(
        "Received /multiple/{:?} (SCHEMA_ID={})",
        object_ids,
        schema_id
    );

    let address = address_cache.get_address(schema_id).await?;
    let object_ids = object_ids.split(',').map(str::to_owned).collect();
    let values = query_service::query_multiple(object_ids, address)
        .await
        .map_err(Error::QueryError)?;

    Ok(warp::reply::json(&byte_map_to_json_map(values)?))
}

pub async fn query_by_schema(
    schema_id: Uuid,
    address_cache: Arc<AddressCache>,
    schema_type_cache: Arc<SchemaTypeCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("Received /schema (SCHEMA_ID={})", schema_id);

    let address = address_cache.get_address(schema_id).await?;

    let schema_type = schema_type_cache.get_schema_type(schema_id).await?;
    let reply = match schema_type {
        SchemaType::DocumentStorage => {
            let values = query_service::query_by_schema(schema_id.to_string(), address)
                .await
                .map_err(Error::QueryError)?;
            warp::reply::json(&byte_map_to_json_map(values)?)
        }
        SchemaType::Timeseries => {
            let timeseries = query_service_ts::query_by_schema(schema_id.to_string(), address)
                .await
                .map_err(Error::QueryError)?;
            warp::reply::json(&(timeseries))
        }
    };

    Ok(reply)
}

fn byte_map_to_json_map(map: HashMap<String, Vec<u8>>) -> Result<Map<String, Value>, Error> {
    map.into_iter()
        .map(|(object_id, value)| {
            Ok((
                object_id,
                serde_json::from_slice(&value).map_err(Error::JsonError)?,
            ))
        })
        .collect::<Result<Map<String, Value>, Error>>()
}
