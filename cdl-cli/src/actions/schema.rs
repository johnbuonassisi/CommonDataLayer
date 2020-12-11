use crate::utils::*;
use rpc::schema_registry::{
    Empty, Id, NewSchema, NewSchemaVersion, SchemaNameUpdate, SchemaQueryAddressUpdate,
    SchemaTopicUpdate, SchemaTypeUpdate, ValueToValidate, VersionedId,
};
use schema_registry::types::SchemaType;
use semver::{Version, VersionReq};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn get_schema(
    schema_id: Uuid,
    version: Option<VersionReq>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .get_schema(VersionedId {
            id: schema_id.to_string(),
            version_req: version.unwrap_or_else(VersionReq::any).to_string(),
        })
        .await?;

    println!(
        "{:#}",
        serde_json::from_str::<Value>(&response.into_inner().definition)?
    );

    Ok(())
}

pub async fn add_schema(
    schema_name: String,
    topic: String,
    query_address: String,
    file: Option<PathBuf>,
    registry_addr: String,
    schema_type: SchemaType,
) -> anyhow::Result<()> {
    let definition = read_json(file)?;

    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .add_schema(NewSchema {
            id: "".into(),
            name: schema_name.clone(),
            definition: serde_json::to_string(&definition)?,
            query_address,
            topic_name: topic,
            schema_type: schema_type as i32,
        })
        .await?;

    eprintln!(
        "Successfully added schema \"{}\" to the schema registry.",
        schema_name
    );
    eprintln!("The following UUID was assigned:");
    println!("{}", response.into_inner().id);

    Ok(())
}

pub async fn get_schema_versions(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .get_schema_versions(Id {
            id: schema_id.to_string(),
        })
        .await?;

    let mut versions = response
        .into_inner()
        .versions
        .into_iter()
        .map(|v| Version::parse(&v))
        .collect::<Result<Vec<Version>, _>>()?;
    versions.sort();

    for version in versions {
        println!("{}", version);
    }

    Ok(())
}

pub async fn set_schema_name(
    schema_id: Uuid,
    name: String,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    client
        .update_schema_name(SchemaNameUpdate {
            id: schema_id.to_string(),
            name,
        })
        .await?;

    Ok(())
}

pub async fn set_schema_topic(
    schema_id: Uuid,
    topic: String,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    client
        .update_schema_topic(SchemaTopicUpdate {
            id: schema_id.to_string(),
            topic,
        })
        .await?;

    Ok(())
}

pub async fn set_schema_query_address(
    schema_id: Uuid,
    query_address: String,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    client
        .update_schema_query_address(SchemaQueryAddressUpdate {
            id: schema_id.to_string(),
            query_address,
        })
        .await?;

    Ok(())
}

pub async fn set_schema_type(
    schema_id: Uuid,
    schema_type: SchemaType,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    client
        .update_schema_type(SchemaTypeUpdate {
            id: schema_id.to_string(),
            schema_type: schema_type as i32,
        })
        .await?;

    Ok(())
}

pub async fn add_schema_version(
    schema_id: Uuid,
    version: Version,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let definition = read_json(file)?;
    let schema = NewSchemaVersion {
        id: schema_id.to_string(),
        version: version.to_string(),
        definition: serde_json::to_string(&definition)?,
    };

    let mut client = connect_to_registry(registry_addr).await?;
    client.add_schema_version(schema).await?;

    Ok(())
}

pub async fn get_schema_names(registry_addr: String) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let schemas = client
        .get_all_schema_names(Empty {})
        .await?
        .into_inner()
        .names;

    if schemas.is_empty() {
        anyhow::bail!("No schemas exist yet in the schema registry.");
    }

    for (id, name) in schemas {
        println!("ID: {}, Name: {}", id, name);
    }

    Ok(())
}

pub async fn get_schema_topic(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .get_schema_topic(Id {
            id: schema_id.to_string(),
        })
        .await?;

    println!("{}", response.into_inner().topic);

    Ok(())
}

pub async fn get_schema_query_address(
    schema_id: Uuid,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .get_schema_query_address(Id {
            id: schema_id.to_string(),
        })
        .await?;

    println!("{}", response.into_inner().address);

    Ok(())
}

pub async fn get_schema_type(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = connect_to_registry(registry_addr).await?;
    let response = client
        .get_schema_type(Id {
            id: schema_id.to_string(),
        })
        .await?;

    println!("{}", SchemaType::from(response.into_inner().schema_type()));

    Ok(())
}

pub async fn validate_value(
    schema_id: Uuid,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let value = read_json(file)?;

    let mut client = connect_to_registry(registry_addr).await?;
    let errors = client
        .validate_value(ValueToValidate {
            schema_id: schema_id.to_string(),
            value: serde_json::to_string(&value)?,
        })
        .await?
        .into_inner()
        .errors;

    if !errors.is_empty() {
        anyhow::bail!("The value is not valid. {}", errors.join(", "));
    }

    println!("The value is valid.");
    Ok(())
}
