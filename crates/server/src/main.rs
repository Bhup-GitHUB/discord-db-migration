mod store;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use tonic::{transport::Server, Request, Response, Status};

use proto::{
    GetMessagesRequest, GetMessagesResponse, MessageService, MessageServiceServer, StatsRequest,
    StatsResponse,
};

use store::Store;

struct DataService {
    store: Arc<Store>,
}

#[tonic::async_trait]
impl MessageService for DataService {
    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<GetMessagesResponse>, Status> {
        let req = request.into_inner();
        let messages = self
            .store
            .fetch(req.channel_id, req.before, req.limit)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetMessagesResponse { messages }))
    }

    async fn get_stats(
        &self,
        _request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        Ok(Response::new(StatsResponse {
            db_queries: self.store.db_query_count() as i64,
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scylla_addr = env::var("SCYLLA_ADDR").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    let query_delay = Duration::from_millis(
        env::var("QUERY_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    );

    let store = Store::connect(&scylla_addr, query_delay).await?;
    let service = DataService { store };

    println!(
        "data service listening on {} (query_delay={:?}, scylla={})",
        listen_addr, query_delay, scylla_addr
    );

    Server::builder()
        .add_service(MessageServiceServer::new(service))
        .serve(listen_addr.parse()?)
        .await?;

    Ok(())
}
