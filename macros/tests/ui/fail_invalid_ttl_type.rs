use macros::app_cached;

#[app_cached(name = "test", ttl = "invalid")]
async fn invalid_ttl_type() -> Result<String, ()> {
    Ok("test".to_string())
}

fn main() {}
