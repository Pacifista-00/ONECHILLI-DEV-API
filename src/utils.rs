// src/utils.rs
#![allow(dead_code)]
use sqlx::PgPool;

/// Validation utilities for input sanitization
pub mod validation {
    use std::str::FromStr;
    use chrono::{DateTime, Utc};

    /// Check if a string is a safe integer (prevents SQL injection)
    pub fn is_safe_integer(input: &str) -> bool {
        !input.is_empty() 
            && input.chars().all(|c| c.is_ascii_digit() || c == '-')
            && input.matches('-').count() <= 1
            && !input.starts_with("--")
    }

    /// Check if a string is safe for database queries (prevents SQL injection)
    pub fn is_safe_string(input: &str) -> bool {
        // Check for empty string
        if input.is_empty() {
            return false;
        }

        // Allow wildcard for search
        if input == "*" {
            return true;
        }

        // Explicitly reject SQL injection attempts
        let dangerous_patterns = [
            "select", "insert", "update", "delete", "drop", "alter", "create",
            "union", "script", "--", "/*", "*/", "'", "\"", ";", "\\",
            "exec", "execute", "sp_", "xp_"
        ];
        
        let input_lower = input.to_lowercase();
        
        // Check for dangerous SQL patterns
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                return false;
            }
        }
        
        // Allow alphanumeric, spaces, common punctuation, and Unicode characters
        // This is more permissive to allow international characters like Thai
        input.chars().all(|c| {
            c.is_alphanumeric() 
            || c.is_whitespace() 
            || matches!(c, '.' | '-' | '_' | '*' | '(' | ')' | '[' | ']' | '+' | '/' | '@' | '#' | ':' | '&' | '!' | '?' | ',' | '%')
            || c as u32 > 127  // Allow Unicode characters (like Thai)
        })
    }

    /// Check if a string is a safe decimal number
    pub fn is_safe_decimal(input: &str) -> bool {
        !input.is_empty()
            && input.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
            && input.matches('.').count() <= 1
            && input.matches('-').count() <= 1
            && !input.starts_with("--")
            && !input.ends_with(".")
            && !input.starts_with(".")
    }

    /// Check if a string is a safe datetime format
    pub fn is_safe_datetime(input: &str) -> bool {
        !input.is_empty()
            && input.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | 'T' | 'Z' | '+' | '.'))
            && input.len() >= 10  // Minimum length for date
            && input.len() <= 30  // Maximum reasonable length
    }

    /// Parse and validate integer string
    pub fn parse_safe_integer(input: &str, field_name: &str) -> Result<i32, String> {
        if !is_safe_integer(input) {
            return Err(format!("Invalid {} format", field_name));
        }
        input.parse::<i32>()
            .map_err(|_| format!("Invalid integer format for {}", field_name))
    }

    /// Parse and validate decimal string
    pub fn parse_safe_decimal(input: &str, field_name: &str) -> Result<rust_decimal::Decimal, String> {
        if !is_safe_decimal(input) {
            return Err(format!("Invalid {} format", field_name));
        }
        rust_decimal::Decimal::from_str(input)
            .map_err(|_| format!("Invalid decimal format for {}", field_name))
    }

    /// Parse and validate datetime string
    pub fn parse_safe_datetime(input: &str, field_name: &str) -> Result<DateTime<Utc>, String> {
        if !is_safe_datetime(input) {
            return Err(format!("Invalid {} format", field_name));
        }
        input.parse::<DateTime<Utc>>()
            .map_err(|_| format!("Invalid datetime format for {}. Use ISO 8601 format (e.g., 2024-12-31T23:59:59Z)", field_name))
    }

    /// Validate string and return error if invalid
    pub fn validate_safe_string(input: &str, field_name: &str) -> Result<(), String> {
        if input.is_empty() {
            return Err(format!("{} cannot be empty", field_name));
        }
        if !is_safe_string(input) {
            return Err(format!("Invalid {}", field_name));
        }
        Ok(())
    }
}

/// Database utility functions
pub mod database {
    use super::*;

