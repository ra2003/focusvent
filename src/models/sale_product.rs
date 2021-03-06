use diesel;
use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::BoolExpressionMethods;
use crate::schema::sale_products;
use crate::models::money::Money;
use crate::models::db_connection::*;
use crate::models::item_calculation::ItemCalculation;

#[derive(AsChangeset, Insertable, Serialize, Deserialize, Clone, PartialEq, Identifiable, Associations,
         Queryable, Debug, FromForm, FromData, Responder)]
pub struct SaleProduct {
    pub id: i32,
    pub sale_id: i32,
    pub product_id: i32,
    pub tax: Money,
    pub amount: f64,
    pub price: Money,
    pub discount: Money,
    pub subtotal: Money,
    pub sub_total_without_discount: Money,
    pub discount_calculated: Money,
    pub taxes_calculated: Money,
    pub total: Money,
    pub observation: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug, FromForm, FromData)]
#[table_name="sale_products"]
pub struct NewSaleProduct {
    pub sale_id: Option<i32>,
    pub product_id: i32,
    pub tax: Money,
    pub amount: f64,
    pub price: Money,
    pub discount: Option<Money>,
    pub subtotal: Option<Money>,
    pub sub_total_without_discount: Option<Money>,
    pub discount_calculated: Option<Money>,
    pub taxes_calculated: Option<Money>,
    pub total: Option<Money>,
    pub observation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromForm, Responder)]
pub struct SearchSaleProduct {
    pub id: Option<i32>,
    pub sale_id: Option<i32>,
    pub product_id: Option<i32>,
    pub tax: Option<i32>,
    pub amount: Option<f64>,
    pub price: Option<i32>,
    pub discount: Option<i32>,
    pub subtotal: Option<f64>,
    pub sub_total_without_discount: Option<f64>,
    pub discount_calculated: Option<f64>,
    pub taxes_calculated: Option<f64>,
    pub total: Option<f64>,
    pub observation: Option<String>,
}

impl SaleProduct {
    pub fn batch_action(vec_sale_products: Vec<NewSaleProduct>, sale_id: i32) -> Result<bool, diesel::result::Error> {
        use crate::schema::sale_products::dsl;
        let connection = establish_connection();

        for mut new_sale_product in vec_sale_products {
            new_sale_product.sale_id = Some(sale_id);

            let result_sale_product = 
                dsl::sale_products
                    .filter(dsl::product_id.eq(new_sale_product.product_id).and(dsl::sale_id.eq(sale_id)))
                    .first::<SaleProduct>(&connection);

            if let Ok(edit_sale_product) = result_sale_product {
                let discount = (&new_sale_product).discount.clone();
                diesel::update(dsl::sale_products.find(edit_sale_product.id))
                    .set((dsl::tax.eq(&new_sale_product.tax),
                          dsl::amount.eq(&new_sale_product.amount),
                          dsl::price.eq(&new_sale_product.price),
                          dsl::discount.eq(discount.unwrap_or(Money(0))),
                          dsl::subtotal.eq(new_sale_product.calculate_sub_total()),
                          dsl::sub_total_without_discount.eq(new_sale_product.subtotal_without_discount()),
                          dsl::discount_calculated.eq(new_sale_product.calculate_discount()),
                          dsl::taxes_calculated.eq(new_sale_product.calculate_taxes()),
                          dsl::total.eq(new_sale_product.calculate_total())))
                    .get_result::<SaleProduct>(&connection)?;
            } else {
                diesel::insert_into(sale_products::table)
                    .values(&new_sale_product.with_calculations())
                    .get_result::<SaleProduct>(&connection)?;
            }
        }

        Ok(true)
    }
}

impl NewSaleProduct {

    pub fn with_calculations(&self) -> Self {
        let mut new_sale_product = self.clone();
        new_sale_product.discount = Some(self.calculate_discount());
        new_sale_product.subtotal = Some(self.calculate_sub_total());
        new_sale_product.sub_total_without_discount = Some(self.subtotal_without_discount());
        new_sale_product.discount_calculated = Some(self.calculate_discount());
        new_sale_product.taxes_calculated = Some(self.calculate_taxes());
        new_sale_product.total = Some(self.calculate_total());
        new_sale_product
    }

    pub fn to_item_calc_method(&self) -> ItemCalculation {
        let discount = (&self).discount.clone();
        ItemCalculation::new(&self.tax, &discount.unwrap_or(Money(0)), &self.price, self.amount)
    }

    pub fn calculate_total(&self) -> Money {
        let item_calc = self.to_item_calc_method();
        item_calc.calculate_total()
    }

    pub fn calculate_sub_total(&self) -> Money {
        let item_calc = self.to_item_calc_method();
        item_calc.subtotal()
    }

    pub fn subtotal_without_discount(&self) -> Money {
        let item_calc = self.to_item_calc_method();
        item_calc.subtotal_without_discount()
    }

    pub fn calculate_discount(&self) -> Money {
        let item_calc = self.to_item_calc_method();
        item_calc.calculate_discount()
    }

    pub fn calculate_taxes(&self) -> Money {
        let item_calc = self.to_item_calc_method();
        item_calc.calculate_taxes()
    }

}
