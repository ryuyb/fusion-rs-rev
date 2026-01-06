use macros::app_cached;

#[app_cached(ttl = 60)]
async fn missing_name() -> Result<String, ()> {
    Ok("test".to_string())
}

fn main() {}
