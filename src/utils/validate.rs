use crate::error::{AppError, AppResult};
use axum::extract::{
    Form, FromRequest, FromRequestParts, Json, Query, Request,
    rejection::{FormRejection, JsonRejection, QueryRejection},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// Validated Form extractor
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> AppResult<Self> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

/// Validated JSON extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> AppResult<Self> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// Validated Query extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> AppResult<Self> {
        let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, header};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestData {
        #[validate(length(
            min = 3,
            max = 20,
            message = "Username must be between 3 and 20 characters"
        ))]
        username: String,
        #[validate(email(message = "Invalid email format"))]
        email: String,
        #[validate(range(min = 18, max = 100, message = "Age must be between 18 and 100"))]
        age: u8,
    }

    // Form tests
    #[tokio::test]
    async fn test_valid_form() {
        let body = "username=testuser&email=test@example.com&age=25";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_ok());
        let ValidatedForm(form) = result.unwrap();
        assert_eq!(form.username, "testuser");
        assert_eq!(form.email, "test@example.com");
        assert_eq!(form.age, 25);
    }

    #[tokio::test]
    async fn test_form_validation_error_short_username() {
        let body = "username=ab&email=test@example.com&age=25";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "username");
                assert!(errors[0].message.contains("between 3 and 20 characters"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_form_validation_error_invalid_email() {
        let body = "username=testuser&email=invalid-email&age=25";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "email");
                assert!(errors[0].message.contains("Invalid email format"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_form_validation_error_age_out_of_range() {
        let body = "username=testuser&email=test@example.com&age=150";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "age");
                assert!(errors[0].message.contains("between 18 and 100"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_form_validation_error_multiple_fields() {
        let body = "username=ab&email=invalid-email&age=150";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 3);
                let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
                assert!(fields.contains(&"username"));
                assert!(fields.contains(&"email"));
                assert!(fields.contains(&"age"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_form_rejection_missing_field() {
        let body = "username=testuser&email=test@example.com";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_form_rejection_invalid_content_type() {
        let body = "username=testuser&email=test@example.com&age=25";
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedForm::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }

    // JSON tests
    #[tokio::test]
    async fn test_valid_json() {
        let body = r#"{"username":"testuser","email":"test@example.com","age":25}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_ok());
        let ValidatedJson(data) = result.unwrap();
        assert_eq!(data.username, "testuser");
        assert_eq!(data.email, "test@example.com");
        assert_eq!(data.age, 25);
    }

    #[tokio::test]
    async fn test_json_validation_error_short_username() {
        let body = r#"{"username":"ab","email":"test@example.com","age":25}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "username");
                assert!(errors[0].message.contains("between 3 and 20 characters"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_json_validation_error_invalid_email() {
        let body = r#"{"username":"testuser","email":"invalid-email","age":25}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "email");
                assert!(errors[0].message.contains("Invalid email format"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_json_validation_error_multiple_fields() {
        let body = r#"{"username":"ab","email":"invalid-email","age":150}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 3);
                let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
                assert!(fields.contains(&"username"));
                assert!(fields.contains(&"email"));
                assert!(fields.contains(&"age"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_json_rejection_missing_field() {
        let body = r#"{"username":"testuser","email":"test@example.com"}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_json_rejection_invalid_json() {
        let body = r#"{"username":"testuser","email":"test@example.com","age":"invalid"}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }

    // Query tests
    #[tokio::test]
    async fn test_valid_query() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=testuser&email=test@example.com&age=25")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let ValidatedQuery(data) = result.unwrap();
        assert_eq!(data.username, "testuser");
        assert_eq!(data.email, "test@example.com");
        assert_eq!(data.age, 25);
    }

    #[tokio::test]
    async fn test_query_validation_error_short_username() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=ab&email=test@example.com&age=25")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "username");
                assert!(errors[0].message.contains("between 3 and 20 characters"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_query_validation_error_invalid_email() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=testuser&email=invalid-email&age=25")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "email");
                assert!(errors[0].message.contains("Invalid email format"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_query_validation_error_multiple_fields() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=ab&email=invalid-email&age=150")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::ValidationErrors { errors } => {
                assert_eq!(errors.len(), 3);
                let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
                assert!(fields.contains(&"username"));
                assert!(fields.contains(&"email"));
                assert!(fields.contains(&"age"));
            }
            _ => panic!("Expected ValidationErrors error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_query_rejection_missing_field() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=testuser&email=test@example.com")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_query_rejection_invalid_type() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test?username=testuser&email=test@example.com&age=invalid")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = request.into_parts();
        let result = ValidatedQuery::<TestData>::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AppError::BadRequest { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected BadRequest error, got {:?}", error),
        }
    }
}
