// src/request.rs
use crate::tables::{
    GoodsSearchParams, CreateGoodRequest, UpdateGoodRequest,
    InventorySearchParams, CreateInventoryRequest, UpdateInventoryRequest
};
use crate::utils::validation::*;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsQueryParams {
    pub goods_id: Option<String>,
    pub material_code: Option<String>,
    pub goods_name: Option<String>,
    pub price: Option<String>,
    pub volumn_l: Option<String>,
    pub mass_g: Option<String>,
    pub min_volumn_l: Option<String>,
    pub max_volumn_l: Option<String>,
    pub min_mass_g: Option<String>,
    pub max_mass_g: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InventoryQueryParams {
    // Inventory specific params
    pub item_id: Option<String>,
    pub quantity: Option<String>,
    pub min_quantity: Option<String>,
    pub max_quantity: Option<String>,
    pub expired_date: Option<String>,
    pub min_expired_date: Option<String>,
    pub max_expired_date: Option<String>,
    
    // Goods params (inherited)
    pub goods_id: Option<String>,
    pub material_code: Option<String>,
    pub goods_name: Option<String>,
    pub price: Option<String>,
    pub volumn_l: Option<String>,
    pub mass_g: Option<String>,
    pub min_volumn_l: Option<String>,
    pub max_volumn_l: Option<String>,
    pub min_mass_g: Option<String>,
    pub max_mass_g: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
}

impl GoodsQueryParams {
    pub fn validate_and_parse(self) -> Result<GoodsSearchParams, String> {
        let mut search_params = GoodsSearchParams::new();

        if let Some(goods_id_str) = self.goods_id {
            search_params.goods_id = Some(parse_safe_integer(&goods_id_str, "goods_id")?);
        }

        if let Some(material_code) = self.material_code {
            validate_safe_string(&material_code, "material_code")?;
            search_params.material_code = Some(material_code);
        }

        if let Some(goods_name) = self.goods_name {
            validate_safe_string(&goods_name, "goods_name")?;
            search_params.goods_name = Some(goods_name);
        }

        if let Some(price_str) = self.price {
            search_params.price = Some(parse_safe_decimal(&price_str, "price")?);
        }

        if let Some(volumn_l_str) = self.volumn_l {
            search_params.volumn_l = Some(parse_safe_decimal(&volumn_l_str, "volumn_l")?);
        }

        if let Some(mass_g_str) = self.mass_g {
            search_params.mass_g = Some(parse_safe_decimal(&mass_g_str, "mass_g")?);
        }

        if let Some(min_volumn_l_str) = self.min_volumn_l {
            search_params.min_volumn_l = Some(parse_safe_decimal(&min_volumn_l_str, "min_volumn_l")?);
        }

        if let Some(max_volumn_l_str) = self.max_volumn_l {
            search_params.max_volumn_l = Some(parse_safe_decimal(&max_volumn_l_str, "max_volumn_l")?);
        }

        if let Some(min_mass_g_str) = self.min_mass_g {
            search_params.min_mass_g = Some(parse_safe_decimal(&min_mass_g_str, "min_mass_g")?);
        }

        if let Some(max_mass_g_str) = self.max_mass_g {
            search_params.max_mass_g = Some(parse_safe_decimal(&max_mass_g_str, "max_mass_g")?);
        }

        if let Some(min_price_str) = self.min_price {
            search_params.min_price = Some(parse_safe_decimal(&min_price_str, "min_price")?);
        }

        if let Some(max_price_str) = self.max_price {
            search_params.max_price = Some(parse_safe_decimal(&max_price_str, "max_price")?);
        }

        Ok(search_params)
    }

    pub fn has_any_params(&self) -> bool {
        self.goods_id.is_some()
            || self.material_code.is_some()
            || self.goods_name.is_some()
            || self.price.is_some()
            || self.volumn_l.is_some()
            || self.mass_g.is_some()
            || self.min_volumn_l.is_some()
            || self.max_volumn_l.is_some()
            || self.min_mass_g.is_some()
            || self.max_mass_g.is_some()
            || self.min_price.is_some()
            || self.max_price.is_some()
    }
}

impl InventoryQueryParams {
    pub fn validate_and_parse(self) -> Result<InventorySearchParams, String> {
        let mut search_params = InventorySearchParams::new();

        // Parse inventory-specific params
        if let Some(item_id_str) = self.item_id {
            search_params.item_id = Some(parse_safe_integer(&item_id_str, "item_id")?);
        }

        if let Some(quantity_str) = self.quantity {
            search_params.quantity = Some(parse_safe_integer(&quantity_str, "quantity")?);
        }

        if let Some(min_quantity_str) = self.min_quantity {
            search_params.min_quantity = Some(parse_safe_integer(&min_quantity_str, "min_quantity")?);
        }

        if let Some(max_quantity_str) = self.max_quantity {
            search_params.max_quantity = Some(parse_safe_integer(&max_quantity_str, "max_quantity")?);
        }

        if let Some(expired_date_str) = self.expired_date {
            search_params.expired_date = Some(parse_safe_datetime(&expired_date_str, "expired_date")?);
        }

        if let Some(min_expired_date_str) = self.min_expired_date {
            search_params.min_expired_date = Some(parse_safe_datetime(&min_expired_date_str, "min_expired_date")?);
        }

        if let Some(max_expired_date_str) = self.max_expired_date {
            search_params.max_expired_date = Some(parse_safe_datetime(&max_expired_date_str, "max_expired_date")?);
        }

        // Parse goods params using existing validation
        let goods_query_params = GoodsQueryParams {
            goods_id: self.goods_id,
            material_code: self.material_code,
            goods_name: self.goods_name,
            price: self.price,
            volumn_l: self.volumn_l,
            mass_g: self.mass_g,
            min_volumn_l: self.min_volumn_l,
            max_volumn_l: self.max_volumn_l,
            min_mass_g: self.min_mass_g,
            max_mass_g: self.max_mass_g,
            min_price: self.min_price,
            max_price: self.max_price,
        };

        search_params.goods_params = goods_query_params.validate_and_parse()?;

        Ok(search_params)
    }

    pub fn has_any_params(&self) -> bool {
        self.item_id.is_some()
            || self.quantity.is_some()
            || self.min_quantity.is_some()
            || self.max_quantity.is_some()
            || self.expired_date.is_some()
            || self.min_expired_date.is_some()
            || self.max_expired_date.is_some()
            || self.goods_id.is_some()
            || self.material_code.is_some()
            || self.goods_name.is_some()
            || self.price.is_some()
            || self.volumn_l.is_some()
            || self.mass_g.is_some()
            || self.min_volumn_l.is_some()
            || self.max_volumn_l.is_some()
            || self.min_mass_g.is_some()
            || self.max_mass_g.is_some()
            || self.min_price.is_some()
            || self.max_price.is_some()
    }
}

impl CreateGoodRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_safe_string(&self.material_code, "material_code")?;
        validate_safe_string(&self.goods_name, "goods_name")?;

        if let Some(desc) = &self.description {
            for (i, item) in desc.iter().enumerate() {
                validate_safe_string(item, &format!("description[{}]", i))?;
            }
        }

        if self.price < rust_decimal::Decimal::ZERO {
            return Err("Price cannot be negative".to_string());
        }
        if self.volumn_l <= rust_decimal::Decimal::ZERO {
            return Err("Volume must be positive".to_string());
        }
        if self.mass_g <= rust_decimal::Decimal::ZERO {
            return Err("Mass must be positive".to_string());
        }

        Ok(())
    }
}

