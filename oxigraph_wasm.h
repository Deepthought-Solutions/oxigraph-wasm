#ifndef OXIGRAPH_WASM_H
#define OXIGRAPH_WASM_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// Opaque store type
typedef struct Store Store;

// Core store operations
/**
 * Creates a new Oxigraph store.
 * @return Pointer to the store, or NULL on failure.
 */
Store* oxigraph_create_store(void);

/**
 * Destroys an Oxigraph store and frees memory.
 * @param store Pointer to the store to destroy.
 */
void oxigraph_destroy_store(Store* store);

/**
 * Clears all triples from the store.
 * @param store Pointer to the store.
 * @return 0 on success, negative on error.
 */
int32_t oxigraph_clear_store(Store* store);

/**
 * Gets the number of triples in the store.
 * @param store Pointer to the store.
 * @return Number of triples, or negative on error.
 */
int64_t oxigraph_count_triples(Store* store);

// Triple operations
/**
 * Adds a triple to the store.
 * @param store Pointer to the store.
 * @param subject Subject URI (http://...) or blank node (_:id).
 * @param predicate Predicate URI (http://...).
 * @param object Object URI (http://...), blank node (_:id), or literal string.
 * @return 0 on success, negative on error:
 *   -1: NULL pointer
 *   -2: Invalid UTF-8
 *   -3: Invalid RDF term format
 *   -4: Store insertion error
 */
int32_t oxigraph_add_triple(Store* store, const char* subject, const char* predicate, const char* object);

/**
 * Checks if a triple exists in the store.
 * @param store Pointer to the store.
 * @param subject Subject URI or blank node.
 * @param predicate Predicate URI.
 * @param object Object URI, blank node, or literal.
 * @return 1 if exists, 0 if not, negative on error.
 */
int32_t oxigraph_contains_triple(Store* store, const char* subject, const char* predicate, const char* object);

// SPARQL operations
/**
 * Executes a SPARQL query and returns results as a string.
 * @param store Pointer to the store.
 * @param query SPARQL query string.
 * @param result Buffer to store the result string.
 * @param result_len Size of the result buffer.
 * @return Length of result string on success, negative on error:
 *   -1: NULL pointer or zero buffer size
 *   -2: Invalid UTF-8 in query
 *   -3: SPARQL parse error
 *   -4: Query execution error
 *   -5: Result serialization error
 *   -6: Buffer too small
 */
int32_t oxigraph_query_sparql(Store* store, const char* query, char* result, size_t result_len);

// RDF serialization operations
/**
 * Loads Turtle data into the store.
 * @param store Pointer to the store.
 * @param turtle_data Turtle format RDF data.
 * @param base_iri Base IRI for relative URIs (can be NULL).
 * @return 0 on success, negative on error:
 *   -1: NULL pointer
 *   -2: Invalid UTF-8
 *   -3: Turtle parse error
 */
int32_t oxigraph_load_turtle(Store* store, const char* turtle_data, const char* base_iri);

/**
 * Serializes the store to Turtle format.
 * @param store Pointer to the store.
 * @param result Buffer to store the Turtle string.
 * @param result_len Size of the result buffer.
 * @return Length of result string on success, negative on error:
 *   -1: NULL pointer or zero buffer size
 *   -2: Serialization error
 *   -3: Buffer too small
 *   -4: String conversion error
 */
int32_t oxigraph_serialize_turtle(Store* store, char* result, size_t result_len);

// Time function for WASM environments
/**
 * Custom time function for WASM environments.
 * @return Duration representing current time (deterministic in WASM).
 */
uint64_t custom_ox_now(void);

#ifdef __cplusplus
}
#endif

#endif // OXIGRAPH_WASM_H