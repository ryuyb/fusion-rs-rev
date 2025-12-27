use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub const USER_TAG: &str = "User";
pub const AUTH_TAG: &str = "Auth";
pub const HEALTH_TAG: &str = "Health";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Fusion",
        description = "An api server for Fusion",
    ),
    paths(
        crate::api::handlers::health::health_check,
        crate::api::handlers::health::readiness_check,
        crate::api::handlers::health::liveness_check,
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = USER_TAG, description = "User management endpoints"),
        (name = AUTH_TAG, description = "User authentication endpoints"),
        (name = HEALTH_TAG, description = "Health check and monitoring endpoints")
    ),
    components(
        schemas(
            crate::api::dto::ErrorResponse,
        )
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
