mod coalesce;
mod store;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use tonic::{transport::Server, Request, Response, Status};

use proto::{
    GetMessagesRequest, GetMessagesResponse, Message, MessageService, MessageServiceServer,
    StatsRequest, StatsResponse,
};

use coalesce::{CoalesceError, Coalescer};
use store::Store;

struct DataService {
    store: Arc<Store>,
    coalescer: Arc<Coalescer<Vec<Message>>>,
    coalesce_enabled: bool,
}

#[tonic::async_trait]
impl MessageService for DataService {
    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<GetMessagesResponse>, Status> {
        let req = request.into_inner();
        let channel_id = req.channel_id;
        let before = req.before;
        let limit = req.limit;

        let messages = if self.coalesce_enabled {
            let store = self.store.clone();
            self.coalescer
                .run((channel_id, before, limit), move || async move {
                    store
                        .fetch(channel_id, before, limit)
                        .await
                        .map_err(|e| CoalesceError(e.to_string()))
                })
                .await
                .map_err(|e| Status::internal(e.0))?
        } else {
            self.store
                .fetch(channel_id, before, limit)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
        };

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
    let coalesce_enabled = env::var("COALESCE")
        .map(|v| v != "off" && v != "0" && v != "false")
        .unwrap_or(true);
    let query_delay = Duration::from_millis(
        env::var("QUERY_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    );

    let store = Store::connect(&scylla_addr, query_delay).await?;
    let service = DataService {
        store,
        coalescer: Arc::new(Coalescer::new()),
        coalesce_enabled,
    };

    println!(
        "data service listening on {} (coalesce={}, query_delay={:?}, scylla={})",
        listen_addr, coalesce_enabled, query_delay, scylla_addr
    );

    Server::builder()
        .add_service(MessageServiceServer::new(service))
        .serve(listen_addr.parse()?)
        .await?;

    Ok(())
}
