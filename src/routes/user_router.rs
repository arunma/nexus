use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde_json::json;
use tracing::info;

use crate::domain::requests::{LoginUserRequest, LoginUserResponse};
use crate::domain::{RegisterUserDto, UserDto};
use crate::errors::ApiResult;
use crate::service_register::ServiceRegister;
use crate::services::user_service::UserService;

pub struct UserRouter;

impl UserRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/healthz", get(UserRouter::healthz_handler))
            .route("/user/register", post(UserRouter::register_user_handler))
            .route("/user/login", post(UserRouter::login_user_handler))
            /*.route("/api/auth/refresh", get(UserRouter::refresh_access_token_handler))
            .route(
                "/api/auth/logout",
                get(UserRouter::logout_handler),//.route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
            )
            .route(
                "/api/users/me",
                get(UserRouter::get_me_handler),//.route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
            )*/
            .layer(Extension(service_register.user_service))
    }

    pub async fn healthz_handler() -> impl IntoResponse {
        let response = json!( {
            "status": "success",
            "message": "All good !"
        });

        Json(response)
    }

    pub async fn register_user_handler(
        Extension(user_service): Extension<Arc<UserService>>,
        Json(register_user): Json<RegisterUserDto>,
    ) -> ApiResult<Json<UserDto>> {
        info!("Registering new user {:?}", register_user);
        let user = user_service.register_user_handler(register_user).await?;
        Ok(Json(user))
    }

    pub async fn login_user_handler(
        Extension(user_service): Extension<Arc<UserService>>,
        Json(request): Json<LoginUserRequest>,
    ) -> ApiResult<Json<LoginUserResponse>> {
        info!("User logging in");
        let user = user_service.login_user_handler(request.user).await?;

        /*
        Does not apply for backend APIs
        //Prepare cookies
        let access_cookie = Cookie::build("access_token", user.access_token.unwrap_or_default())
            .path("/")
            .max_age(time::Duration::minutes(
                app_state.config.access_token_max_age.wrapping_mul(60),
            ))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();

        let refresh_cookie = Cookie::build("refresh_token", user.refresh_token.unwrap_or_default())
            .path("/")
            .max_age(Duration::minutes(
                app_state.config.refresh_token_max_age.wrapping_mul(60),
            ))
            .http_only(true)
            .same_site(SameSite::Lax)
            .finish();

        let logged_in_cookie = Cookie::build("logged_in", "true")
            .path("/")
            .max_age(time::Duration::minutes(
                app_state.config.access_token_max_age.wrapping_mul(60),
            ))
            .http_only(false)
            .same_site(SameSite::Lax)
            .finish();

        let mut headers = HeaderMap::new();
        headers.append(SET_COOKIE, access_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, refresh_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, logged_in_cookie.to_string().parse().unwrap());

        let mut response = Response::new(
            json!({
                "status": "success",
                "access_token": access_token_details.token.unwrap()
            })
            .to_string(),
        );

        response.headers_mut().extend(headers);

        Ok(response)*/

        Ok(Json(LoginUserResponse { user }))
    }
    /*
    pub async fn login_user_handler(
        State(app_state): State<Arc<AppState>>,
        Json(body): Json<LoginUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let config = app_state.config.clone();
        let access_max_age = config.access_token_max_age;
        let refresh_max_age = config.refresh_token_max_age;

        let user = sqlx::query_as!(User, "SELECT * from users where email = $1", &body.email)
            .fetch_optional(&app_state.db)
            .await
            .map_err(|e| {
                let response = json!({
                "status": "failure",
                "message": format!("Database error : {}", e)
            });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            })?
            .ok_or_else(|| {
                let response = json!({
                "status": "failure",
                "message": "Invalid email or password"
            });
                (StatusCode::BAD_REQUEST, Json(response))
            })?;

        let is_valid = match PasswordHash::new(&user.password) {
            Ok(password_hash) => Argon2::default()
                .verify_password(body.password.as_bytes(), &password_hash)
                .map_or(false, |_| true),
            Err(_) => false,
        };

        if !is_valid {
            let response = json!({
            "status": "failure",
            "message": "Invalid email or password"
        });
            return Err((StatusCode::BAD_REQUEST, Json(response)));
        }

        let access_token_details = generate_token(user.id, access_max_age, &app_state.config.access_token_private_key)?;
        let refresh_token_details = generate_token(user.id, refresh_max_age, &app_state.config.refresh_token_private_key)?;

        //Save to Redis
        user_repository.save_token_to_redis(
            &app_state.redis,
            &access_token_details,
            app_state.config.access_token_max_age,
        )
            .await?;
        save_token_to_redis(
            &app_state.redis,
            &refresh_token_details,
            app_state.config.refresh_token_max_age,
        )
            .await?;

        //Prepare cookies
        let access_cookie = Cookie::build("access_token", access_token_details.clone().token.unwrap_or_default())
            .path("/")
            .max_age(time::Duration::minutes(access_max_age.wrapping_mul(60)))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();

        let refresh_cookie = Cookie::build("refresh_token", refresh_token_details.clone().token.unwrap_or_default())
            .path("/")
            .max_age(Duration::minutes(refresh_max_age.wrapping_mul(60)))
            .http_only(true)
            .same_site(SameSite::Lax)
            .finish();

        let logged_in_cookie = Cookie::build("logged_in", "true")
            .path("/")
            .max_age(Duration::minutes(access_max_age.wrapping_mul(60)))
            .http_only(false)
            .same_site(SameSite::Lax)
            .finish();

        let mut headers = HeaderMap::new();
        headers.append(SET_COOKIE, access_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, refresh_cookie.to_string().parse().unwrap());
        headers.append(SET_COOKIE, logged_in_cookie.to_string().parse().unwrap());

        let mut response = Response::new(
            json!({
            "status": "success",
            "access_token": access_token_details.token.unwrap()
        })
                .to_string(),
        );

        response.headers_mut().extend(headers);

        Ok(response)
    }

    async fn save_token_to_redis(
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
    }

    fn generate_token(user_id: Uuid, max_age: i64, private_key: &str) -> Result<TokenDetails, (StatusCode, Json<Value>)> {
        generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
            let response = json!({
            "status": "failure",
            "message": format!("Unable to generate token: {}", e)
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        })
    }

    pub async fn refresh_access_token_handler(
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

    pub async fn logout_handler(
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

    pub async fn get_me_handler(
        Extension(jwt_auth): Extension<JWTAuthMiddleware>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let json_response = json!({
        "status":"success",
        "data": json!({
            "user": construct_filtered_user(&jwt_auth.user)
        })
    });

        Ok(Json(json_response))
    }*/
}
