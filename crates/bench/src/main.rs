use std::time::Instant;

use clap::Parser;
use hdrhistogram::Histogram;

use proto::{GetMessagesRequest, MessageServiceClient, StatsRequest};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    target: String,
    #[arg(long, default_value_t = 1234)]
    channel_id: i64,
    #[arg(long, default_value_t = 500)]
    concurrency: usize,
    #[arg(long, default_value_t = 50000)]
    requests: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = MessageServiceClient::connect(args.target.clone()).await?;

    let per_task = (args.requests / args.concurrency).max(1);
    let total = per_task * args.concurrency;

    let baseline = client
        .clone()
        .get_stats(StatsRequest {})
        .await?
        .into_inner()
        .db_queries;

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..args.concurrency {
        let mut client = client.clone();
        let channel_id = args.channel_id;
        handles.push(tokio::spawn(async move {
            let mut hist = Histogram::<u64>::new(3).unwrap();
            for _ in 0..per_task {
                let t = Instant::now();
                let _ = client
                    .get_messages(GetMessagesRequest {
                        channel_id,
                        before: i64::MAX,
                        limit: 50,
                    })
                    .await;
                hist.record(t.elapsed().as_micros() as u64).unwrap();
            }
            hist
        }));
    }

    let mut merged = Histogram::<u64>::new(3).unwrap();
    for h in handles {
        merged.add(h.await?).unwrap();
    }
    let elapsed = start.elapsed();

    let final_count = client
        .clone()
        .get_stats(StatsRequest {})
        .await?
        .into_inner()
        .db_queries;
    let db_queries = final_count - baseline;

    println!("requests sent:   {}", total);
    println!("concurrency:     {}", args.concurrency);
    println!("wall time:       {:.2?}", elapsed);
    println!("throughput:      {:.0} req/s", total as f64 / elapsed.as_secs_f64());
    println!("latency p50:     {} us", merged.value_at_quantile(0.50));
    println!("latency p99:     {} us", merged.value_at_quantile(0.99));
    println!("latency p999:    {} us", merged.value_at_quantile(0.999));
    println!("db queries:      {}", db_queries);
    println!("reads per query: {:.1}", total as f64 / db_queries.max(1) as f64);

    Ok(())
}
