// src/tables/inventory_table.rs
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use chrono::{DateTime, Utc};
use super::goods_table::GoodsSearchParams;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub item_id: i32,
    pub goods_id: i32,
    pub quantity: i32,
    pub expired_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemWithGoods {
    pub item_id: i32,
    pub goods_id: i32,
    pub material_code: String,
    pub goods_name: String,
    pub description: Option<Vec<String>>,
    pub price: rust_decimal::Decimal,
    pub volumn_l: rust_decimal::Decimal,
    pub mass_g: rust_decimal::Decimal,
    pub mass_base: i16,
    pub volumn_base: i16,
    pub quantity: i32,
    pub expired_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInventoryRequest {
    // Option 1: Use existing goods by ID or material code
    pub goods_id: Option<i32>,
    pub material_code: Option<String>,
    
    // Option 2: Create new goods with full details
    pub goods_name: Option<String>,
    pub description: Option<Vec<String>>,
    pub price: Option<rust_decimal::Decimal>,
    pub volumn_l: Option<rust_decimal::Decimal>,
    pub mass_g: Option<rust_decimal::Decimal>,
    pub mass_base: Option<i16>,
    pub volumn_base: Option<i16>,
    
    // Inventory specific fields
    pub quantity: i32,
    pub expired_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInventoryRequest {
    // Goods fields (optional updates)
    pub material_code: Option<String>,
    pub goods_name: Option<String>,
    pub description: Option<Vec<String>>,
    pub price: Option<rust_decimal::Decimal>,
    pub volumn_l: Option<rust_decimal::Decimal>,
    pub mass_g: Option<rust_decimal::Decimal>,
    pub mass_base: Option<i16>,
    pub volumn_base: Option<i16>,
    
    // Inventory fields (optional updates)
    pub quantity: Option<i32>,
    pub expired_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct InventorySearchParams {
    // Inventory specific search params
    pub item_id: Option<i32>,
    pub quantity: Option<i32>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub expired_date: Option<DateTime<Utc>>,
    pub min_expired_date: Option<DateTime<Utc>>,
    pub max_expired_date: Option<DateTime<Utc>>,
    
    // Goods search params (inherited)
    pub goods_params: GoodsSearchParams,
}

impl InventorySearchParams {
    pub fn new() -> Self {
        Self {
            item_id: None,
            quantity: None,
            min_quantity: None,
            max_quantity: None,
            expired_date: None,
            min_expired_date: None,
            max_expired_date: None,
            goods_params: GoodsSearchParams::new(),
        }
    }

    pub fn is_get_all(&self) -> bool {
        self.goods_params.is_get_all()
    }
}

#[derive(Clone)]
pub struct InventoryTable {
    pool: PgPool,
}

impl InventoryTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(&self, params: InventorySearchParams) -> Result<Vec<InventoryItemWithGoods>, sqlx::Error> {
        // Handle get all case
        if params.is_get_all() {
            return self.get_all().await;
        }

        // Build dynamic query with JOIN to goods table
        let mut query = r#"
            SELECT 
                i.item_id, i.goods_id, i.quantity, i.expired_date,
                g.material_code, g.goods_name, g.description, g.price, 
                g.volumn_l, g.mass_g, g.mass_base, g.volumn_base
            FROM inventory i
            INNER JOIN goods g ON i.goods_id = g.goods_id
            WHERE 1=1"#.to_string();
        
        let mut bind_count = 0;
        let mut conditions = Vec::new();

        // Inventory specific conditions
        if params.item_id.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.item_id = ${}", bind_count));
        }

        if params.quantity.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.quantity = ${}", bind_count));
        }

        if params.min_quantity.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.quantity >= ${}", bind_count));
        }

        if params.max_quantity.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.quantity <= ${}", bind_count));
        }

        if params.expired_date.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.expired_date = ${}", bind_count));
        }

        if params.min_expired_date.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.expired_date >= ${}", bind_count));
        }

        if params.max_expired_date.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND i.expired_date <= ${}", bind_count));
        }

        // Goods related conditions
        if params.goods_params.goods_id.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.goods_id = ${}", bind_count));
        }

        if params.goods_params.material_code.is_some() && !params.goods_params.is_get_all() {
            bind_count += 1;
            conditions.push(format!(" AND g.material_code ILIKE ${}", bind_count));
        }

        if params.goods_params.goods_name.is_some() && !params.goods_params.is_get_all() {
            bind_count += 1;
            conditions.push(format!(" AND g.goods_name ILIKE ${}", bind_count));
        }

        if params.goods_params.price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.price = ${}", bind_count));
        }

        if params.goods_params.volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.volumn_l = ${}", bind_count));
        }

        if params.goods_params.mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.mass_g = ${}", bind_count));
        }

        if params.goods_params.min_volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.volumn_l >= ${}", bind_count));
        }

        if params.goods_params.max_volumn_l.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.volumn_l <= ${}", bind_count));
        }

        if params.goods_params.min_mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.mass_g >= ${}", bind_count));
        }

        if params.goods_params.max_mass_g.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.mass_g <= ${}", bind_count));
        }

        if params.goods_params.min_price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.price >= ${}", bind_count));
        }

        if params.goods_params.max_price.is_some() {
            bind_count += 1;
            conditions.push(format!(" AND g.price <= ${}", bind_count));
        }

        // Append conditions to query
        query.push_str(&conditions.join(""));
        query.push_str(" ORDER BY i.item_id ASC");

        // Build and execute query with proper parameter binding
        let mut sql_query = sqlx::query(&query);

        // Bind parameters in the same order as they were added
        if let Some(item_id) = params.item_id {
            sql_query = sql_query.bind(item_id);
        }

        if let Some(quantity) = params.quantity {
            sql_query = sql_query.bind(quantity);
        }

        if let Some(min_quantity) = params.min_quantity {
            sql_query = sql_query.bind(min_quantity);
        }

        if let Some(max_quantity) = params.max_quantity {
            sql_query = sql_query.bind(max_quantity);
        }

        if let Some(expired_date) = params.expired_date {
            sql_query = sql_query.bind(expired_date);
        }

        if let Some(min_expired_date) = params.min_expired_date {
            sql_query = sql_query.bind(min_expired_date);
        }

        if let Some(max_expired_date) = params.max_expired_date {
            sql_query = sql_query.bind(max_expired_date);
        }

        if let Some(goods_id) = params.goods_params.goods_id {
            sql_query = sql_query.bind(goods_id);
        }

        if let Some(material_code) = params.goods_params.material_code {
            if material_code != "*" {
                sql_query = sql_query.bind(crate::utils::string_utils::to_search_pattern(&material_code));
            }
        }

        if let Some(goods_name) = params.goods_params.goods_name {
            if goods_name != "*" {
                sql_query = sql_query.bind(crate::utils::string_utils::to_search_pattern(&goods_name));
            }
        }

        if let Some(price) = params.goods_params.price {
            sql_query = sql_query.bind(price);
        }

        if let Some(volumn_l) = params.goods_params.volumn_l {
            sql_query = sql_query.bind(volumn_l);
        }

        if let Some(mass_g) = params.goods_params.mass_g {
            sql_query = sql_query.bind(mass_g);
        }

        if let Some(min_volumn_l) = params.goods_params.min_volumn_l {
            sql_query = sql_query.bind(min_volumn_l);
        }

        if let Some(max_volumn_l) = params.goods_params.max_volumn_l {
            sql_query = sql_query.bind(max_volumn_l);
        }

        if let Some(min_mass_g) = params.goods_params.min_mass_g {
            sql_query = sql_query.bind(min_mass_g);
        }

        if let Some(max_mass_g) = params.goods_params.max_mass_g {
            sql_query = sql_query.bind(max_mass_g);
        }

        if let Some(min_price) = params.goods_params.min_price {
            sql_query = sql_query.bind(min_price);
        }

        if let Some(max_price) = params.goods_params.max_price {
            sql_query = sql_query.bind(max_price);
        }

        // Execute and map results
        let rows = sql_query.fetch_all(&self.pool).await?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(InventoryItemWithGoods {
                item_id: row.get("item_id"),
                goods_id: row.get("goods_id"),
                material_code: row.get("material_code"),
                goods_name: row.get("goods_name"),
                description: row.get("description"),
                price: row.get("price"),
                volumn_l: row.get("volumn_l"),
                mass_g: row.get("mass_g"),
                mass_base: row.get("mass_base"),
                volumn_base: row.get("volumn_base"),
                quantity: row.get("quantity"),
                expired_date: row.get("expired_date"),
            });
        }

        Ok(results)
    }

    async fn get_all(&self) -> Result<Vec<InventoryItemWithGoods>, sqlx::Error> {
        let query = r#"
            SELECT 
                i.item_id, i.goods_id, i.quantity, i.expired_date,
                g.material_code, g.goods_name, g.description, g.price, 
                g.volumn_l, g.mass_g, g.mass_base, g.volumn_base
            FROM inventory i
            INNER JOIN goods g ON i.goods_id = g.goods_id
            ORDER BY i.item_id ASC"#;

        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(InventoryItemWithGoods {
                item_id: row.get("item_id"),
                goods_id: row.get("goods_id"),
                material_code: row.get("material_code"),
                goods_name: row.get("goods_name"),
                description: row.get("description"),
                price: row.get("price"),
                volumn_l: row.get("volumn_l"),
                mass_g: row.get("mass_g"),
                mass_base: row.get("mass_base"),
                volumn_base: row.get("volumn_base"),
                quantity: row.get("quantity"),
                expired_date: row.get("expired_date"),
            });
        }

        Ok(results)
    }

    pub async fn insert(&self, request: CreateInventoryRequest) -> Result<(InventoryItemWithGoods, bool), sqlx::Error> {
        // Determine goods_id to use
        let goods_id = if let Some(id) = request.goods_id {
            // Use utility function to verify goods exists
            let exists = crate::utils::database::exists_by_id(&self.pool, "goods", "goods_id", id).await?;
            
            if !exists {
                return Err(sqlx::Error::RowNotFound);
            }
            id
        } else if let Some(material_code) = &request.material_code {
            // Use utility function to find goods by material_code
            let goods_id = crate::utils::database::get_id_by_string(
                &self.pool, 
                "goods", 
                "goods_id", 
                "material_code", 
                material_code
            ).await?;
            
            if let Some(id) = goods_id {
                id
            } else {
                return Err(sqlx::Error::RowNotFound);
            }
        } else if request.goods_name.is_some() && request.price.is_some() && 
                  request.volumn_l.is_some() && request.mass_g.is_some() {
            // Create new goods if all required fields are provided
            let material_code = request.material_code.clone()
                .ok_or_else(|| sqlx::Error::ColumnNotFound("material_code is required for new goods".into()))?;
            
            let new_goods = sqlx::query_as::<_, (i32,)>(
                r#"
                INSERT INTO goods (material_code, goods_name, description, price, volumn_l, mass_g, mass_base, volumn_base)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING goods_id
                "#
            )
            .bind(&material_code)
            .bind(request.goods_name.as_ref().unwrap())
            .bind(&request.description)
            .bind(request.price.unwrap())
            .bind(request.volumn_l.unwrap())
            .bind(request.mass_g.unwrap())
            .bind(request.mass_base.unwrap_or(0))
            .bind(request.volumn_base.unwrap_or(0))
            .fetch_one(&self.pool)
            .await?;
            
            new_goods.0
        } else {
            return Err(sqlx::Error::ColumnNotFound(
                "Either goods_id, material_code, or complete goods information is required".into()
            ));
        };

        // Check if inventory item with same goods_id and expired_date already exists
        let existing_item = if let Some(expired_date) = request.expired_date {
            // Log warning if creating inventory that's already expired
            if crate::utils::datetime::is_expired(&expired_date) {
                tracing::warn!("Creating inventory item that's already expired for goods_id: {}", goods_id);
            }
            
            sqlx::query_as::<_, InventoryItem>(
                "SELECT item_id, goods_id, quantity, expired_date FROM inventory WHERE goods_id = $1 AND expired_date = $2"
            )
            .bind(goods_id)
            .bind(expired_date)
            .fetch_optional(&self.pool)
            .await?
        } else {
            // Check for items with NULL expired_date
            sqlx::query_as::<_, InventoryItem>(
                "SELECT item_id, goods_id, quantity, expired_date FROM inventory WHERE goods_id = $1 AND expired_date IS NULL"
            )
            .bind(goods_id)
            .fetch_optional(&self.pool)
            .await?
        };

        if let Some(existing) = existing_item {
            // Return the existing inventory item with goods details and flag as existing
            let existing_with_goods = self.get_by_item_id(existing.item_id).await?;
            return Ok((existing_with_goods, false)); // false = not newly created
        }

        // Insert new inventory item if no duplicate found
        let new_item = sqlx::query_as::<_, InventoryItem>(
            r#"
            INSERT INTO inventory (goods_id, quantity, expired_date)
            VALUES ($1, $2, $3)
            RETURNING item_id, goods_id, quantity, expired_date
            "#
        )
        .bind(goods_id)
        .bind(request.quantity)
        .bind(request.expired_date)
        .fetch_one(&self.pool)
        .await?;

        // Get the full inventory item with goods details
        let new_with_goods = self.get_by_item_id(new_item.item_id).await?;
        Ok((new_with_goods, true)) // true = newly created
    }

    pub async fn get_by_item_id(&self, item_id: i32) -> Result<InventoryItemWithGoods, sqlx::Error> {
        let query = r#"
            SELECT 
                i.item_id, i.goods_id, i.quantity, i.expired_date,
                g.material_code, g.goods_name, g.description, g.price, 
                g.volumn_l, g.mass_g, g.mass_base, g.volumn_base
            FROM inventory i
            INNER JOIN goods g ON i.goods_id = g.goods_id
            WHERE i.item_id = $1"#;

        let row = sqlx::query(query)
            .bind(item_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(InventoryItemWithGoods {
            item_id: row.get("item_id"),
            goods_id: row.get("goods_id"),
            material_code: row.get("material_code"),
            goods_name: row.get("goods_name"),
            description: row.get("description"),
            price: row.get("price"),
            volumn_l: row.get("volumn_l"),
            mass_g: row.get("mass_g"),
            mass_base: row.get("mass_base"),
            volumn_base: row.get("volumn_base"),
            quantity: row.get("quantity"),
            expired_date: row.get("expired_date"),
        })
    }

    pub async fn update(&self, params: InventorySearchParams, update_request: UpdateInventoryRequest) -> Result<Vec<InventoryItemWithGoods>, sqlx::Error> {
        // First, find inventory items to update
        let items_to_update = self.search(params).await?;
        
        if items_to_update.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();

        for item in items_to_update {
            // Start transaction for each item to handle both goods and inventory updates
            let mut tx = self.pool.begin().await?;

            // Update goods if goods-related fields are provided
            if update_request.material_code.is_some() || 
               update_request.goods_name.is_some() ||
               update_request.description.is_some() ||
               update_request.price.is_some() ||
               update_request.volumn_l.is_some() ||
               update_request.mass_g.is_some() ||
               update_request.mass_base.is_some() ||
               update_request.volumn_base.is_some() {
                
                sqlx::query(
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
                    "#
                )
                .bind(item.goods_id)
                .bind(&update_request.material_code)
                .bind(&update_request.goods_name)
                .bind(&update_request.description)
                .bind(&update_request.price)
                .bind(&update_request.volumn_l)
                .bind(&update_request.mass_g)
                .bind(&update_request.mass_base)
                .bind(&update_request.volumn_base)
                .execute(&mut *tx)
                .await?;
            }

            // Update inventory if inventory-related fields are provided
            if update_request.quantity.is_some() || update_request.expired_date.is_some() {
                sqlx::query(
                    r#"
                    UPDATE inventory 
                    SET 
                        quantity = COALESCE($2, quantity),
                        expired_date = COALESCE($3, expired_date)
                    WHERE item_id = $1
                    "#
                )
                .bind(item.item_id)
                .bind(&update_request.quantity)
                .bind(&update_request.expired_date)
                .execute(&mut *tx)
                .await?;
            }

            tx.commit().await?;

            // Get updated item
            let updated_item = self.get_by_item_id(item.item_id).await?;
            updated_items.push(updated_item);
        }

        Ok(updated_items)
    }

    pub async fn delete(&self, params: InventorySearchParams) -> Result<Vec<i32>, sqlx::Error> {
        // First, find inventory items to delete
        let items_to_delete = self.search(params).await?;
        
        if items_to_delete.is_empty() {
            return Ok(Vec::new());
        }

        let mut deleted_ids = Vec::new();

        // Delete each found inventory item
        for item in items_to_delete {
            sqlx::query("DELETE FROM inventory WHERE item_id = $1")
                .bind(item.item_id)
                .execute(&self.pool)
                .await?;
            
            deleted_ids.push(item.item_id);
        }

        Ok(deleted_ids)
    }
}