impl UpdateGoodRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.material_code.is_none() 
            && self.goods_name.is_none() 
            && self.description.is_none() 
            && self.price.is_none() 
            && self.volumn_l.is_none() 
            && self.mass_g.is_none() 
            && self.mass_base.is_none() 
            && self.volumn_base.is_none() {
            return Err("At least one field must be provided for update".to_string());
        }

        if let Some(material_code) = &self.material_code {
            validate_safe_string(material_code, "material_code")?;
        }

        if let Some(goods_name) = &self.goods_name {
            validate_safe_string(goods_name, "goods_name")?;
        }

        if let Some(desc) = &self.description {
            for (i, item) in desc.iter().enumerate() {
                validate_safe_string(item, &format!("description[{}]", i))?;
            }
        }

        if let Some(price) = self.price {
            if price < rust_decimal::Decimal::ZERO {
                return Err("Price cannot be negative".to_string());
            }
        }
        if let Some(volumn_l) = self.volumn_l {
            if volumn_l <= rust_decimal::Decimal::ZERO {
                return Err("Volume must be positive".to_string());
            }
        }
        if let Some(mass_g) = self.mass_g {
            if mass_g <= rust_decimal::Decimal::ZERO {
                return Err("Mass must be positive".to_string());
            }
        }

        Ok(())
    }
}

