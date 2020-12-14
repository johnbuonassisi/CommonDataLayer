# Command Service

## Technical Description

Cdl-cli is currently the only tool able to interact with schema registry. Its purpose is to interact with the schema registry database by setting and viewing its content.

Please mind that this tool is a reference. Schema-Registry port should be configured open, and every tool that is able to communicate to it should be able to set and get information about schemas from Schema-Registry. Currently only schema-regisry only supports GRPC protocol, but there are some progress with both REST and webgui parts, which may be presented in the future as GRPC's alternative.

Communication Methods:
- GRPC

## How to guide:

#### Manipulate views
views are a WIP feature, currently not used widely beside some cases in development.

#### manipulate schemas

###### add schema
`cdl --registry-address "http://localhost:6400 schema <add|get|names|update> --name <schemaname> \
    --query-address <query-service-uri>" \
    --topic <ingest-topic> \
    --file <optional:schema-path>
`

- if `--file` is provided, specific file must have valid json inside.
- if `--file` is missing, you will be asked to provide json inside cli,
  to do so, insety valid json, followed by newline, and then press `ctrl+d` to finish editing
- minimal valid json schema is `{}`
- random schema id will be assigned to the request

###### List schemas
- will print all existing schema names and IDs:
`cdl --registry-address "http:/localhost:6400 schema names`
