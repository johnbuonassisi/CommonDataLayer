use crate::schema::query_server::Query;
use crate::schema::{ObjectIds, SchemaId, ValueMap};
use anyhow::Context;
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::config::Config as PgConfig;
use bb8_postgres::tokio_postgres::{types::ToSql, NoTls, Row};
use bb8_postgres::PostgresConnectionManager;
use std::collections::HashMap;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use utils::metrics::counter;

#[derive(Debug, StructOpt)]
pub struct PsqlConfig {
    #[structopt(long = "postgres-query-url", env = "POSTGRES_QUERY_URL")]
    psql_url: String,
}

pub struct PsqlQuery {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PsqlQuery {
    pub async fn load(config: PsqlConfig) -> anyhow::Result<Self> {
        let pg_config: PgConfig = config.psql_url.parse().context("Invalid psql config")?;
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("Failed to build connection pool")?;

        Ok(Self { pool })
    }

    async fn connect(
        &self,
    ) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn make_query(
        &self,
        query: &str,
        args: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Status> {
        let connection = self.connect().await?;
        let statement = connection.prepare(query).await.map_err(|err| {
            Status::internal(format!("Unable to prepare query statement: {}", err))
        })?;

        connection
            .query(&statement, args)
            .await
            .map_err(|err| Status::internal(format!("Unable to query data: {}", err)))
    }

    fn collect_id_payload_rows(rows: Vec<Row>) -> HashMap<String, Vec<u8>> {
        rows
            .into_iter()
            .map(|row| {
                let object_id = row
                    .get::<usize, uuid::Uuid>(0)
                    .to_hyphenated()
                    .encode_upper(&mut uuid::Uuid::encode_buffer())
                    .to_owned();
                let value = serde_json::to_vec(&row.get::<usize, serde_json::Value>(1)).unwrap();
                (object_id, value)
            })
            .collect()
    }
}

#[tonic::async_trait]
impl Query for PsqlQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-multiple.psql", 1);

        let object_uuids = request
            .into_inner()
            .object_ids
            .iter()
            .map(|x| {
                uuid::Uuid::parse_str(x).map_err(|err| {
                    Status::internal(format!("Unable to parse string to Uuid: {}", err))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let rows = self
            .make_query(
                "SELECT object_id, payload FROM data d1 \
                WHERE object_id = ANY($1) AND d1.version = \
                    (SELECT MAX(version) FROM data d2 \
                     WHERE d2.object_id = d1.object_id)",
                &[&object_uuids],
            )
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-by-schema.psql", 1);

        let schema_uuid = uuid::Uuid::parse_str(&request.into_inner().schema_id)
            .map_err(|err| Status::internal(format!("Unable to parse string to Uuid: {}", err)))?;

        let rows = self
            .make_query(
                "SELECT object_id, payload FROM data d1 \
                WHERE schema_id = $1 AND d1.version = \
                    (SELECT MAX(version) FROM data d2 \
                     WHERE d2.object_id = d1.object_id)",
                &[&schema_uuid],
            )
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }
}
