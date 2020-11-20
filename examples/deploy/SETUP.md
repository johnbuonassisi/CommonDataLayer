# Setting UP local environment

## Preamble

Contents of this folder aren't meant for use on production and they may be lagging behind our k8s deployment.
Sole purpose of this directory is to prepare exemplary development environment, from which anyone can startup their development on
`common data layer` without Kubernetes knowledge. Contents of docker-compose may not contain all applications, so be aware of that. You may alter it
on your local machine to your needs.

For k8s deployment, please refer to our [documentation](../../docs/K8s-Local-Deployment.md).

## Requirements
* docker
* docker-compose
* rust (optionally)

## Deployment
You must first add environment variables:

`DOCKER_BUILDKIT=1`
`COMPOSE_DOCKER_CLI_BUILD=1`

Due to services needing additional startup time, we advise to let docker setup infrastructure first, and deploy CDL after. So...

`docker-compose up -d kafka1 zoo1 postgres1 rabbit1`

After it had some time to setup, you can proceed with rest of the environment:

`docker-compose up -d`

## Entry points in system
### RabbitMQ

RabbitMQ should be accessible from `localhost:5672` for ingesting data.

It is setup with default account,
> login:password = guest:guest

Default exchange is `cdl`, default queue and routing key are `cdl.data.input`, default vhost is `/`.

You can access RabbitMQ web admin on `localhost:15672`.

### Kafka

You can write to kafka on `localhost:9092`.
By default there is no replication on *schema_registry* and postgres *command_service* input channel is `cdl.document.input`.

Errors are written to `cdl.reports`.

### PostgreSQL

To access postgres you must have some postgresql client installed.

For command line it's best to refer to your OS package manager (`homebrew` on OSX, `apt-get` on Ubuntu, `choco` on Windows).

`psql -U postgres --password -h localhost`
the password is `1234`

### VictoriaMetrics

You can query VM over it's rest endpoints at [localhost:8428](http://localhost:8428)

### Schema registry
Schema registry can be either accessed via [gRPC](../../schema-registry/proto/registry.proto), or via `cdl-cli`. Using `cdl-cli` will require presence of rust compiler on your local machine.
Tips on how to install rust are available on [rustup website](https://rustup.rs/).

From main directory of this project you can run `cdl-cli` via:

`cargo run -p cdl-cli -- --help`

Registry address is `http://localhost:50101`.

eg.

* Create basic schema and add it to the system:
* Adding new schema:
> `echo "{}" > schema.json`
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema add --name default-document --file schema.json --topic "cdl.document.input" --query-address "http://postgres_query1:50102"`

* Setting schema topic can be also done as a separate step (in order for this schema to be routed to `command-service` topic must be `cdl.document.input`)
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema set-topic --id 0a626bba-15ff-11eb-8004-000000000000 --topic "cdl.document.input"`

* Getting all schemas
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema names`

### Query service

We can query postgres for data on [localhost:50102](http://localhost:50102) via gRPC ([proto file](../../query-service/proto/query.proto))

### Query router

Available at [localhost:50103](http://localhost:50103) via REST API. No JSON API documentation yet, please refer to source code.

Query router can be called with following commands:
curl -H "SCHEMA_ID: {schema_id}" http://localhost:50103/single/object-uuid
curl -H "SCHEMA_ID: {schema_id}" http://localhost:50103/multiple/comma,separated,object_ids
curl -H "SCHEMA_ID: {schema_id}" http://localhost:50103/schema

For example:
`curl -H "SCHEMA_ID: 843c8518-2b1b-11eb-800c-000000000000" http://localhost:50103/single/b0b2b146-0f3d-4887-94d3-7b5088a6f12e`


To populate system it is possible to use:
`cargo run -p benchmarking --bin upload_to_rabbitmq -- --address "amqp://localhost:5672" --count 8 --exchange cdl --queue cdl.data.input --schema-id 843c8518-2b1b-11eb-800c-000000000000`