    /// Check if a record exists in a table by ID
    pub async fn exists_by_id(pool: &PgPool, table: &str, id_column: &str, id: i32) -> Result<bool, sqlx::Error> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)", table, id_column);
        sqlx::query_scalar::<_, bool>(&query)
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Check if a record exists in a table by string field
    pub async fn exists_by_string(pool: &PgPool, table: &str, column: &str, value: &str) -> Result<bool, sqlx::Error> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)", table, column);
        sqlx::query_scalar::<_, bool>(&query)
            .bind(value)
            .fetch_one(pool)
            .await
    }

    /// Get a single ID by string field
    pub async fn get_id_by_string(pool: &PgPool, table: &str, id_column: &str, search_column: &str, value: &str) -> Result<Option<i32>, sqlx::Error> {
        let query = format!("SELECT {} FROM {} WHERE {} = $1", id_column, table, search_column);
        sqlx::query_scalar::<_, i32>(&query)
            .bind(value)
            .fetch_optional(pool)
            .await
    }

    /// Count records in a table by foreign key
    pub async fn count_by_foreign_key(pool: &PgPool, table: &str, fk_column: &str, fk_id: i32) -> Result<i64, sqlx::Error> {
        let query = format!("SELECT COUNT(*) FROM {} WHERE {} = $1", table, fk_column);
        sqlx::query_scalar::<_, i64>(&query)
            .bind(fk_id)
            .fetch_one(pool)
            .await
    }

    /// Verify table access by running a simple query
    pub async fn verify_table_access(pool: &PgPool, table: &str) -> Result<(), sqlx::Error> {
        let query = format!("SELECT 1 FROM {} LIMIT 1", table);
        sqlx::query(&query)
            .execute(pool)
            .await
            .map(|_| ())
    }
}

/// Query building utilities for dynamic SQL generation
pub mod query_builder {
    /// Dynamic query builder for search operations
    pub struct SearchQueryBuilder {
        base_query: String,
        conditions: Vec<String>,
        bind_count: usize,
    }

    impl SearchQueryBuilder {
        pub fn new(base_query: String) -> Self {
            Self {
                base_query,
                conditions: Vec::new(),
                bind_count: 0,
            }
        }

        /// Add a condition with a parameter placeholder
        pub fn add_condition(&mut self, condition: &str) -> usize {
            self.bind_count += 1;
            self.conditions.push(format!(" AND {}", condition.replace("?", &format!("${}", self.bind_count))));
            self.bind_count
        }

        /// Add an optional condition if the value is Some
        pub fn add_optional_condition<T>(&mut self, condition: &str, value: &Option<T>) -> Option<usize> {
            if value.is_some() {
                Some(self.add_condition(condition))
            } else {
                None
            }
        }

        /// Build the final query string
        pub fn build(self, order_by: Option<&str>) -> String {
            let mut query = self.base_query;
            
            if !self.conditions.is_empty() {
                query.push_str(&self.conditions.join(""));
            }
            
            if let Some(order) = order_by {
                query.push_str(&format!(" ORDER BY {}", order));
            }
            
            query
        }

        /// Get the current bind count
        pub fn bind_count(&self) -> usize {
            self.bind_count
        }
    }
}

/// Response formatting utilities
pub mod response {
    /// Create a standardized error message
    pub fn format_error_message(operation: &str, details: &str) -> String {
        format!("{}: {}", operation, details)
    }

    /// Create a standardized success message
    pub fn format_success_message(operation: &str, count: usize) -> String {
        match count {
            0 => format!("No items found for {}", operation),
            1 => format!("{} completed successfully", operation),
            n => format!("{} completed successfully for {} items", operation, n),
        }
    }

    /// Format database error for user response
    pub fn format_database_error(error: &sqlx::Error, operation: &str) -> String {
        match error {
            sqlx::Error::RowNotFound => {
                format!("No records found for {}", operation)
            }
            sqlx::Error::Database(db_err) => {
                // Handle specific database error codes
                match db_err.code().as_deref() {
                    Some("23503") => "Cannot perform operation due to foreign key constraint".to_string(),
                    Some("23505") => "Record already exists".to_string(),
                    Some("23514") => "Data validation failed".to_string(),
                    _ => format!("Database error during {}", operation),
                }
            }
            _ => format!("Internal error during {}", operation),
        }
    }
}

/// Date and time utilities
pub mod datetime {
    use chrono::{DateTime, Utc, NaiveDate};

