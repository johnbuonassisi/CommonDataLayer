use command_service::communication::MessageRouter;
use command_service::input::{Error, KafkaInput};
use command_service::output::{
    DruidOutputPlugin, OutputArgs, OutputPlugin, PostgresOutputPlugin, SleighOutputPlugin,
    VictoriaMetricsOutputPlugin,
};
use command_service::report::{FullReportSenderBase, ReportSender, ReportServiceConfig};
use command_service::{args::Args, args::Input, input::GRPCInput, input::InputConfig};
use log::trace;
use rpc::command_service::cmd_service_server::CmdServiceServer;
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tonic::transport::Server;
use utils::metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args: Args = Args::from_args();

    trace!("Environment: {:?}", args);

    metrics::serve();
    match args.output_config {
        OutputArgs::Sleigh(sleigh_config) => {
            start_services(
                args.input,
                args.input_config,
                args.report_config,
                SleighOutputPlugin::new(sleigh_config).await?,
            )
            .await
        }
        OutputArgs::Postgres(postgres_config) => {
            start_services(
                args.input,
                args.input_config,
                args.report_config,
                PostgresOutputPlugin::new(postgres_config).await?,
            )
            .await
        }
        OutputArgs::Druid(druid_config) => {
            start_services(
                args.input,
                args.input_config,
                args.report_config,
                DruidOutputPlugin::new(druid_config).await?,
            )
            .await
        }
        OutputArgs::VictoriaMetrics(victoria_metrics_config) => {
            start_services(
                args.input,
                args.input_config,
                args.report_config,
                VictoriaMetricsOutputPlugin::new(victoria_metrics_config)?,
            )
            .await
        }
    }?;

    Ok(())
}

async fn start_services(
    input: Input,
    input_config: InputConfig,
    report_config: ReportServiceConfig,
    output: impl OutputPlugin,
) -> Result<(), Error> {
    let report_service = match (report_config.topic, report_config.broker) {
        (Some(topic), Some(broker)) => ReportSender::Full(
            FullReportSenderBase::new(broker, topic, output.name().to_string())
                .await
                .map_err(Error::FailedToInitializeReporting)?,
        ),
        (None, None) => ReportSender::Disabled,
        _ => panic!("Must provide both topic and brokers for reporting service to enable it"),
    };

    let message_router = MessageRouter::new(report_service, output);

    match input {
        Input::Kafka => {
            KafkaInput::new(input_config, message_router)
                .await?
                .listen()
                .await?
        }
        Input::GRPC => {
            let input = GRPCInput::new(message_router);
            let addr = SocketAddrV4::new(
                Ipv4Addr::new(0, 0, 0, 0),
                input_config
                    .grpc_port
                    .ok_or(Error::MissingConfig("grpc_port"))?,
            );
            Server::builder()
                .add_service(CmdServiceServer::new(input))
                .serve(addr.into())
                .await?;
        }
    }

    Ok(())
}
