pub mod discord {
    tonic::include_proto!("discord");
}

pub use discord::message_service_client::MessageServiceClient;
pub use discord::message_service_server::{MessageService, MessageServiceServer};
pub use discord::{GetMessagesRequest, GetMessagesResponse, Message, StatsRequest, StatsResponse};
