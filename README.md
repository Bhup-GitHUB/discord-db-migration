# discord data services layer

A small prototype of the request coalescing trick behind Discord's Cassandra to ScyllaDB
migration. A Rust gRPC service sits in front of ScyllaDB; when many clients read the same
hot channel at the same moment, the in-flight reads collapse into a single database query
and that one result is handed back to every caller.

## layout

- `crates/proto` grpc contract for the message service
- `crates/server` the data service, a scylla store plus the coalescer
- `crates/bench` a concurrent load generator that hammers one hot channel

## prerequisites

rust, docker, protoc.

## run

1. start scylladb and wait for it to boot (about 40s)

   ```
   docker-compose up -d
   ```

2. seed the schema and a hot channel

   ```
   docker exec -i scylla-demo cqlsh < cql/schema.cql
   ```

3. build

   ```
   cargo build
   ```

### naive pass, coalescing off

```
COALESCE=off QUERY_DELAY_MS=5 cargo run -p server
cargo run -p bench -- --channel-id 1234 --concurrency 500 --requests 50000
```

### coalesced pass, coalescing on

```
COALESCE=on QUERY_DELAY_MS=5 cargo run -p server
cargo run -p bench -- --channel-id 1234 --concurrency 500 --requests 50000
```

## what you see

50000 concurrent reads of one hot channel:

| mode | db queries | reads per query | p99 |
| --- | --- | --- | --- |
| naive | 50000 | 1.0 | 69 ms |
| coalesced | 336 | 148.8 | 66 ms |

Same latency, about 149x less load on the database. That is the point: coalescing keeps a
hot partition from drowning in duplicate reads. Latency is flat in this run because the
single grpc connection is the bottleneck, not the database. Under a database that actually
buckles under the herd, that 149x drop in queries is what holds p99 down.

## knobs

- `COALESCE` on or off
- `QUERY_DELAY_MS` simulated per-query database latency
- `SCYLLA_ADDR`, `LISTEN_ADDR`
- bench flags `--channel-id`, `--concurrency`, `--requests`, `--target`
