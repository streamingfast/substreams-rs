#!/bin/sh

buf generate ../substreams/proto \
  --exclude-path ../substreams/proto/sf/substreams/intern/v2 \
  --exclude-path ../substreams/proto/sf/substreams/rpc/v2 \
  --exclude-path ../substreams/proto/sf/substreams/sink \
  --exclude-path ../substreams/proto/sf/substreams/options.proto