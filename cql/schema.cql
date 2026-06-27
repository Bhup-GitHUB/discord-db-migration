CREATE KEYSPACE IF NOT EXISTS discord
  WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE IF NOT EXISTS discord.messages (
  channel_id bigint,
  bucket bigint,
  message_id bigint,
  author_id bigint,
  content text,
  PRIMARY KEY ((channel_id, bucket), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);

INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1010, 7001, 'pushed the migrator branch, running it now');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1009, 7002, 'scylla node is already at 9TB, cassandra capped at 4');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1008, 7001, 'p99 dropped to a flat 5ms, no more GC sawtooth');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1007, 7003, 'reverse range scans were the worst on cassandra');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1006, 7002, 'shard per core means no lock contention');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1005, 7001, 'rewrote the loader in rust over an afternoon');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1004, 7004, 'hit 3.2 million rows per second on the migrate');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1003, 7002, 'three month ETA turned into nine days');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1002, 7003, 'coalescing collapsed the hot partition reads');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (1234, 0, 1001, 7001, 'going from 177 cassandra nodes to 72 scylla nodes');

INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (5678, 0, 2003, 8001, 'random side channel chatter');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (5678, 0, 2002, 8002, 'not a hot partition');
INSERT INTO discord.messages (channel_id, bucket, message_id, author_id, content) VALUES (5678, 0, 2001, 8001, 'just here for the demo');
