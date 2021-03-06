#[macro_export]
macro_rules! basic_model_actions {
    ($table_model:ident, $type_model:ident, $new_type_model:ty, $search_struct:ident) => {

        pub trait BasicModelActions {
            fn list(limit: i64, offset: i64, search: Option<crate::handlers::base::Search<$search_struct>>) ->
                Result<Vec<$type_model>, diesel::result::Error> {
                let connection = crate::models::db_connection::establish_connection();

                let query = $type_model::searching_records(search);

                query
                    .limit(limit)
                    .offset(offset)
                    .load::<$type_model>(&connection)
            }

            fn create(new_type_model: $new_type_model) -> Result<$type_model, diesel::result::Error> {
                let connection = crate::models::db_connection::establish_connection();

                diesel::insert_into($table_model::table)
                    .values(&new_type_model)
                    .get_result(&connection)
            }

            fn show(request_id: i32) -> Result<$type_model, diesel::result::Error> {
                use crate::schema::$table_model::dsl::*;
                let connection = crate::models::db_connection::establish_connection();

                $table_model
                    .find(request_id)
                    .get_result::<$type_model>(&connection)
            }

            fn update(param_id: i32, type_model: $type_model) -> Result<$type_model, diesel::result::Error> {
                use crate::schema::$table_model::dsl::*;
                let connection = crate::models::db_connection::establish_connection();

                diesel::update($table_model.find(param_id))
                    .set(&type_model)
                    .get_result::<$type_model>(&connection)
            }

            fn delete(param_id: i32) -> Result<usize, diesel::result::Error> {
                use crate::schema::$table_model::dsl::*;
                let connection = crate::models::db_connection::establish_connection();

                diesel::delete($table_model.find(param_id))
                    .execute(&connection)
            }
        }

        impl BasicModelActions for $type_model{ }
    };
}
