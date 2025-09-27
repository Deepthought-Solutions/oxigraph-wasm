use oxigraph::model::{BlankNode, GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    // For WASM environments, we provide a deterministic/fixed random source
    // This is acceptable for RDF processing workflows where true randomness
    // isn't critical and deterministic behavior is preferred
    let buf = unsafe { std::slice::from_raw_parts_mut(dest, len) };

    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = ((i * 137 + 42) % 256) as u8; // Simple deterministic pattern
    }
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn custom_ox_now() -> u64 {
    // Return a simple timestamp as u64 for FFI safety
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_create_store() -> *mut Store {
    match Store::new() {
        Ok(store) => Box::into_raw(Box::new(store)),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_destroy_store(store: *mut Store) {
    if !store.is_null() {
        unsafe {
            let _ = Box::from_raw(store);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_add_triple(
    store: *mut Store,
    subject: *const c_char,
    predicate: *const c_char,
    object: *const c_char,
) -> i32 {
    if store.is_null() || subject.is_null() || predicate.is_null() || object.is_null() {
        return -1;
    }

    let store = unsafe { &mut *store };

    let subject_str = match unsafe { CStr::from_ptr(subject) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let predicate_str = match unsafe { CStr::from_ptr(predicate) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let object_str = match unsafe { CStr::from_ptr(object) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let subject = if subject_str.starts_with("http") {
        match NamedNode::new(subject_str) {
            Ok(node) => NamedOrBlankNode::NamedNode(node),
            Err(_) => return -3,
        }
    } else if subject_str.starts_with("_:") {
        NamedOrBlankNode::BlankNode(BlankNode::new(&subject_str[2..]).unwrap_or_default())
    } else {
        return -3;
    };

    let predicate = match NamedNode::new(predicate_str) {
        Ok(node) => node,
        Err(_) => return -3,
    };

    let object = if object_str.starts_with("http") {
        match NamedNode::new(object_str) {
            Ok(node) => Term::NamedNode(node),
            Err(_) => return -3,
        }
    } else if object_str.starts_with("_:") {
        Term::BlankNode(BlankNode::new(&object_str[2..]).unwrap_or_default())
    } else {
        Term::Literal(Literal::new_simple_literal(object_str))
    };

    let quad = Quad::new(subject, predicate, object, GraphName::DefaultGraph);

    match store.insert(&quad) {
        Ok(_) => 0,
        Err(_) => -4,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_query_sparql(
    store: *mut Store,
    query: *const c_char,
    result: *mut c_char,
    result_len: usize,
) -> i32 {
    if store.is_null() || query.is_null() || result.is_null() || result_len == 0 {
        return -1;
    }

    let store = unsafe { &*store };

    let query_str = match unsafe { CStr::from_ptr(query) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let evaluator = SparqlEvaluator::new();
    let results = match evaluator.parse_query(query_str) {
        Ok(prepared_query) => match prepared_query.on_store(&store).execute() {
            Ok(r) => r,
            Err(_) => return -4,
        },
        Err(_) => return -3,
    };

    let output = match results {
        QueryResults::Solutions(solutions) => {
            let mut output_lines = Vec::new();
            for solution in solutions {
                match solution {
                    Ok(solution) => {
                        let mut line = String::new();
                        for (var, term) in solution.iter() {
                            if !line.is_empty() {
                                line.push_str(", ");
                            }
                            line.push_str(&format!("{}={}", var.as_str(), term));
                        }
                        output_lines.push(line);
                    }
                    Err(_) => continue,
                }
            }
            output_lines.join("\n")
        }
        QueryResults::Boolean(b) => b.to_string(),
        QueryResults::Graph(_) => "graph result".to_string(),
    };

    let output_cstring = match CString::new(output) {
        Ok(s) => s,
        Err(_) => return -5,
    };

    let output_bytes = output_cstring.as_bytes_with_nul();

    if output_bytes.len() > result_len {
        return -6; // Buffer too small
    }

    unsafe {
        ptr::copy_nonoverlapping(output_bytes.as_ptr(), result as *mut u8, output_bytes.len());
    }

    output_bytes.len() as i32 - 1 // Return length without null terminator
}

// Additional C bindings for enhanced functionality

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_load_turtle(
    store: *mut Store,
    turtle_data: *const c_char,
    base_iri: *const c_char,
) -> i32 {
    if store.is_null() || turtle_data.is_null() {
        return -1;
    }

    let store = unsafe { &mut *store };

    let turtle_str = match unsafe { CStr::from_ptr(turtle_data) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let _base = if base_iri.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(base_iri) }.to_str() {
            Ok(s) => Some(s),
            Err(_) => return -2,
        }
    };

    let reader = std::io::Cursor::new(turtle_str.as_bytes());
    match store.load_from_reader(oxigraph::io::RdfFormat::Turtle, reader) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_serialize_turtle(
    store: *mut Store,
    result: *mut c_char,
    result_len: usize,
) -> i32 {
    if store.is_null() || result.is_null() || result_len == 0 {
        return -1;
    }

    let store = unsafe { &*store };

    let mut buffer = Vec::new();
    match store.dump_to_writer(oxigraph::io::RdfFormat::Turtle, &mut buffer) {
        Ok(_) => {}
        Err(_) => return -2,
    }

    if buffer.len() >= result_len {
        return -3; // Buffer too small
    }

    let output_cstring = match CString::new(buffer) {
        Ok(s) => s,
        Err(_) => return -4,
    };

    let output_bytes = output_cstring.as_bytes_with_nul();

    if output_bytes.len() > result_len {
        return -3; // Buffer too small
    }

    unsafe {
        ptr::copy_nonoverlapping(output_bytes.as_ptr(), result as *mut u8, output_bytes.len());
    }

    output_bytes.len() as i32 - 1
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_count_triples(store: *mut Store) -> i64 {
    if store.is_null() {
        return -1;
    }

    let store = unsafe { &*store };

    match store.len() {
        Ok(count) => count as i64,
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_clear_store(store: *mut Store) -> i32 {
    if store.is_null() {
        return -1;
    }

    let store = unsafe { &mut *store };

    match store.clear() {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn oxigraph_contains_triple(
    store: *mut Store,
    subject: *const c_char,
    predicate: *const c_char,
    object: *const c_char,
) -> i32 {
    if store.is_null() || subject.is_null() || predicate.is_null() || object.is_null() {
        return -1;
    }

    let store = unsafe { &*store };

    let subject_str = match unsafe { CStr::from_ptr(subject) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let predicate_str = match unsafe { CStr::from_ptr(predicate) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let object_str = match unsafe { CStr::from_ptr(object) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let subject = if subject_str.starts_with("http") {
        match NamedNode::new(subject_str) {
            Ok(node) => NamedOrBlankNode::NamedNode(node),
            Err(_) => return -3,
        }
    } else if subject_str.starts_with("_:") {
        NamedOrBlankNode::BlankNode(BlankNode::new(&subject_str[2..]).unwrap_or_default())
    } else {
        return -3;
    };

    let predicate = match NamedNode::new(predicate_str) {
        Ok(node) => node,
        Err(_) => return -3,
    };

    let object = if object_str.starts_with("http") {
        match NamedNode::new(object_str) {
            Ok(node) => Term::NamedNode(node),
            Err(_) => return -3,
        }
    } else if object_str.starts_with("_:") {
        Term::BlankNode(BlankNode::new(&object_str[2..]).unwrap_or_default())
    } else {
        Term::Literal(Literal::new_simple_literal(object_str))
    };

    let quad = Quad::new(subject, predicate, object, GraphName::DefaultGraph);

    match store.contains(&quad) {
        Ok(exists) => {
            if exists {
                1
            } else {
                0
            }
        }
        Err(_) => -4,
    }
}
