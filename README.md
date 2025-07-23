# J1R4 Clone Backend

Backend del clon de Jira desarrollado en Rust con Axum.

## Configuración

### Variables de Entorno

Copia el archivo `.env.example` a `.env` y configura las siguientes variables:

```env
DATABASE_URL=mongodb://localhost:27017
DATABASE_NAME=jira_clone
JWT_SECRET=your-super-secret-jwt-key-here
SERVER_ADDRESS=127.0.0.1:8000
CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173

# Configuración de Google Cloud Storage (opcional para desarrollo)
GCS_BUCKET_NAME=tu-bucket-nombre
GCS_PROJECT_ID=tu-proyecto-id
GOOGLE_APPLICATION_CREDENTIALS=/ruta/a/credenciales.json
```

### CORS Configuration

El backend está configurado con CORS (Cross-Origin Resource Sharing) para permitir peticiones desde tu frontend. 

**Configuración de CORS:**
- **Orígenes permitidos**: Se configuran a través de la variable `CORS_ORIGINS`
- **Métodos permitidos**: GET, POST, PATCH, DELETE, OPTIONS
- **Headers permitidos**: Authorization, Content-Type, Accept
- **Credenciales**: Habilitadas para soportar cookies y headers de autenticación

**Puertos por defecto incluidos:**
- `http://localhost:3000` - React, Next.js
- `http://localhost:5173` - Vite
- `http://localhost:3001` - Aplicaciones adicionales

### Instalación y Ejecución

1. Instala las dependencias:
```bash
cargo build
```

2. Configura las variables de entorno:
```bash
cp .env.example .env
# Edita el archivo .env con tus configuraciones
```

3. Ejecuta el servidor:
```bash
cargo run
```

El servidor estará disponible en `http://127.0.0.1:8000`

### Endpoints Principales

- **Autenticación**: `/api/auth/login`, `/api/auth/register`
- **Proyectos**: `/api/projects`
- **Tareas**: `/api/tasks`
- **Comentarios**: `/api/tasks/{task_id}/comments`
- **Imágenes**: `/api/images` (subida, descarga, gestión)
- **Fechas**: `/api/tasks/{task_id}/date-range`
- **WebSocket**: `/ws`

### Documentación Detallada

Consulta la documentación específica en la carpeta `/docs`:
- **Imágenes**: [docs/IMAGES_ENDPOINTS_DOCS.md](docs/IMAGES_ENDPOINTS_DOCS.md)
- **Fechas**: [docs/FECHA_ENDPOINTS_DOCS.md](docs/FECHA_ENDPOINTS_DOCS.md)
- **Deployment**: [docs/SHUTTLE_DEPLOYMENT_GUIDE.md](docs/SHUTTLE_DEPLOYMENT_GUIDE.md)

### Desarrollo

Para desarrollo, asegúrate de que tu frontend esté ejecutándose en uno de los puertos configurados en `CORS_ORIGINS` o añade tu puerto específico a la variable de entorno.

Ejemplo para un frontend en el puerto 3000:
```bash
# Tu frontend debería estar en http://localhost:3000
npm start  # o yarn dev, etc.
```

### Pruebas

```bash
# Ejecutar todos los tests (recomendado con un solo hilo)
cargo test -- --test-threads=1

# Ejecutar solo tests específicos
cargo test simple_image_upload_test -- --nocapture
```

**Nota**: Los tests de imágenes no requieren configuración de Google Cloud Storage real, están diseñados para funcionar en cualquier entorno de CI/CD.
