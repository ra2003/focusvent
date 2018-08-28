use std::io::Read;
use diesel;
use diesel::prelude::*;
use models::db_connection::*;
use models::product_price::ProductPrice;
use models::price::Price;
use schema::product_prices::dsl::*;
use schema::prices::dsl::*;
use schema::products;
use schema::products::dsl::*;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub stock: f64
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name="products"]
pub struct NewProduct {
    pub name: String,
    pub description: Option<String>,
}

impl Product {
    pub fn list(limit: i64, offset: i64) -> Result<Vec<(Product, Option<(ProductPrice, Price)>)>, diesel::result::Error> {
        use schema::products::dsl::*;

        let connection = establish_connection();
        products
            .left_join(product_prices.inner_join(prices))
            .limit(limit)
            .offset(offset)
            .load::<(Product, Option<(ProductPrice, Price)>)>(&connection)
    }

    pub fn create(product: NewProduct) -> Result<Product, diesel::result::Error> {
        let connection = establish_connection();

        diesel::insert_into(products::table)
            .values(&product)
            .get_result(&connection)
    }

    pub fn update(param_id: i32, product: Product) -> Result<Product, diesel::result::Error> {
        use schema::products::dsl::name;
        let connection = establish_connection();

        diesel::update(products.find(param_id))
            .set((name.eq(product.name),
                  description.eq(product.description)))
            .get_result::<Product>(&connection)
    }

    pub fn delete(param_id: i32) -> Result<usize, diesel::result::Error> {
        let connection = establish_connection();

        diesel::delete(products.find(param_id))
            .execute(&connection)
    }
}

from_data!(Product);
from_data!(NewProduct);