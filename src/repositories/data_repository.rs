use std::collections::HashMap;

use anyhow::{anyhow, Context};
use axum_odbc::odbc::buffers::TextRowSet;
use axum_odbc::odbc::{Cursor, ResultSetMetadata};
use axum_odbc::ODBCConnectionManager;
use tera::Tera;
use tracing::{error, info};

use crate::domain::req_res::DataResponse;
use crate::errors::ApiError;
use crate::errors::ApiError::InternalServerErrorWithContext;

pub const BATCH_SIZE: usize = 1000;

#[derive(Clone)]
pub struct DataRepository {
    pool: ODBCConnectionManager,
    tera: Tera,
}

impl DataRepository {
    pub fn new(pool: ODBCConnectionManager, tera: Tera) -> Self {
        Self { pool, tera }
    }

    pub async fn extract_results(&self, params: HashMap<String, String>) -> Result<DataResponse, ApiError> {
        let conn = self
            .pool
            .aquire()
            .await
            .context("Unable to get a connection from the ODBCConnectionManager")?;
        //TOOD - Figure out how to avoid sql injection here.
        let sql = self.construct_sql(&params)?;

        //FIXME - Convert parameters to a HashMap
        info!("Executing query: {}", sql);
        info!("Template parameters: {:?}", &params);

        let rows_result = match conn.execute(&sql, ()) {
            Err(e) => {
                error!("Error while executing query: {}", e);
                return Err(InternalServerErrorWithContext(format!(
                    "StatementExecutionError: {}",
                    e.to_string()
                )));
            }
            Ok(None) => {
                error!("No results returned");
                return Err(InternalServerErrorWithContext("NoResultsError".into()));
            }
            Ok(Some(mut cursor)) => {
                let col_names = cursor.column_names()?.map(|s| s.unwrap()).collect::<Vec<String>>(); //The unwrap needs further investigation. Why would the metadata column return an error at a column name level?

                let mut buffers = TextRowSet::for_cursor(BATCH_SIZE, &mut cursor, None)?;

                let mut rows_cursor = cursor.bind_buffer(&mut buffers)?;

                let mut rows = Vec::new();

                while let Some(rowset) = rows_cursor.fetch()? {
                    for rowi in 0..rowset.num_rows() {
                        let mut row_map = HashMap::new();
                        for (coli, col_name) in col_names.iter().enumerate() {
                            let col_value = rowset.at(coli, rowi);
                            match col_value {
                                None => {
                                    row_map.insert(col_name.to_string(), "".to_string());
                                }
                                Some(cval) => {
                                    row_map
                                        .insert(col_name.to_string(), std::str::from_utf8(cval).unwrap().to_string());
                                }
                            }
                        }
                        rows.push(row_map);
                    }
                }

                Ok(rows)
            }
        };

        rows_result
    }

    fn construct_sql(&self, params: &HashMap<String, String>) -> Result<String, ApiError> {
        let mut context = tera::Context::new();

        for (key, value) in params {
            context.insert(key, &value);
        }
        let result = self.tera.render("sql", &context).map_err(|e| {
            error!("Error while rendering template: {}", e);
            anyhow!(format!("SqlTemplatingError {}", e.to_string()))
        })?;

        Ok(result)
    }
}
