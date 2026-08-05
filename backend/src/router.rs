use axum::{middleware, Router};
use tower_http::cors::{Any, CorsLayer};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

use crate::{
    auth,
    handlers,
    openapi,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let public_routes = OpenApiRouter::new()
        .routes(routes!(handlers::websocket_handler))
        .routes(routes!(handlers::login_user));

    let auth_routes = OpenApiRouter::new()
        .routes(routes!(handlers::get_history_handler))
        .routes(routes!(handlers::get_snapshot))
        .routes(routes!(handlers::get_allow_list, handlers::add_allow_list))
        .routes(routes!(handlers::delete_allow_list))
        .routes(routes!(handlers::mjpeg_stream_handler))
        .routes(routes!(handlers::change_password))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let admin_routes = OpenApiRouter::new()
        .routes(routes!(handlers::create_user, handlers::delete_user, handlers::get_users))
        .routes(routes!(handlers::get_framerate, handlers::update_framerate))
        .routes(routes!(handlers::get_rtsp_url, handlers::update_rtsp_url))
        .routes(routes!(handlers::get_trim_snapshots, handlers::update_trim_snapshots))
        .routes(routes!(handlers::get_trim_history, handlers::update_trim_history))
        .route_layer(middleware::from_fn(auth::admin_middleware))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let (axum_router, api) = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        .with_state(state)
        .split_for_parts();

    axum_router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(cors)
}