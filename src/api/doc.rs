use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub const USER_TAG: &str = "User";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Fusion",
        description = "An api server for Fusion",
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = USER_TAG, description = "User management endpoints")
    ),
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT Bearer Token Authentication"))
                        .build(),
                ),
            )
        }
    }
}
