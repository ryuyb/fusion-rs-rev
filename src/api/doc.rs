use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub const USER_TAG: &str = "User";
pub const AUTH_TAG: &str = "Auth";
pub const HEALTH_TAG: &str = "Health";
pub const NOTIFICATION_TAG: &str = "Notifications";
pub const JOB_TAG: &str = "Jobs";
pub const LIVE_TAG: &str = "Live";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Fusion",
        description = "An api server for Fusion",
    ),
    modifiers(&SecurityAddon),
    components(
        schemas(
            crate::api::dto::ErrorResponse,
            crate::external::live::LivePlatform,
        )
    ),
    tags(
        (name = USER_TAG, description = "User management endpoints"),
        (name = AUTH_TAG, description = "Authentication endpoints"),
        (name = HEALTH_TAG, description = "Health check endpoints"),
        (name = NOTIFICATION_TAG, description = "Notification channel and message endpoints"),
        (name = JOB_TAG, description = "Job scheduling endpoints"),
        (name = LIVE_TAG, description = "Live streaming platform endpoints"),
    )
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
