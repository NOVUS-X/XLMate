use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    body::{BoxBody, MessageBody},
    HttpMessage, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtAuthMiddleware {
    secret_key: Rc<String>,
}

impl JwtAuthMiddleware {
    pub fn new(secret_key: String) -> Self {
        JwtAuthMiddleware {
            secret_key: Rc::new(secret_key),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddlewareService {
            service,
            secret_key: self.secret_key.clone(),
        })
    }
}

pub struct JwtAuthMiddlewareService<S> {
    service: S,
    secret_key: Rc<String>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret_key = self.secret_key.clone();
        let auth_header = req.headers().get("Authorization");
        
        // Check for Authorization header
        let auth_result = match auth_header {
            Some(header) => {
                let header_str = header.to_str().unwrap_or("");
                if !header_str.starts_with("Bearer ") {
                    // Invalid format
                    return Box::pin(async move {
                        Ok(req.into_response(
                            HttpResponse::Unauthorized()
                                .json(serde_json::json!({
                                    "error": "Invalid authorization token format",
                                    "code": 401
                                }))
                        ).map_into_boxed_body())
                    });
                }
                
                let token = &header_str[7..]; // Remove "Bearer " prefix
                let validation = Validation::new(Algorithm::HS256);
                
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret_key.as_bytes()),
                    &validation,
                ) {
                    Ok(token_data) => {
                        // Valid token
                        req.extensions_mut().insert(token_data.claims);
                        true
                    }
                    Err(_) => false, // Invalid token
                }
            }
            None => false, // No token
        };
        
        if auth_result {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            })
        } else {
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid or missing authorization token",
                            "code": 401
                        }))
                ).map_into_boxed_body())
            })
        }
    }
}
