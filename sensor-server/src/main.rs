fn main() {
    // Call the async main from lib.rs
    // Use tokio runtime since your main is async
    let result = sensor_server::main();
    // Block on the async main
    if let Err(e) = result {
        eprintln!("Error running the server: {}", e);
    }
}
