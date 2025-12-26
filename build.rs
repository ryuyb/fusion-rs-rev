use shadow_rs::ShadowBuilder;

fn main() {
    // Generate build metadata for version information using shadow-rs 1.5.0
    ShadowBuilder::builder()
        .build()
        .expect("Failed to generate build metadata");
}