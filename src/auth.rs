use serde::Serialize;
use uuid::Uuid;

use crate::domain::UserDto;

#[derive(Debug, Clone, Serialize)]
pub struct JWTAuthMiddleware {
    pub user: UserDto,
    pub access_token_uuid: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

/*pub async fn auth<B>(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let access_token =
        cookie_jar
            .get("access_token")
            .map(|cookie| cookie.value().to_string())
            .or_else(|| {
                //fall back to getting bearer token
                let token =
                    req.headers()
                        .get(AUTHORIZATION)
                        .and_then(|h| h.to_str().ok())
                        .and_then(|auth_value| {
                            if auth_value.starts_with("Bearer ") {
                                Some(auth_value[7..].to_string())
                            } else {
                                None
                            }
                        });
                token
            });

    let access_token = access_token.ok_or_else(|| {
        let response = ErrorResponse {
            status: "failure",
            message: "You are not logged in. No token found in cookie or in header".to_string(),
        };

        (StatusCode::UNAUTHORIZED, Json(response))
    })?;

    let access_token_details =
        match verify_and_decode_jwt_token(&access_token, &app_state.config.access_token_public_key) {
            Ok(token_details) => token_details,
            Err(e) => {
                let response = ErrorResponse {
                    status: "failure",
                    message: format!("Unable to verify given token {}", e),
                };
                return Err((StatusCode::UNAUTHORIZED, Json(response)));
            }
        };

    let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string()).map_err(|_| {
        let error_response = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    let mut redis = app_state.redis.get_async_connection().await.map_err(|e| {
        let response = ErrorResponse {
            status: "failure",
            message: format!("Unable to connect to redis {}", e),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
    })?;

    let redis_user_id = redis
        .get::<_, String>(access_token_uuid.to_string())
        .await
        .map_err(|e| {
            let response = ErrorResponse {
                status: "failure",
                message: format!("Invalid token {}", e),
            };

            (StatusCode::UNAUTHORIZED, Json(response))
        })?;

    let user_id_uuid = Uuid::parse_str(&redis_user_id).map_err(|e| {
        let response = ErrorResponse {
            status: "failure",
            message: format!("Token is invalid {}", e),
        };

        (StatusCode::UNAUTHORIZED, Json(response))
    })?;

    println!("Access token userId {}", &access_token_details.user_id);
    println!("Redis userId {}", &user_id_uuid);

    let user = query_as!(UserEntity, "SELECT * from users where id = $1", user_id_uuid)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| {
            let response = ErrorResponse {
                status: "failure",
                message: format!("Unable to query database {}", e),
            };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        })?;

    let user = user.ok_or_else(|| {
        let response = ErrorResponse {
            status: "failure",
            message: format!("Unable to find user in the database"),
        };

        (StatusCode::UNAUTHORIZED, Json(response))
    })?;

    req.extensions_mut().insert(JWTAuthMiddleware {
        user: user.into(),
        access_token_uuid,
    });

    Ok(next.run(req).await)
}
*/
