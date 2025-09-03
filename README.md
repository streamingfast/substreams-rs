# Substreams

Substreams is a powerful blockchain indexing technology, developed for The Graph Network.

It enables you to write Rust modules, composing data streams alongside the community, and provides extremely high performance indexing by virtue of parallelization, in a streaming-first fashion.

It has all the benefits of the Firehose, like low-cost caching and archiving of blockchain data, high throughput processing, and cursor-based reorgs handling.

## Documentation

Full documentation is accessible at https://docs.substreams.dev.

### Getting Started

* [Your First Stream](https:/docs.substreams.dev/getting-started/your-first-stream)

### Concept & Fundamentals

* [Definition](https:/docs.substreams.dev/concepts/definition)
* [Comparison](https:/docs.substreams.dev/concepts/comparison)
* [Modules](https:/docs.substreams.dev/concepts/modules)
    * [Inputs](https:/docs.substreams.dev/concept-and-fundamentals/modules/inputs)
    * [Outputs](https:/docs.substreams.dev/concept-and-fundamentals/modules/outputs)

### Developer Guide

* [Overview](https:/docs.substreams.dev/developer-guide/overview)
* [Installation](https:/docs.substreams.dev/developer-guide/installation-requirements)
* [Creating your Manifest](https:/docs.substreams.dev/developer-guide/creating-your-manifest)
* [Creating Protobuf Schemas](https:/docs.substreams.dev/developer-guide/creating-protobuf-schemas)
* [Setting Up Handlers](https:/docs.substreams.dev/developer-guide/setting-up-handlers)
* [Writing Module Handlers](https:/docs.substreams.dev/developer-guide/writing-module-handlers)
* [Running Your Substreams](https:/docs.substreams.dev/developer-guide/running-substreams)

#### Development

#### Protobuf Generation

It seems that the way we used to generate the Protobuf bindings hasn't been properly documented. It seems that for now, a manual generation of Protobuf needs to be done:

```
# Working directory at root of project

cd substreams
protoc --prost_out=src/pb --proto_path=../../substreams-foundational-store/proto ../../substreams-foundational-store/proto/sf/substreams/foundational-store/v1/*.proto
```