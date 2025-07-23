#[cfg(test)]
mod simple_image_upload_test {
    use crate::helpers::helper_setup_app::setup_app;
    use axum::{body::Body, http::Request};
    use serde_json::json;
    use tower::ServiceExt;

    // Test simple que solo verifica que el endpoint de imagen existe y responde
    #[tokio::test]
    async fn test_image_endpoint_exists() {
        let app = setup_app().await;
        
        // 1. Crear y autenticar usuario
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let unique_username = format!("testuser{}", timestamp);
        let unique_email = format!("test{}@example.com", timestamp);

        // Registrar usuario
        let register_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/register")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "username": unique_username,
                            "email": unique_email,
                            "password": "password123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(register_response.status(), 201, "Failed to register user");

        // Login para obtener token
        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/login")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "email": unique_email,
                            "password": "password123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(login_response.status(), 200, "Failed to login");

        let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
        let token = login_data["token"].as_str().unwrap();

        // 2. Verificar que el endpoint de imagen GET existe y responde correctamente
        let list_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/images")
                    .method("GET")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = list_response.status();
        println!("List images response status: {}", status);
        
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list_body_str = String::from_utf8(list_body.to_vec()).unwrap();
        println!("List images response body: {}", list_body_str);

        // Verificar que el endpoint responde (200 OK o error del servicio, no 404/405)
        assert!(
            status != 404 && status != 405,
            "Endpoint should exist and not return 404/405, got: {} with body: {}",
            status,
            list_body_str
        );

        println!("✅ Image endpoint exists and responds!");
    }

    #[tokio::test]
    async fn test_image_upload_endpoint_configured() {
        let app = setup_app().await;
        
        // 1. Crear y autenticar usuario (código reutilizado del test anterior)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let unique_username = format!("testuser{}", timestamp);
        let unique_email = format!("test{}@example.com", timestamp);

        // Registro y login
        let register_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/register")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "username": unique_username,
                            "email": unique_email,
                            "password": "password123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(register_response.status(), 201);

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/login")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "email": unique_email,
                            "password": "password123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
        let token = login_data["token"].as_str().unwrap();

        // 2. Verificar endpoint POST sin enviar datos reales (solo verificar que existe)
        let upload_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/images")
                    .method("POST")
                    .header("authorization", format!("Bearer {}", token))
                    .header("content-type", "text/plain") // Content-type incorrecto a propósito
                    .body(Body::from("test"))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = upload_response.status();
        println!("Upload endpoint response status: {}", status);

        // Verificar que no es 404 o 405 (método no permitido)
        // Esperamos 400 (bad request) o 500 (error interno) pero NO 404/405
        assert!(
            status != 404 && status != 405,
            "Upload endpoint should exist, got: {}",
            status
        );

        println!("✅ Image upload endpoint is configured correctly!");
    }
}
