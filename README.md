# Oxigraph WASM with C Bindings

A WebAssembly library that provides raw C bindings for the Oxigraph RDF store, designed for use in sandboxed WASM environments that don't support JavaScript.

## Features

- **Store Management**: Create, destroy, and clear RDF stores
- **Triple Operations**: Add, check existence, and count triples
- **SPARQL Queries**: Execute SPARQL queries with string results
- **RDF Serialization**: Load and serialize Turtle format
- **WASM Optimized**: Built for `wasm32-unknown-unknown` target
- **Custom Random**: Deterministic random number generation for WASM
- **FFI Safe**: All functions use C-compatible types

## API Functions

### Store Management
- `oxigraph_create_store()` - Create a new store
- `oxigraph_destroy_store(store)` - Destroy a store
- `oxigraph_clear_store(store)` - Clear all triples
- `oxigraph_count_triples(store)` - Get triple count

### Triple Operations
- `oxigraph_add_triple(store, subject, predicate, object)` - Add a triple
- `oxigraph_contains_triple(store, subject, predicate, object)` - Check if triple exists

### SPARQL
- `oxigraph_query_sparql(store, query, result, result_len)` - Execute SPARQL query

### RDF Serialization
- `oxigraph_load_turtle(store, turtle_data, base_iri)` - Load Turtle data
- `oxigraph_serialize_turtle(store, result, result_len)` - Export as Turtle

## Building

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build release version
cargo build --release --target wasm32-unknown-unknown

# Or use the build script
./build.sh
```

## Generated Files

- `target/wasm32-unknown-unknown/release/oxigraph_wasm.wasm` - WASM module
- `target/wasm32-unknown-unknown/release/liboxigraph_wasm.a` - Static library
- `oxigraph_wasm.h` - C header file

## Usage

Include the header file in your C/C++ code:

```c
#include "oxigraph_wasm.h"

// Create store
Store* store = oxigraph_create_store();

// Add a triple
int result = oxigraph_add_triple(store,
    "http://example.org/subject",
    "http://example.org/predicate",
    "example value");

// Query
char buffer[1024];
int len = oxigraph_query_sparql(store,
    "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
    buffer, sizeof(buffer));

// Cleanup
oxigraph_destroy_store(store);
```

## Dependencies

- Oxigraph (Deepthought-Solutions fork with WASM support)
- getrandom 0.3 (with custom backend for WASM)

## Error Codes

All functions return negative values on error:
- `-1`: NULL pointer or invalid arguments
- `-2`: Invalid UTF-8 encoding
- `-3`: RDF/SPARQL parse error
- `-4`: Store operation error
- `-5`: Serialization error
- `-6`: Buffer too small

## WASM Environment

This crate is specifically designed for sandboxed WASM environments and includes:
- Custom deterministic random number generation
- Fixed time functions
- No JavaScript dependencies
- Optimized build configuration