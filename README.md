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
- **WebSocket**: `/ws`

### Desarrollo

Para desarrollo, asegúrate de que tu frontend esté ejecutándose en uno de los puertos configurados en `CORS_ORIGINS` o añade tu puerto específico a la variable de entorno.

Ejemplo para un frontend en el puerto 3000:
```bash
# Tu frontend debería estar en http://localhost:3000
npm start  # o yarn dev, etc.
```

### Pruebas

```bash
cargo test
```