impl CreateInventoryRequest {
    pub fn validate(&self) -> Result<(), String> {
        // Validate that we have some way to identify or create goods
        if self.goods_id.is_none() && self.material_code.is_none() {
            // If no goods_id or material_code, we need complete goods information
            if self.goods_name.is_none() || self.price.is_none() || 
               self.volumn_l.is_none() || self.mass_g.is_none() {
                return Err("Either goods_id, material_code, or complete goods information (goods_name, price, volumn_l, mass_g) is required".to_string());
            }
        }

        // Validate quantity
        if self.quantity < 0 {
            return Err("Quantity cannot be negative".to_string());
        }

        // Validate strings if provided
        if let Some(material_code) = &self.material_code {
            validate_safe_string(material_code, "material_code")?;
        }

        if let Some(goods_name) = &self.goods_name {
            validate_safe_string(goods_name, "goods_name")?;
        }

        if let Some(desc) = &self.description {
            for (i, item) in desc.iter().enumerate() {
                validate_safe_string(item, &format!("description[{}]", i))?;
            }
        }

        // Validate numeric values if provided
        if let Some(price) = self.price {
            if price < rust_decimal::Decimal::ZERO {
                return Err("Price cannot be negative".to_string());
            }
        }
        if let Some(volumn_l) = self.volumn_l {
            if volumn_l <= rust_decimal::Decimal::ZERO {
                return Err("Volume must be positive".to_string());
            }
        }
        if let Some(mass_g) = self.mass_g {
            if mass_g <= rust_decimal::Decimal::ZERO {
                return Err("Mass must be positive".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateInventoryRequest {
    pub fn validate(&self) -> Result<(), String> {
        // Check if at least one field is provided
        if self.material_code.is_none() 
            && self.goods_name.is_none() 
            && self.description.is_none() 
            && self.price.is_none() 
            && self.volumn_l.is_none() 
            && self.mass_g.is_none() 
            && self.mass_base.is_none() 
            && self.volumn_base.is_none()
            && self.quantity.is_none()
            && self.expired_date.is_none() {
            return Err("At least one field must be provided for update".to_string());
        }

        // Validate goods fields if provided
        if let Some(material_code) = &self.material_code {
            validate_safe_string(material_code, "material_code")?;
        }

        if let Some(goods_name) = &self.goods_name {
            validate_safe_string(goods_name, "goods_name")?;
        }

        if let Some(desc) = &self.description {
            for (i, item) in desc.iter().enumerate() {
                validate_safe_string(item, &format!("description[{}]", i))?;
            }
        }

        if let Some(price) = self.price {
            if price < rust_decimal::Decimal::ZERO {
                return Err("Price cannot be negative".to_string());
            }
        }
        if let Some(volumn_l) = self.volumn_l {
            if volumn_l <= rust_decimal::Decimal::ZERO {
                return Err("Volume must be positive".to_string());
            }
        }
        if let Some(mass_g) = self.mass_g {
            if mass_g <= rust_decimal::Decimal::ZERO {
                return Err("Mass must be positive".to_string());
            }
        }

        // Validate inventory fields if provided
        if let Some(quantity) = self.quantity {
            if quantity < 0 {
                return Err("Quantity cannot be negative".to_string());
            }
        }

        Ok(())
    }
}

pub fn extract_goods_query_params(query: Query<HashMap<String, String>>) -> GoodsQueryParams {
    let params = query.0;
    
    GoodsQueryParams {
        goods_id: params.get("goods_id").cloned(),
        material_code: params.get("material_code").cloned(),
        goods_name: params.get("goods_name").cloned(),
        price: params.get("price").cloned(),
        volumn_l: params.get("volumn_l").cloned(),
        mass_g: params.get("mass_g").cloned(),
        min_volumn_l: params.get("min_volumn_l").cloned(),
        max_volumn_l: params.get("max_volumn_l").cloned(),
        min_mass_g: params.get("min_mass_g").cloned(),
        max_mass_g: params.get("max_mass_g").cloned(),
        min_price: params.get("min_price").cloned(),
        max_price: params.get("max_price").cloned(),
    }
}

pub fn extract_inventory_query_params(query: Query<HashMap<String, String>>) -> InventoryQueryParams {
    let params = query.0;
    
    InventoryQueryParams {
        // Inventory specific params
        item_id: params.get("item_id").cloned(),
        quantity: params.get("quantity").cloned(),
        min_quantity: params.get("min_quantity").cloned(),
        max_quantity: params.get("max_quantity").cloned(),
        expired_date: params.get("expired_date").cloned(),
        min_expired_date: params.get("min_expired_date").cloned(),
        max_expired_date: params.get("max_expired_date").cloned(),
        
        // Goods params
        goods_id: params.get("goods_id").cloned(),
        material_code: params.get("material_code").cloned(),
        goods_name: params.get("goods_name").cloned(),
        price: params.get("price").cloned(),
        volumn_l: params.get("volumn_l").cloned(),
        mass_g: params.get("mass_g").cloned(),
        min_volumn_l: params.get("min_volumn_l").cloned(),
        max_volumn_l: params.get("max_volumn_l").cloned(),
        min_mass_g: params.get("min_mass_g").cloned(),
        max_mass_g: params.get("max_mass_g").cloned(),
        min_price: params.get("min_price").cloned(),
        max_price: params.get("max_price").cloned(),
    }
}