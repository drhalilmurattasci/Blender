//! Forge3D -- open-source 3-D creation suite.
//!
//! Binary entry point: initialise logging, build the application, create the
//! default startup scene, and hand control to the main loop.

fn main() {
    // Initialise a basic tracing subscriber.
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let mut app = forge3d::Application::new();
    app.create_default_scene();
    app.run();
}
