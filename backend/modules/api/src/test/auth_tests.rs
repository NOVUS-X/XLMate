#[cfg(test)]
mod auth_tests {
    use actix_web::{App, dev::Service, http::StatusCode, test, web, cookie::Cookie};
    use dto::auth::{LoginRequest, RegisterRequest, RefreshTokenRequest};
    use serde_json::Value;

    use crate::auth::{login, register, refresh_token, logout};

    fn get_refresh_cookie(res: &actix_web::dev::ServiceResponse) -> Option<String> {
        res.response()
            .cookies()
            .find(|c| c.name() == "refresh_token")
            .map(|c| c.value().to_string())
    }

    #[actix_web::test]
    async fn test_login_returns_access_token_and_cookie() {
        let app = test::init_service(
            App::new()
                .service(web::scope("/v1/auth")
                    .service(register)
                    .service(login))
        ).await;

        // 1. Register
        let r_req = test::TestRequest::post()
            .uri("/v1/auth/register")
            .set_json(RegisterRequest {
                username: "login_tester".to_string(),
                email: "login@example.com".to_string(),
                password: "Password123!".to_string(),
                wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            })
            .to_request();
        let _ = app.call(r_req).await.unwrap();

        // 2. Login
        let req = test::TestRequest::post()
            .uri("/v1/auth/login")
            .set_json(LoginRequest {
                username: "login_tester".to_string(),
                password: "Password123!".to_string(),
            })
            .to_request();

        let res = app.call(req).await.unwrap();
        
        // Assert status first
        assert_eq!(res.status(), StatusCode::OK);
        
        // Assert cookie
        let cookie = get_refresh_cookie(&res);
        assert!(cookie.is_some(), "Response should contain refresh_token cookie");
        
        // Assert body
        let body: Value = test::read_body_json(res).await;
        assert!(body.get("access_token").is_some());
        assert!(body.get("token_type").is_some());
        assert!(body.get("expires_in").is_some());
    }

    #[actix_web::test]
    async fn test_refresh_rotates_token() {
        let app = test::init_service(
            App::new()
                .service(web::scope("/v1/auth")
                    .service(register)
                    .service(login)
                    .service(refresh_token))
        ).await;

        // 1. Register & Login
        let r_req = test::TestRequest::post()
            .uri("/v1/auth/register")
            .set_json(RegisterRequest {
                username: "refresh_tester".to_string(),
                email: "refresh@example.com".to_string(),
                password: "Password123!".to_string(),
                wallet_address: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
            })
            .to_request();
        let _ = app.call(r_req).await.unwrap();

        let l_req = test::TestRequest::post()
            .uri("/v1/auth/login")
            .set_json(LoginRequest {
                username: "refresh_tester".to_string(),
                password: "Password123!".to_string(),
            })
            .to_request();
        let l_res = app.call(l_req).await.unwrap();
        let old_token = get_refresh_cookie(&l_res).expect("Login should verify");

        // 2. Refresh
        let req = test::TestRequest::post()
            .uri("/v1/auth/refresh")
            .cookie(Cookie::new("refresh_token", old_token.clone()))
            .to_request();

        let res = app.call(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        
        let new_token = get_refresh_cookie(&res).expect("Refresh should verify");
        assert_ne!(old_token, new_token, "Token should rotate");
    }

    #[actix_web::test]
    async fn test_refresh_with_used_token_returns_theft_error() {
        let app = test::init_service(
            App::new()
                .service(web::scope("/v1/auth")
                    .service(register)
                    .service(login)
                    .service(refresh_token))
        ).await;

        // 1. Setup User & Token
        let r_req = test::TestRequest::post()
            .uri("/v1/auth/register")
            .set_json(RegisterRequest {
                username: "theft_tester".to_string(),
                email: "theft@example.com".to_string(),
                password: "Password123!".to_string(),
                wallet_address: "0x1111111111111111111111111111111111111111".to_string(),
            })
            .to_request();
        let _ = app.call(r_req).await.unwrap();

        let l_req = test::TestRequest::post()
            .uri("/v1/auth/login")
            .set_json(LoginRequest {
                username: "theft_tester".to_string(),
                password: "Password123!".to_string(),
            })
            .to_request();
        let l_res = app.call(l_req).await.unwrap();
        let token_a = get_refresh_cookie(&l_res).expect("Login failed");

        // 2. Refresh (Rotates A -> B, A is now used)
        let req1 = test::TestRequest::post()
            .uri("/v1/auth/refresh")
            .cookie(Cookie::new("refresh_token", token_a.clone()))
            .to_request();
        let res1 = app.call(req1).await.unwrap();
        assert_eq!(res1.status(), StatusCode::OK);

        // 3. Reuse A (Theft Attempt)
        let req2 = test::TestRequest::post()
            .uri("/v1/auth/refresh")
            .cookie(Cookie::new("refresh_token", token_a.clone()))
            .to_request();
        let res2 = app.call(req2).await.unwrap();
        
        assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);
        // Can't check "theft_detected" boolean easily without knowing exact response struct,
        // but 401 is expected for theft/reuse.
    }

    #[actix_web::test]
    async fn test_logout_clears_cookie() {
        let app = test::init_service(
            App::new().service(web::scope("/v1/auth").service(logout))
        ).await;

        let req = test::TestRequest::post()
            .uri("/v1/auth/logout")
            .to_request();

        let res = app.call(req).await.unwrap();
        
        assert_eq!(res.status(), StatusCode::OK);
        
        let cookie = res.response()
            .cookies()
            .find(|c| c.name() == "refresh_token")
            .expect("Logout response must contain Set-Cookie for refresh_token");
        
        assert!(cookie.value().is_empty() || cookie.max_age() == Some(time::Duration::seconds(0)), 
                "Cookie must be cleared (empty or max-age 0)");
    }

    #[actix_web::test]
    async fn test_register_creates_player_and_tokens() {
        let app = test::init_service(
            App::new().service(web::scope("/v1/auth").service(register))
        ).await;

        let req = test::TestRequest::post()
            .uri("/v1/auth/register")
            .set_json(RegisterRequest {
                username: "new_reg_user".to_string(),
                email: "new_reg@example.com".to_string(),
                password: "Password123!".to_string(),
                wallet_address: "0x2222222222222222222222222222222222222222".to_string(),
            })
            .to_request();

        let res = app.call(req).await.unwrap();
        
        if res.status() != StatusCode::CREATED {
             let status = res.status();
             let body = test::read_body(res).await;
             panic!("Register failed: Status {}, Body {:?}", status, body);
        }
        
        // Assert cookie presence
        let cookie = get_refresh_cookie(&res);
        assert!(cookie.is_some());
        
        // Assert body presence
        let body: Value = test::read_body_json(res).await;
        assert!(body.get("access_token").is_some());
    }
}