    /// Parse date string in various formats
    pub fn parse_flexible_date(date_str: &str) -> Result<DateTime<Utc>, String> {
        // Try ISO 8601 format first
        if let Ok(dt) = date_str.parse::<DateTime<Utc>>() {
            return Ok(dt);
        }

        // Try date only format (YYYY-MM-DD)
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Ok(date.and_hms_opt(0, 0, 0)
                .ok_or("Invalid date")?
                .and_utc());
        }

        // Try other common formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d",
            "%d/%m/%Y",
            "%m/%d/%Y",
        ];

        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Ok(date.and_hms_opt(0, 0, 0)
                    .ok_or("Invalid date")?
                    .and_utc());
            }
        }

        Err("Unable to parse date. Use ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ) or YYYY-MM-DD".to_string())
    }

    /// Check if a date is in the past
    pub fn is_expired(date: &DateTime<Utc>) -> bool {
        *date < Utc::now()
    }

    /// Get days until expiration (negative if expired)
    pub fn days_until_expiration(date: &DateTime<Utc>) -> i64 {
        (date.timestamp() - Utc::now().timestamp()) / 86400
    }
}

/// String manipulation utilities
pub mod string_utils {
    /// Convert string to search pattern for ILIKE queries
    pub fn to_search_pattern(input: &str) -> String {
        if input == "*" {
            "%".to_string()
        } else {
            format!("%{}%", input.replace("%", "\\%").replace("_", "\\_"))
        }
    }

    /// Truncate string to maximum length
    pub fn truncate(input: &str, max_len: usize) -> String {
        if input.len() <= max_len {
            input.to_string()
        } else {
            format!("{}...", &input[..max_len.saturating_sub(3)])
        }
    }

    /// Sanitize string for logging (remove sensitive information)
    pub fn sanitize_for_log(input: &str) -> String {
        // Replace potential sensitive patterns
        input.replace("password", "***")
             .replace("token", "***")
             .replace("secret", "***")
    }
}

/// Pagination utilities
pub mod pagination {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PaginationParams {
        pub page: Option<u32>,
        pub per_page: Option<u32>,
    }

    impl PaginationParams {
        pub fn new() -> Self {
            Self {
                page: None,
                per_page: None,
            }
        }

        pub fn page(&self) -> u32 {
            self.page.unwrap_or(1)
        }

        pub fn per_page(&self) -> u32 {
            self.per_page.unwrap_or(50).min(1000) // Cap at 1000 items per page
        }

        pub fn offset(&self) -> u32 {
            (self.page().saturating_sub(1)) * self.per_page()
        }

        pub fn limit(&self) -> u32 {
            self.per_page()
        }
    }

    #[derive(Debug, Serialize)]
    pub struct PaginatedResponse<T> {
        pub data: Vec<T>,
        pub page: u32,
        pub per_page: u32,
        pub total_count: Option<u64>,
        pub total_pages: Option<u32>,
    }

    impl<T> PaginatedResponse<T> {
        pub fn new(data: Vec<T>, params: &PaginationParams, total_count: Option<u64>) -> Self {
            let total_pages = total_count.map(|count| {
                ((count as f64) / (params.per_page() as f64)).ceil() as u32
            });

            Self {
                data,
                page: params.page(),
                per_page: params.per_page(),
                total_count,
                total_pages,
            }
        }
    }
}

/// Logging utilities
/// Logging utilities
pub mod logging {
    use tracing::{info, warn, error};
    // No need to explicitly use `serde::Serialize` if you're not calling its methods
    // use serde::Serialize; // This line might be removed if only Debug is needed

    /// Log a successful operation
    pub fn log_success<T: std::fmt::Debug>(operation: &str, data: &T, count: usize) {
        info!(
            operation = operation,
            data = ?data, // Log the data using its Debug representation
            count = count,
            "Operation completed successfully"
        );
    }

    /// Log a validation error
    pub fn log_validation_error(operation: &str, error: &str) {
        warn!(
            operation = operation,
            error = error,
            "Validation failed"
        );
    }

    /// Log a database error
    pub fn log_database_error(operation: &str, error: &sqlx::Error) {
        error!(
            operation = operation,
            error = %error, // Using % for Display
            "Database operation failed"
        );
    }

    /// Log request parameters for debugging
    pub fn log_request_params<T: serde::Serialize + std::fmt::Debug>(operation: &str, params: &T) {
        info!(
            operation = operation,
            params = ?params, // Log the params using Debug
            "Request received"
        );
    }
}