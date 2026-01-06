use macros::app_cached;

#[app_cached(name = "test", unknown = "value")]
async fn unknown_attribute() -> Result<String, ()> {
    Ok("test".to_string())
}

fn main() {}
