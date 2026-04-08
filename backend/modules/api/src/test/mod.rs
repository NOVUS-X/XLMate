#[cfg(test)]
mod rate_limit;

#[cfg(test)]
mod tests {
    use dto::players::{NewPlayer, InvalidPlayer};
    use validator::Validate;
    use serde_json::json;

    // Load test environment before running tests
    fn setup_test_env() {
        dotenv::from_filename(".env.test").ok();
    }

    #[actix_web::test]
    async fn test_index_post_no_body() {
        setup_test_env();
        let payload = NewPlayer::test_player();
        
        // Test validation without body (should pass)
        match payload.validate() {
            Ok(_) => {
                // Should reach here since test_player() creates valid data
                assert!(true, "Valid player should pass validation");
            }
            Err(_) => {
                panic!("Valid player should not fail validation");
            }
        }
    }

    #[actix_web::test]
    async fn test_index_post_with_body() {
        setup_test_env();
        let payload = NewPlayer::test_player();
        
        // Test validation with valid body
        match payload.validate() {
            Ok(_) => {
                // Should reach here since test_player() creates valid data
                assert!(true, "Valid player should pass validation");
                
                // Test response structure
                let response = json!({
                    "message":"New player added",
                    "data": {
                        "id": "test-id",
                        "username": payload.username,
                        "email": payload.email,
                        "real_name": payload.real_name
                    }
                });
                
                assert!(
                    response.get("data").is_some(),
                    "Response should contain a 'data' field"
                );
                let data = &response["data"];
                
                assert!(
                    data.get("id").is_some(),
                    "Response should contain player ID"
                );
                assert!(
                    data.get("username").is_some(),
                    "Response should contain username"
                );
                assert!(
                    data.get("email").is_some(),
                    "Response should contain email address"
                );
                assert!(
                    data.get("real_name").is_some(),
                    "Response should contain 'real_name'"
                );
            }
            Err(errors) => {
                panic!("Valid player should not fail validation: {:?}", errors);
            }
        }
    }

    #[actix_web::test]
    async fn test_index_post_with_invalid_username() {
        setup_test_env();
        let payload = NewPlayer::invalid_player(InvalidPlayer::Username);
        
        // Test validation with invalid username
        match payload.validate() {
            Ok(_) => {
                panic!("Invalid username should fail validation");
            }
            Err(errors) => {
                let error_response = json!({
                    "error": format!("{:?}", errors),
                    "code": "VALIDATION_ERROR"
                });
                
                assert!(
                    error_response.get("error").is_some(),
                    "Response should contain an 'error' field"
                );
                assert!(
                    error_response.get("code").is_some(),
                    "Response should contain a 'code' field"
                );
                let error = error_response["error"].as_str().unwrap();
                assert!(
                    error.contains("Username"),
                    "Error should mention username field"
                );
            }
        }
    }

    #[actix_web::test]
    async fn test_index_post_with_invalid_email() {
        setup_test_env();
        let payload = NewPlayer::invalid_player(InvalidPlayer::Email);
        
        // Test validation with invalid email
        match payload.validate() {
            Ok(_) => {
                panic!("Invalid email should fail validation");
            }
            Err(errors) => {
                let error_response = json!({
                    "error": format!("{:?}", errors),
                    "code": "VALIDATION_ERROR"
                });
                
                assert!(
                    error_response.get("error").is_some(),
                    "Response should contain an 'error' field"
                );
                assert!(
                    error_response.get("code").is_some(),
                    "Response should contain a 'code' field"
                );
                let error = error_response["error"].as_str().unwrap();
                assert!(
                    error.contains("email"),
                    "Error should mention email field"
                );
            }
        }
    }

    #[actix_web::test]
    async fn test_index_post_with_invalid_password() {
        setup_test_env();
        let payload = NewPlayer::invalid_player(InvalidPlayer::Password);
        
        // Test validation with invalid password
        match payload.validate() {
            Ok(_) => {
                panic!("Invalid password should fail validation");
            }
            Err(errors) => {
                let error_response = json!({
                    "error": format!("{:?}", errors),
                    "code": "VALIDATION_ERROR"
                });
                
                assert!(
                    error_response.get("error").is_some(),
                    "Response should contain an 'error' field"
                );
                assert!(
                    error_response.get("code").is_some(),
                    "Response should contain a 'code' field"
                );
                let error = error_response["error"].as_str().unwrap();
                assert!(
                    error.contains("Password"),
                    "Error should mention password field"
                );
            }
        }
    }
}
