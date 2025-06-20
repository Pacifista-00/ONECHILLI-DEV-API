// src/server.rs
use crate::config::AppConfig;
use crate::database::Database;
use crate::request::{extract_goods_query_params, extract_inventory_query_params};
use crate::response::{ErrorResponse, success_response, health_response};
use crate::tables::{CreateGoodRequest, UpdateGoodRequest, CreateInventoryRequest, UpdateInventoryRequest};
use crate::utils::{logging::*, response::*};
use axum::{
    extract::{Query, State},
    response::Response,
    routing::{get, post, put, delete},
    Json, Router,
};
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

pub struct Server {
    config: AppConfig,
    database: Database,
}

impl Server {
    pub fn new(config: AppConfig, database: Database) -> Self {
        Self { config, database }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let host = self.config.server.host.clone();
        let port = self.config.server.port;
        
        let app_state = AppState {
            database: self.database,
        };

        let app = Self::create_router(app_state);

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;

        info!("Server running on {}:{}", host, port);

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn create_router(state: AppState) -> Router {
        Router::new()
            .route("/", get(api_health))
            .route("/health", get(database_health))
            // Goods routes
            .route("/goods", get(get_goods))
            .route("/goods", post(create_goods))
            .route("/goods", put(update_goods))
            .route("/goods", delete(delete_goods))
            // Inventory routes
            .route("/inventory", get(get_inventory))
            .route("/inventory", post(create_inventory))
            .route("/inventory", put(update_inventory))
            .route("/inventory", delete(delete_inventory))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            )
            .with_state(state)
    }
}

// Route: GET / - API health check
async fn api_health() -> Response {
    info!("API health check requested");
    success_response(
        serde_json::json!({
            "status": "working",
            "service": "Rust API",
            "version": "1.0.0"
        }),
        "API is working correctly"
    )
}

// Route: GET /health - Database health check
async fn database_health(State(state): State<AppState>) -> Response {
    info!("Database health check requested");
    
    match state.database.health_check().await {
        Ok(_) => {
            info!("Database health check passed");
            health_response(true)
        }
        Err(e) => {
            log_database_error("health check", &e);
            health_response(false)
        }
    }
}

// GOODS ROUTES

// Route: POST /goods - Create new goods
async fn create_goods(
    State(state): State<AppState>,
    Json(request): Json<CreateGoodRequest>,
) -> Response {
    log_request_params("create goods", &request);

    // Validate request
    match request.validate() {
        Ok(_) => {}
        Err(validation_error) => {
            log_validation_error("create goods", &validation_error);
            return ErrorResponse::bad_request(&validation_error);
        }
    }

    // Insert goods
    match state.database.goods_table.insert(request).await {
        Ok(goods) => {
            log_success("create goods", &goods, 1);
            success_response(goods, &format_success_message("Goods creation", 1))
        }
        Err(e) => {
            log_database_error("create goods", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "goods creation"))
        }
    }
}

