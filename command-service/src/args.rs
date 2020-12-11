use structopt::StructOpt;

use crate::input::InputConfig;
use crate::output::OutputArgs;
use crate::report::ReportServiceConfig;
use structopt::clap::arg_enum;

#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub output_config: OutputArgs,

    #[structopt(long, env = "INPUT", possible_values = &Input::variants(), case_insensitive = true)]
    pub input: Input,

    #[structopt(flatten)]
    pub input_config: InputConfig,

    #[structopt(flatten)]
    pub report_config: ReportServiceConfig,
}

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum Input {
        Kafka,
        GRPC,
    }
}
