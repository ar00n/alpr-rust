use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    servers(
        (url = "http://127.0.0.1:3000")
    ),
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}
