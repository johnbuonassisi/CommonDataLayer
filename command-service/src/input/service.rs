mod kafka;

mod grpc {
    use log::trace;
    use rpc::command_service::{cmd_service_server::CmdService, Empty, InsertMessage};

    use tonic::{Request, Response, Status};
    use utils::message_types::CommandServiceInsertMessage;

    use crate::{
        communication::GenericMessage, communication::MessageRouter, input::Error,
        output::OutputPlugin,
    };

    pub struct GRPCInput<P: OutputPlugin> {
        message_router: MessageRouter<P>,
    }

    impl<P: OutputPlugin> GRPCInput<P> {
        pub fn new(message_router: MessageRouter<P>) -> Self {
            Self { message_router }
        }

        async fn handle_message(
            router: MessageRouter<P>,
            message: InsertMessage,
        ) -> Result<(), Error> {
            let generic_message = Self::build_message(&message)?;
            trace!("Received message {:?}", generic_message);

            router
                .handle_message(generic_message)
                .await
                .map_err(Error::CommunicationError)?;

            Ok(())
        }

        fn build_message(message: &'_ InsertMessage) -> Result<GenericMessage, Error> {
            let json = &message.payload;
            let event: CommandServiceInsertMessage =
                serde_json::from_slice(&json).map_err(Error::PayloadDeserializationFailed)?;

            Ok(GenericMessage {
                object_id: event.object_id,
                schema_id: event.schema_id,
                timestamp: event.timestamp,
                payload: event.payload.to_string().as_bytes().to_vec(),
            })
        }
    }

    #[tonic::async_trait]
    impl<P: OutputPlugin> CmdService for GRPCInput<P> {
        async fn insert(&self, request: Request<InsertMessage>) -> Result<Response<Empty>, Status> {
            let message = request.into_inner();
            let router = self.message_router.clone();

            match Self::handle_message(router, message).await {
                Ok(_) => Ok(Response::new(Empty {})),
                Err(err) => Err(Status::internal(err.to_string())),
            }
        }
    }
}

pub use grpc::GRPCInput;
pub use kafka::KafkaInput;
