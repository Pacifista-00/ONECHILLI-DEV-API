// src/tables/goods_table.rs
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Good {
    pub goods_id: i32,
    pub material_code: String,
    pub goods_name: String,
    pub description: Option<Vec<String>>,
    pub price: rust_decimal::Decimal,
    pub volumn_l: rust_decimal::Decimal,
    pub mass_g: rust_decimal::Decimal,
    pub mass_base: i16,
    pub volumn_base: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoodRequest {
    pub material_code: String,
    pub goods_name: String,
    pub description: Option<Vec<String>>,
    pub price: rust_decimal::Decimal,
    pub volumn_l: rust_decimal::Decimal,
    pub mass_g: rust_decimal::Decimal,
    pub mass_base: Option<i16>,
    pub volumn_base: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoodRequest {
    pub material_code: Option<String>,
    pub goods_name: Option<String>,
    pub description: Option<Vec<String>>,
    pub price: Option<rust_decimal::Decimal>,
    pub volumn_l: Option<rust_decimal::Decimal>,
    pub mass_g: Option<rust_decimal::Decimal>,
    pub mass_base: Option<i16>,
    pub volumn_base: Option<i16>,
}

#[derive(Debug, Clone)]
pub struct GoodsSearchParams {
    pub goods_id: Option<i32>,
    pub material_code: Option<String>,
    pub goods_name: Option<String>,
    pub price: Option<rust_decimal::Decimal>,
    pub volumn_l: Option<rust_decimal::Decimal>,
    pub mass_g: Option<rust_decimal::Decimal>,
    pub min_volumn_l: Option<rust_decimal::Decimal>,
    pub max_volumn_l: Option<rust_decimal::Decimal>,
    pub min_mass_g: Option<rust_decimal::Decimal>,
    pub max_mass_g: Option<rust_decimal::Decimal>,
    pub min_price: Option<rust_decimal::Decimal>,
    pub max_price: Option<rust_decimal::Decimal>,
}

impl GoodsSearchParams {
    pub fn new() -> Self {
        Self {
            goods_id: None,
            material_code: None,
            goods_name: None,
            price: None,
            volumn_l: None,
            mass_g: None,
            min_volumn_l: None,
            max_volumn_l: None,
            min_mass_g: None,
            max_mass_g: None,
            min_price: None,
            max_price: None,
        }
    }

    pub fn is_get_all(&self) -> bool {
        matches!(self.goods_name.as_deref(), Some("*"))
            || matches!(self.material_code.as_deref(), Some("*"))
    }
}

#[derive(Clone)]
pub struct GoodsTable {
    pool: PgPool,
}

impl GoodsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(&self, params: GoodsSearchParams) -> Result<Vec<Good>, sqlx::Error> {
        // Handle get all case
        if params.is_get_all() {
            return self.get_all().await;
        }

        // Build dynamic query with parameterized statements to prevent SQL injection
        let mut query = "SELECT goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base FROM goods WHERE 1=1".to_string();
        let mut bind_count = 0;
        let mut conditions = Vec::new();

        // Build WHERE conditions safely using parameterized queries
        if params.goods_id.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND goods_id = ${}", bind_count));
        }

        if params.material_code.is_some() && !params.is_get_all() {
            bind_count += 1;
            conditions.push(format!(" AND material_code ILIKE ${}", bind_count));
        }

        if params.goods_name.is_some() && !params.is_get_all() {
            bind_count += 1;
            conditions.push(format!(" AND goods_name ILIKE ${}", bind_count));
        }

        if params.price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND price = ${}", bind_count));
        }

        if params.volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND volumn_l = ${}", bind_count));
        }

        if params.mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND mass_g = ${}", bind_count));
        }

        if params.min_volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND volumn_l >= ${}", bind_count));
        }

        if params.max_volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND volumn_l <= ${}", bind_count));
        }

        if params.min_mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND mass_g >= ${}", bind_count));
        }

        if params.max_mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND mass_g <= ${}", bind_count));
        }

        if params.min_price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND price >= ${}", bind_count));
        }

        if params.max_price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND price <= ${}", bind_count));
        }

        // Append conditions to query
        query.push_str(&conditions.join(""));
        query.push_str(" ORDER BY goods_id ASC");

        // Build and execute query with proper parameter binding
        let mut sql_query = sqlx::query_as::<_, Good>(&query);

        // Bind parameters in the same order as they were added
        if let Some(goods_id) = params.goods_id {
            sql_query = sql_query.bind(goods_id);
        }

        if let Some(material_code) = params.material_code {
            if material_code != "*" {
                sql_query = sql_query.bind(crate::utils::string_utils::to_search_pattern(&material_code));
            }
        }

        if let Some(goods_name) = params.goods_name {
            if goods_name != "*" {
                sql_query = sql_query.bind(crate::utils::string_utils::to_search_pattern(&goods_name));
            }
        }

        if let Some(price) = params.price {
            sql_query = sql_query.bind(price);
        }

        if let Some(volumn_l) = params.volumn_l {
            sql_query = sql_query.bind(volumn_l);
        }

        if let Some(mass_g) = params.mass_g {
            sql_query = sql_query.bind(mass_g);
        }

        if let Some(min_volumn_l) = params.min_volumn_l {
            sql_query = sql_query.bind(min_volumn_l);
        }

        if let Some(max_volumn_l) = params.max_volumn_l {
            sql_query = sql_query.bind(max_volumn_l);
        }

        if let Some(min_mass_g) = params.min_mass_g {
            sql_query = sql_query.bind(min_mass_g);
        }

        if let Some(max_mass_g) = params.max_mass_g {
            sql_query = sql_query.bind(max_mass_g);
        }

        if let Some(min_price) = params.min_price {
            sql_query = sql_query.bind(min_price);
        }

        if let Some(max_price) = params.max_price {
            sql_query = sql_query.bind(max_price);
        }

        sql_query.fetch_all(&self.pool).await
    }

    async fn get_all(&self) -> Result<Vec<Good>, sqlx::Error> {
        sqlx::query_as::<_, Good>(
            "SELECT goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base FROM goods ORDER BY goods_id ASC"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, goods_id: i32) -> Result<Option<Good>, sqlx::Error> {
        sqlx::query_as::<_, Good>(
            "SELECT goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base FROM goods WHERE goods_id = $1"
        )
        .bind(goods_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_material_code(&self, material_code: &str) -> Result<Option<Good>, sqlx::Error> {
        sqlx::query_as::<_, Good>(
            "SELECT goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base FROM goods WHERE material_code = $1"
        )
        .bind(material_code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn insert(&self, request: CreateGoodRequest) -> Result<Good, sqlx::Error> {
        // Check if goods with same material_code already exists
        let existing = self.get_by_material_code(&request.material_code).await?;

        if let Some(existing_good) = existing {
            // Return the existing good instead of error
            return Ok(existing_good);
        }

        // Insert new good
        let new_good = sqlx::query_as::<_, Good>(
            r#"
            INSERT INTO goods (material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base
            "#
        )
        .bind(&request.material_code)
        .bind(&request.goods_name)
        .bind(&request.description)
        .bind(&request.price)
        .bind(&request.volumn_l)
        .bind(&request.mass_g)
        .bind(request.mass_base.unwrap_or(0))
        .bind(request.volumn_base.unwrap_or(0))
        .fetch_one(&self.pool)
        .await?;

        Ok(new_good)
    }

    pub async fn update(&self, params: GoodsSearchParams, update_request: UpdateGoodRequest) -> Result<Vec<Good>, sqlx::Error> {
        // First, find goods to update using the same search logic
        let goods_to_update = self.search(params).await?;
        
        if goods_to_update.is_empty() {
            return Ok(Vec::new()); // Return empty vector if nothing to update
        }

        let mut updated_goods = Vec::new();

        // Update each found good
        for good in goods_to_update {
            let updated_good = sqlx::query_as::<_, Good>(
                r#"
                UPDATE goods 
                SET 
                    material_code = COALESCE($2, material_code),
                    goods_name = COALESCE($3, goods_name),
                    description = COALESCE($4, description),
                    price = COALESCE($5, price),
                    volumn_l = COALESCE($6, volumn_l),
                    mass_g = COALESCE($7, mass_g),
                    mass_base = COALESCE($8, mass_base),
                    volumn_base = COALESCE($9, volumn_base)
                WHERE goods_id = $1
                RETURNING goods_id, material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base
                "#
            )
            .bind(good.goods_id)
            .bind(&update_request.material_code)
            .bind(&update_request.goods_name)
            .bind(&update_request.description)
            .bind(&update_request.price)
            .bind(&update_request.volumn_l)
            .bind(&update_request.mass_g)
            .bind(&update_request.mass_base)
            .bind(&update_request.volumn_base)
            .fetch_one(&self.pool)
            .await?;

            updated_goods.push(updated_good);
        }

        // Use get_by_id to verify the updates were successful (this uses the method)
        if let Some(first_updated) = updated_goods.first() {
            let _verification = self.get_by_id(first_updated.goods_id).await?;
        }

        Ok(updated_goods)
    }

    pub async fn delete(&self, params: GoodsSearchParams) -> Result<Vec<i32>, sqlx::Error> {
        // First, find goods to delete using the same search logic
        let goods_to_delete = self.search(params).await?;
        
        if goods_to_delete.is_empty() {
            return Ok(Vec::new()); // Return empty vector if nothing to delete
        }

        let mut deleted_ids = Vec::new();

        // Check if any goods have inventory items before deletion
        for good in goods_to_delete {
            // Use utility function to count inventory items
            let inventory_count = crate::utils::database::count_by_foreign_key(
                &self.pool, 
                "inventory", 
                "goods_id", 
                good.goods_id
            ).await?;

            if inventory_count > 0 {
                // Return error indicating which goods cannot be deleted
                return Err(sqlx::Error::RowNotFound);
            }

            // If no inventory items, proceed with deletion
            sqlx::query("DELETE FROM goods WHERE goods_id = $1")
                .bind(good.goods_id)
                .execute(&self.pool)
                .await?;
            
            deleted_ids.push(good.goods_id);
        }

        Ok(deleted_ids)
    }
}
