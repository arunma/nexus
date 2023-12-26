use anyhow::{bail, Context};
use axum_odbc::odbc::buffers::TextRowSet;
use axum_odbc::odbc::{Cursor, IntoParameter};
use axum_odbc::ODBCConnectionManager;
use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::domain::RegisterUserDto;

#[derive(Clone)]
pub struct UserRepository {
    pool: ODBCConnectionManager,
}

impl UserRepository {
    pub fn new(db: ODBCConnectionManager) -> Self {
        Self { pool: db }
    }
    pub async fn create_user(&self, register_user: RegisterUserDto, hashed_password: String) -> anyhow::Result<String> {
        let conn = self
            .pool
            .aquire()
            .await
            .context("Unable to get a connection from the ODBCConnectionManager")?;

        let id = Uuid::new_v4().to_string();
        let mut prepared =
            conn.prepare("INSERT INTO nexus_db.public.nexus_users  (id, name, email, password) VALUES (?,?,?,?)")?;
        let params = (
            &id.into_parameter(),
            &register_user.name.clone().into_parameter(),
            &register_user.email.into_parameter(),
            &hashed_password.into_parameter(),
        );
        prepared.execute(params)?;

        info!("User {} successfully inserted", &register_user.name);

        Ok(format!(
            "User {user} successfully registered",
            user = &register_user.name
        ))
    }

    pub async fn does_user_exist(&self, email: &str) -> anyhow::Result<bool> {
        let conn = self
            .pool
            .aquire()
            .await
            .context("Unable to get a connection from the ODBCConnectionManager")?;

        let mut cursor = conn
            .execute(
                "SELECT EXISTS(SELECT 1 FROM nexus_db.public.nexus_users WHERE email = ?)",
                &email.into_parameter(),
            )?
            .expect("select statement must create a cursor");
        if let Some(mut row) = cursor.next_row()? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<UserEntity> {
        let conn = self
            .pool
            .aquire()
            .await
            .context("Unable to get a connection from the ODBCConnectionManager")?;

        let mut cursor = conn
            .execute(
                "SELECT id, name, email, password FROM nexus_db.public.nexus_users WHERE email = ?",
                &email.into_parameter(),
            )?
            .expect("select statement must create a cursor");
        let mut buffers = TextRowSet::for_cursor(1, &mut cursor, Some(4096))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

        if let Some(batch) = row_set_cursor.fetch()? {
            let user = UserEntity {
                id: batch
                    .at_as_str(0, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string(),
                name: batch
                    .at_as_str(1, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string()
                    .to_string(),
                email: batch
                    .at_as_str(2, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string()
                    .to_string(),
                password: batch
                    .at_as_str(3, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string()
                    .to_string(),
                created_at: None,
                updated_at: None,
            };

            info!("user = {:?}", user);
            Ok(user)
        } else {
            bail!(format!("Unable to fetch user with that email id : {}", email))
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> anyhow::Result<UserEntity> {
        let conn = self
            .pool
            .aquire()
            .await
            .context("Unable to get a connection from the ODBCConnectionManager")?;

        let mut cursor = conn
            .execute(
                "SELECT id, name, email FROM nexus_db.public.nexus_users WHERE id = ?",
                &user_id.into_parameter(),
            )?
            .expect("select statement must create a cursor");
        let mut buffers = TextRowSet::for_cursor(1, &mut cursor, Some(4096))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

        if let Some(batch) = row_set_cursor.fetch()? {
            let user = UserEntity {
                id: batch
                    .at_as_str(0, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string(),
                name: batch
                    .at_as_str(1, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string()
                    .to_string(),
                email: batch
                    .at_as_str(2, 0)
                    .unwrap()
                    .expect("Can't convert the userid to string")
                    .to_string()
                    .to_string(),
                password: "".to_string(),
                created_at: None,
                updated_at: None,
            };
            Ok(user)
        } else {
            bail!(format!("Unable to fetch user with that id : {}", user_id))
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
