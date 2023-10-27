use std::sync::Arc;

use tracing::error;

use crate::domain::{LoginUserDto, RegisterUserDto, UserDto};
use crate::errors::{ApiError, ApiResult};
use crate::repositories::user_repository::UserRepository;
use crate::services::security_service::SecurityService;
use crate::services::token_service::TokenService;

#[derive(Clone)]
pub struct UserService {
    pub user_repository: Arc<UserRepository>,
    pub security_service: Arc<SecurityService>,
    pub token_service: Arc<TokenService>,
}

impl UserService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        security_service: Arc<SecurityService>,
        token_service: Arc<TokenService>,
    ) -> Self {
        Self {
            user_repository,
            security_service,
            token_service,
        }
    }
    pub async fn register_user_handler(&self, register_user: RegisterUserDto) -> ApiResult<UserDto> {
        let existing_user = self.user_repository.does_user_exist(&register_user.email).await;

        if let Err(_e) = existing_user {
            error!("user with email {} already exists", &register_user.email);
            return Err(ApiError::ObjectConflict(
                "User with that email id already exists".to_string(),
            ));
        }

        let hashed_password = self.security_service.hash_password(&register_user.password)?;

        let created_user = self.user_repository.create_user(register_user, hashed_password).await?;

        let user = created_user.into();

        Ok(user)
    }

    pub async fn login_user_handler(&self, login_user: LoginUserDto) -> ApiResult<UserDto> {
        let user = self.user_repository.get_user_by_email(&login_user.email).await;

        if let Err(_e) = user {
            error!("User with email does not exist: {}", &login_user.email);
            return Err(ApiError::NotFound("User with that email does not exist".to_string()));
        }

        let user = user.unwrap();
        let is_valid = self
            .security_service
            .verify_password(&user.password, &login_user.password)?;

        if !is_valid {
            return Err(ApiError::InvalidLoginAttempt);
        }

        let access_token = self.token_service.generate_access_token(user.id.clone()).await?;
        let refresh_token = self.token_service.generate_refresh_token(user.id.clone()).await?;

        let mut user_dto: UserDto = user.into();
        user_dto.access_token = Some(access_token);
        user_dto.refresh_token = Some(refresh_token);

        Ok(user_dto)
    }

    /*    async fn save_token_to_redis(
        &self,
        redis_client: &Client,
        token_details: &TokenDetails,
        max_age: i64,
    ) -> Result<(), (StatusCode, Json<Value>)> {
        let mut redis = redis_client.get_async_connection().await.map_err(|e| {
            let response = json!({
                "status": "failure",
                "message": "Unable to establish connection with redis"
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        })?;

        redis
            .set_ex(
                token_details.token_uuid.to_string(),
                token_details.user_id.to_string(),
                (max_age * 60) as usize,
            )
            .await
            .map_err(|e| {
                let response = json!({
                    "status": "failure",
                    "message": format!("Unable to write to redis {}", e)
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            })?;

        Ok(())
    }*/

    /*
    pub async fn refresh_access_token_handler(&self,
                                              cookie_jar: CookieJar,
                                              State(app_state): State<Arc<AppState>>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        //Get token from cookie - parse the token
        let refresh_token = cookie_jar
            .get("refresh_token")
            .map(|cookie| cookie.value().to_string())
            .ok_or_else(|| {
                let response = json!({
                "status": "failure",
                "message": "Could not retrieve token from cookie jar"
            });
                (StatusCode::FORBIDDEN, Json(response))
            })?;

        //Verify token validity
        let token_details = verify_and_decode_jwt_token(&refresh_token, &app_state.config.refresh_token_public_key)
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": "Could not validate and parse jwt token"
            });
                return (StatusCode::UNAUTHORIZED, Json(response));
            })?;

        //Look up redis and confirm if the incoming token belongs to the userid/subject stated
        let mut redis = app_state.redis.get_async_connection().await.map_err(|e| {
            let response = json!({
            "status": "failure",
            "message": "Unable to establish connection with redis"
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        })?;

        let user_id = redis
            .get::<_, String>(token_details.token_uuid.to_string())
            .await
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": "Cannot find token in redis. It's invalid or expired"
            });
                (StatusCode::UNAUTHORIZED, Json(response))
            })?;

        let user_id_uuid = Uuid::parse_str(&user_id).map_err(|e| {
            let response = json!({
            "status": "failure",
            "message": "Unable to parse userid to Uuid"
        });
            (StatusCode::UNAUTHORIZED, Json(response))
        })?;

        //Validate user against the database
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id_uuid)
            .fetch_optional(&app_state.db)
            .await
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": format!("Cannot run query against database {}", e)
            });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            })?
            .ok_or_else(|| {
                let response = json!({
                "status": "failure",
                "message": "Cannot find user in database"
            });
                (StatusCode::UNAUTHORIZED, Json(response))
            })?;

        //create new access token details
        let access_token_details = generate_token(
            user.id,
            app_state.config.access_token_max_age,
            &app_state.config.access_token_private_key,
        )?;

        //Save access token to redis
        save_token_to_redis(
            &app_state.redis,
            &access_token_details,
            app_state.config.access_token_max_age,
        )
            .await?;

        //Construct access cookie
        let access_cookie = Cookie::build("access_token", access_token_details.clone().token.unwrap_or_default())
            .path("/")
            .max_age(Duration::minutes(app_state.config.access_token_max_age * 60))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();

        let logged_in_cookie = Cookie::build("logged_in", "true")
            .path("/")
            .max_age(Duration::minutes(app_state.config.access_token_max_age * 60))
            .same_site(SameSite::Lax)
            .http_only(false)
            .finish();
        //Set new cookie in the response
        let mut response = Response::new(
            json!({
            "status": "success",
            "access_token": access_token_details.token.unwrap()
        })
                .to_string(),
        );

        let mut headers = HeaderMap::new();
        headers.append(SET_COOKIE, access_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, access_cookie.to_string().parse().unwrap());
        response.headers_mut().extend(headers);

        Ok(response)
    }

    pub async fn logout_handler(&self,
                                cookie_jar: CookieJar,
                                State(app_state): State<Arc<AppState>>,
                                Extension(jwt_auth): Extension<JWTAuthMiddleware>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        //Read refresh token from cookie/header
        let refresh_token = cookie_jar
            .get("refresh_token")
            .map(|cookie| cookie.value().to_string())
            .ok_or_else(|| {
                let response = json!({
                "status": "failure",
                "message": "Could not retrieve token from cookie jar"
            });
                (StatusCode::FORBIDDEN, Json(response))
            })?;

        //Parse token
        let refresh_token_details = verify_and_decode_jwt_token(&refresh_token, &app_state.config.refresh_token_public_key)
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": "Could not retrieve token from cookie jar"
            });
                (StatusCode::FORBIDDEN, Json(response))
            })?;

        let mut redis = app_state.redis.get_async_connection().await.map_err(|e| {
            let response = json!({
            "status": "failure",
            "message": "Unable to establish connection with redis"
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        })?;

        //Remove user from redis
        redis
            .del(&[
                refresh_token_details.token_uuid.to_string(),
                jwt_auth.access_token_uuid.to_string(),
            ])
            .await
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": format!("Unable to delete token from redis {}", e)
            });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            })?;
        //Set cookie expiry
        let access_cookie = Cookie::build("access_token", "")
            .path("/")
            .max_age(Duration::minutes(-1))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();

        let refresh_cookie = Cookie::build("refresh_token", "")
            .path("/")
            .max_age(Duration::minutes(-1))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();

        let login_cookie = Cookie::build("login_token", "")
            .path("/")
            .max_age(Duration::minutes(-1))
            .same_site(SameSite::Lax)
            .http_only(false)
            .finish();

        let mut headers = HeaderMap::new();
        headers.append(SET_COOKIE, access_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, refresh_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, login_cookie.to_string().parse().unwrap());

        let mut response = Response::new(json!({"status":"success"}).to_string());
        response.headers_mut().extend(headers);
        Ok(response)
    }

    pub async fn get_me_handler(&self,
                                Extension(jwt_auth): Extension<JWTAuthMiddleware>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let json_response = json!({
        "status":"success",
        "data": json!({
            "user": construct_filtered_user(&jwt_auth.user)
        })
    });

        Ok(Json(json_response))
    }

    fn construct_filtered_user(user: &User) -> FilteredUser {
        FilteredUser {
            id: user.id.to_string(),
            name: user.name.to_string(),
            email: user.email.to_string(),
            role: user.role.to_string(),
            photo: user.photo.to_string(),
            verified: user.verified,
            created_at: user.created_at.unwrap(),
            updated_at: user.created_at.unwrap(),
        }
    }*/
}