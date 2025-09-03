pub mod sf {
    pub mod substreams {
        pub mod index {
            // @@protoc_insertion_point(attribute:sf.substreams.index.v1)
            pub mod v1 {
                include!("sf.substreams.index.v1.rs");
                // @@protoc_insertion_point(sf.substreams.index.v1)
            }
        }

        pub mod foundational_store {
            // @@protoc_insertion_point(attribute:sf.substreams.foundational_store.v1)
            pub mod v1 {
                include!("sf.substreams.foundational_store.v1.rs");
                // @@protoc_insertion_point(sf.substreams.foundational_store.v1)
            }
        }
    }
}

// Legacy way to import Protobuf definition to keep backward compatibility with all
// Substreams modules out there.
//
// New Protobuf definitions should be added using the namespacing seen above.
#[path = "./sf.substreams.v1.rs"]
pub mod substreams;
