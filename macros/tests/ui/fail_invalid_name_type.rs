use macros::app_cached;

#[app_cached(name = 123)]
async fn invalid_name_type() -> Result<String, ()> {
    Ok("test".to_string())
}

fn main() {}
