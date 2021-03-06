use diesel;
use diesel::prelude::*;
use diesel::sql_types;
use crate::handlers::base::Search;
use crate::models::db_connection::*;
use crate::models::product_price::ProductPrice;
use crate::models::product_price::EditableProductPrice;
use crate::models::product_price::FullProductPrice;
use crate::models::price::Price;
use crate::models::product_cost::ProductCost;
use crate::models::product_cost::EditableProductCost;
use crate::models::product_cost::FullProductCost;
use crate::models::cost::Cost;
use crate::models::supplier::Supplier;
use crate::schema;
use crate::schema::products;

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, FromForm, FromData, Responder)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub stock: Option<f64>,
    pub code: Option<String>
}

#[derive(Serialize, Deserialize, Debug, FromForm, Responder)]
pub struct SearchProduct {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub stock: Option<f64>,
    pub code: Option<String>
}

#[derive(Serialize, Deserialize, Insertable, Debug, Clone, Responder)]
#[table_name="products"]
pub struct NewProduct {
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone, FromData)]
pub struct FullNewProduct {
    product: NewProduct,
    prices: Vec<EditableProductPrice>,
    costs: Vec<EditableProductCost>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullProduct {
    pub product: Product,
    pub prices: Vec<FullProductPrice>,
    pub costs: Vec<FullProductCost>
}

type BoxedQuery<'a> = 
    diesel::query_builder::BoxedSelectStatement<'a, (sql_types::Integer,
                                                     sql_types::Text,
                                                     sql_types::Nullable<sql_types::Text>,
                                                     sql_types::Nullable<sql_types::Double>,
                                                     sql_types::Nullable<sql_types::Text>),
                                                     schema::products::table, diesel::pg::Pg>;

impl Product {
    pub fn list(limit: i64, offset: i64, search: Option<Search<SearchProduct>>) ->
        Result<Vec<Product>, diesel::result::Error> {
            let connection = establish_connection();
            
            let query = Self::searching_product(search);

            query
                .limit(limit)
                .offset(offset)
                .load(&connection)
    }

    pub fn show(request_id: i32) -> Result<FullProduct, diesel::result::Error> {
        use crate::schema::products::dsl::*;
        use crate::schema::product_prices;
        use crate::schema::product_costs;
        use crate::schema::suppliers;
        use crate::schema::costs;
        use crate::schema::prices;

        let connection = establish_connection();
        let mut full_product: FullProduct =
            FullProduct { 
                product: Product::blank_product(),
                prices: vec![],
                costs: vec![]
            };
        let db_product = products
            .find(request_id)
            .get_result::<Product>(&connection)?;

        let vec_product_costs = product_costs::dsl::product_costs
            .filter(product_costs::dsl::product_id.eq(db_product.id))
            .inner_join(costs::dsl::costs)
            .inner_join(suppliers::dsl::suppliers)
            .order(costs::name)
            .load::<(ProductCost, Cost, Supplier)>(&connection)?;

        let vec_product_prices = product_prices::dsl::product_prices
            .filter(product_prices::dsl::product_id.eq(db_product.id))
            .inner_join(prices::dsl::prices)
            .order(prices::name)
            .load::<(ProductPrice, Price)>(&connection)?;

        full_product.product = db_product;

        for (product_price, price) in vec_product_prices {
            full_product.prices.push(
                FullProductPrice {
                    price_id: price.id,
                    price: product_price.price,
                    name: price.name
                }
            );
        }

        for (product_cost, cost, supplier) in vec_product_costs {
            full_product.costs.push(
                FullProductCost {
                    cost_id: cost.id,
                    supplier_id: supplier.id,
                    cost: product_cost.cost,
                    name: cost.name
                }
            );
        }
        Ok(full_product)
    }

    pub fn create(full_new_product: FullNewProduct) -> Result<Product, diesel::result::Error> {
        let connection = establish_connection();

        let product: Result<Product, diesel::result::Error> = diesel::insert_into(products::table)
            .values(&full_new_product.product)
            .get_result(&connection);

        if let Ok(db_product) = &product {
            ProductPrice::batch_action(full_new_product.prices, db_product.id)?;
            ProductCost::batch_action(full_new_product.costs, db_product.id)?;
        }

        product
    }

    pub fn update(param_id: i32, full_product: FullNewProduct) -> Result<Product, diesel::result::Error> {
        use crate::schema::products::dsl::*;
        let connection = establish_connection();

        let product = diesel::update(products.find(param_id))
            .set((name.eq(full_product.product.name),
                  code.eq(full_product.product.code),
                  description.eq(full_product.product.description)))
            .get_result::<Product>(&connection);

        if let Ok(db_product) = &product {
            ProductPrice::batch_action(full_product.prices, db_product.id)?;
            ProductCost::batch_action(full_product.costs, db_product.id)?;
        }

        product
    }

    pub fn delete(param_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::products::dsl::*;
        let connection = establish_connection();

        diesel::delete(products.find(param_id))
            .execute(&connection)
    }

    fn searching_product<'a>(search: Option<Search<SearchProduct>>) -> BoxedQuery<'a> {
        use crate::schema::products::dsl::*;

        let mut query = schema::products::table.into_boxed::<diesel::pg::Pg>();

        if let Some(search_product) = search {
            let Search(product) = search_product;
            if let Some(product_name) = product.name {
                query = query.filter(name.like(product_name));
            }
            if let Some(product_id) = product.id {
                query = query.filter(id.eq(product_id));
            }
            if let Some(product_description) = product.description {
                query = query.filter(description.like(product_description));
            }
            if let Some(product_code) = product.code {
                query = query.filter(code.like(product_code));
            }
            if let Some(product_stock) = product.stock {
                query = query.filter(stock.eq(product_stock));
            }
        }

        query
    }

    fn blank_product() -> Product {
        Product {
            id: 0,
            code: None,
            name: "".to_string(),
            description: None,
            stock: None
        }
    }
}

use std::str::FromStr;
use serde_json;

impl FromStr for Product {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl FromStr for SearchProduct {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
