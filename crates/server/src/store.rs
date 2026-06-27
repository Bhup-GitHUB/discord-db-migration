use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::statement::prepared::PreparedStatement;

use proto::Message;

pub struct Store {
    session: Session,
    prepared: PreparedStatement,
    db_queries: AtomicU64,
    query_delay: Duration,
}

impl Store {
    pub async fn connect(addr: &str, query_delay: Duration) -> anyhow::Result<Arc<Self>> {
        let session = SessionBuilder::new().known_node(addr).build().await?;
        session.use_keyspace("discord", false).await?;
        let prepared = session
            .prepare(
                "SELECT message_id, author_id, content FROM messages \
                 WHERE channel_id = ? AND bucket = ? AND message_id < ? LIMIT ?",
            )
            .await?;
        Ok(Arc::new(Self {
            session,
            prepared,
            db_queries: AtomicU64::new(0),
            query_delay,
        }))
    }

    pub async fn fetch(&self, channel_id: i64, before: i64, limit: i32) -> anyhow::Result<Vec<Message>> {
        self.db_queries.fetch_add(1, Ordering::Relaxed);
        if !self.query_delay.is_zero() {
            tokio::time::sleep(self.query_delay).await;
        }
        let bucket = 0i64;
        let result = self
            .session
            .execute_unpaged(&self.prepared, (channel_id, bucket, before, limit))
            .await?;
        let rows = result.into_rows_result()?;
        let mut out = Vec::new();
        for row in rows.rows::<(i64, i64, String)>()? {
            let (message_id, author_id, content) = row?;
            out.push(Message {
                message_id,
                author_id,
                content,
            });
        }
        Ok(out)
    }

    pub fn db_query_count(&self) -> u64 {
        self.db_queries.load(Ordering::Relaxed)
    }
}
