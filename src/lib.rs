use axum::{body::Body, http::header, response::IntoResponse, routing::get, Json, Router};
use axum_embed::ServeEmbed;

use rust_embed::RustEmbed;

static INITIALIZER: &str = r#"window.onload = function () {
window.ui = SwaggerUIBundle({
  url: "$API_JSON$",
  dom_id: '#swagger-ui',
  deepLinking: true,
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
  layout: "StandaloneLayout"
});
}"#;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
struct Assets;

static API_JSON_URL: &str = "/api.json";

pub struct Swagger {
    swagger_path: String,
    docs: utoipa::openapi::OpenApi,
}

impl Swagger {
    pub fn new<U: Into<String>>(swagger_path: U, docs: utoipa::openapi::OpenApi) -> Self {
        Self {
            swagger_path: swagger_path.into(),
            docs,
        }
    }

    pub fn to_router(&self) -> Router {
        let serve_assets = ServeEmbed::<Assets>::new();

        let swagger_router = Router::new()
            .route(API_JSON_URL, get(self.serve_openapi_json()))
            .route(
                format!("{}/swaggerconfig.js", &self.swagger_path).as_str(),
                get(move || Self::serve_swagger_config(API_JSON_URL)),
            )
            .nest_service(&self.swagger_path, serve_assets);

        swagger_router
    }

    async fn serve_swagger_config(api_json_url: &str) -> impl IntoResponse {
        let headers = [(header::CONTENT_TYPE, "text/javascript")];
        (
            headers,
            Body::from(INITIALIZER.replace("$API_JSON$", api_json_url)),
        )
    }

    fn serve_openapi_json(&self) -> Json<utoipa::openapi::OpenApi> {
        Json(self.docs.clone())
    }
}