// Route: PUT /goods - Update goods with query parameters
async fn update_goods(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
    Json(request): Json<UpdateGoodRequest>,
) -> Response {
    let query_params = extract_goods_query_params(query);
    log_request_params("update goods", &(&query_params, &request));

    // Validate update request
    match request.validate() {
        Ok(_) => {}
        Err(validation_error) => {
            log_validation_error("update goods", &validation_error);
            return ErrorResponse::bad_request(&validation_error);
        }
    }

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required to specify which goods to update";
        log_validation_error("update goods", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("update goods", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database update
    match state.database.goods_table.update(search_params, request).await {
        Ok(updated_goods) => {
            if updated_goods.is_empty() {
                warn!("No goods found to update");
                return ErrorResponse::bad_request("No goods found to update");
            }
            let count = updated_goods.len();
            log_success("update goods", &updated_goods, count);
            success_response(updated_goods, &format_success_message("Goods update", count))
        }
        Err(e) => {
            log_database_error("update goods", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "goods update"))
        }
    }
}

// Route: DELETE /goods - Delete goods with query parameters
async fn delete_goods(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Response {
    let query_params = extract_goods_query_params(query);
    log_request_params("delete goods", &query_params);

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required to specify which goods to delete";
        log_validation_error("delete goods", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("delete goods", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database deletion
    match state.database.goods_table.delete(search_params).await {
        Ok(deleted_ids) => {
            if deleted_ids.is_empty() {
                warn!("No goods found to delete");
                return ErrorResponse::bad_request("No goods found to delete");
            }
            let count = deleted_ids.len();
            log_success("delete goods", &deleted_ids, count);
            success_response(deleted_ids, &format_success_message("Goods deletion", count))
        }
        Err(e) => {
            log_database_error("delete goods", &e);
            // Check if it's a foreign key constraint violation
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23503")) {
                    return ErrorResponse::bad_request(&crate::utils::response::format_database_error(&e, "goods deletion"));
                }
            }
            ErrorResponse::internal_server_error(&format_database_error(&e, "goods deletion"))
        }
    }
}

// Route: GET /goods - Get goods with query parameters
async fn get_goods(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Response {
    let query_params = extract_goods_query_params(query);
    log_request_params("search goods", &query_params);

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required. Use goods_name=* or material_code=* to get all goods, or specify search criteria like goods_id, material_code, goods_name, price, volumn_l, mass_g, min_volumn_l, max_volumn_l, min_mass_g, max_mass_g, min_price, max_price";
        log_validation_error("search goods", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("search goods", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database search
    match state.database.goods_table.search(search_params).await {
        Ok(goods) => {
            let count = goods.len();
            log_success("search goods", &goods, count);
            success_response(goods, &format_success_message("Goods search", count))
        }
        Err(e) => {
            log_database_error("search goods", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "goods search"))
        }
    }
}

// INVENTORY ROUTES

// Route: GET /inventory - Get inventory with query parameters
async fn get_inventory(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Response {
    let query_params = extract_inventory_query_params(query);
    log_request_params("search inventory", &query_params);

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required. Use goods_name=* or material_code=* to get all inventory, or specify search criteria like item_id, goods_id, material_code, goods_name, quantity, min_quantity, max_quantity, expired_date, min_expired_date, max_expired_date, and all goods search parameters";
        log_validation_error("search inventory", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("search inventory", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database search
    match state.database.inventory_table.search(search_params).await {
        Ok(inventory) => {
            let count = inventory.len();
            log_success("search inventory", &inventory, count);
            success_response(inventory, &format_success_message("Inventory search", count))
        }
        Err(e) => {
            log_database_error("search inventory", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "inventory search"))
        }
    }
}

// Route: POST /inventory - Create new inventory item
async fn create_inventory(
    State(state): State<AppState>,
    Json(request): Json<CreateInventoryRequest>,
) -> Response {
    log_request_params("create inventory", &request);

    // Validate request
    match request.validate() {
        Ok(_) => {}
        Err(validation_error) => {
            log_validation_error("create inventory", &validation_error);
            return ErrorResponse::bad_request(&validation_error);
        }
    }

    // Insert inventory item
    match state.database.inventory_table.insert(request).await {
        Ok((inventory_item, is_new)) => {
            if is_new {
                log_success("create inventory (new)", &inventory_item, 1);
                success_response(
                    inventory_item, 
                    &format_success_message("Inventory creation", 1)
                )
            } else {
                log_success("create inventory (existing found)", &inventory_item, 1);
                success_response(
                    inventory_item, 
                    "Inventory item already exists with same goods and expiration date. Returning existing item."
                )
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            let error = "Referenced goods not found. Please provide valid goods_id, material_code, or complete goods information.";
            log_validation_error("create inventory", error);
            ErrorResponse::bad_request(error)
        }
        Err(e) => {
            log_database_error("create inventory", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "inventory creation"))
        }
    }
}

// Route: PUT /inventory - Update inventory with query parameters
async fn update_inventory(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
    Json(request): Json<UpdateInventoryRequest>,
) -> Response {
    let query_params = extract_inventory_query_params(query);
    log_request_params("update inventory", &(&query_params, &request));

    // Validate update request
    match request.validate() {
        Ok(_) => {}
        Err(validation_error) => {
            log_validation_error("update inventory", &validation_error);
            return ErrorResponse::bad_request(&validation_error);
        }
    }

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required to specify which inventory items to update";
        log_validation_error("update inventory", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("update inventory", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database update
    match state.database.inventory_table.update(search_params, request).await {
        Ok(updated_items) => {
            if updated_items.is_empty() {
                warn!("No inventory items found to update");
                return ErrorResponse::bad_request("No inventory items found to update");
            }
            let count = updated_items.len();
            log_success("update inventory", &updated_items, count);
            success_response(updated_items, &format_success_message("Inventory update", count))
        }
        Err(e) => {
            log_database_error("update inventory", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "inventory update"))
        }
    }
}

// Route: DELETE /inventory - Delete inventory with query parameters
async fn delete_inventory(
    State(state): State<AppState>,
    query: Query<HashMap<String, String>>,
) -> Response {
    let query_params = extract_inventory_query_params(query);
    log_request_params("delete inventory", &query_params);

    // Check if no parameters provided
    if !query_params.has_any_params() {
        let error = "Query parameters required to specify which inventory items to delete";
        log_validation_error("delete inventory", error);
        return ErrorResponse::bad_request(error);
    }

    // Validate and parse query parameters
    let search_params = match query_params.validate_and_parse() {
        Ok(params) => params,
        Err(parse_error) => {
            log_validation_error("delete inventory", &parse_error);
            return ErrorResponse::bad_request(&format!("Invalid query parameters: {}", parse_error));
        }
    };

    // Perform database deletion
    match state.database.inventory_table.delete(search_params).await {
        Ok(deleted_ids) => {
            if deleted_ids.is_empty() {
                warn!("No inventory items found to delete");
                return ErrorResponse::bad_request("No inventory items found to delete");
            }
            let count = deleted_ids.len();
            log_success("delete inventory", &deleted_ids, count);
            success_response(deleted_ids, &format_success_message("Inventory deletion", count))
        }
        Err(e) => {
            log_database_error("delete inventory", &e);
            ErrorResponse::internal_server_error(&format_database_error(&e, "inventory deletion"))
        }
    }
}