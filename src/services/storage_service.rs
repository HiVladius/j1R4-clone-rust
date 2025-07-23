use std::sync::Arc;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        upload::{Media, UploadObjectRequest, UploadType},
        Object,
    },
};
use mime_guess;
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket_name: String,
}

impl StorageService {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        // Si hay credenciales específicas, configurarlas
        if let Some(credentials_path) = &config.google_application_credentials {
            std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", credentials_path);
        }

        let client_config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| anyhow!("Error configurando cliente GCS: {}", e))?;

        let client = Client::new(client_config);

        Ok(Self {
            client,
            bucket_name: config.gcs_bucket_name.clone(),
        })
    }

    /// Sube un archivo al bucket de GCS
    pub async fn upload_file(
        &self,
        file_name: &str,
        file_data: Bytes,
        content_type: Option<String>,
    ) -> Result<String> {
        // Generar un nombre único para el archivo
        let unique_file_name = format!("{}-{}", Uuid::new_v4(), file_name);

        // Determinar el content type si no se proporcionó
        let content_type = content_type.unwrap_or_else(|| {
            mime_guess::from_path(file_name)
                .first_or_octet_stream()
                .to_string()
        });

        let upload_type = UploadType::Simple(Media::new(unique_file_name.clone()));
        let upload_request = UploadObjectRequest {
            bucket: self.bucket_name.clone(),
            ..Default::default()
        };

        // Crear el objeto con metadatos
        let object = Object {
            name: Some(unique_file_name.clone()),
            content_type: Some(content_type),
            ..Default::default()
        };

        self.client
            .upload_object(&upload_request, file_data, &upload_type)
            .await
            .map_err(|e| anyhow!("Error subiendo archivo a GCS: {}", e))?;

        // Retornar la URL pública del archivo
        let public_url = format!(
            "https://storage.googleapis.com/{}/{}",
            self.bucket_name, unique_file_name
        );

        Ok(public_url)
    }

    /// Elimina un archivo del bucket
    pub async fn delete_file(&self, file_url: &str) -> Result<()> {
        // Extraer el nombre del archivo de la URL
        let file_name = file_url
            .split('/')
            .last()
            .ok_or_else(|| anyhow!("URL de archivo inválida: {}", file_url))?;

        self.client
            .delete_object(&self.bucket_name, file_name, None)
            .await
            .map_err(|e| anyhow!("Error eliminando archivo de GCS: {}", e))?;

        Ok(())
    }

    /// Lista archivos en el bucket (opcional, para debugging)
    pub async fn list_files(&self, prefix: Option<String>) -> Result<Vec<Object>> {
        let mut request = google_cloud_storage::http::objects::list::ListObjectsRequest {
            bucket: self.bucket_name.clone(),
            ..Default::default()
        };

        if let Some(prefix) = prefix {
            request.prefix = Some(prefix);
        }

        let response = self
            .client
            .list_objects(&request)
            .await
            .map_err(|e| anyhow!("Error listando archivos en GCS: {}", e))?;

        Ok(response.items.unwrap_or_default())
    }
}
