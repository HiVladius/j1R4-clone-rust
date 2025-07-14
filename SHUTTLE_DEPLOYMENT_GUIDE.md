# 🚀 Guía de Deployment a Shuttle

Esta documentación describe el proceso completo para subir un proyecto Rust/Axum existente a la plataforma Shuttle.

## 📋 Tabla de Contenidos

1. [Prerequisitos](#prerequisitos)
2. [Instalación de Shuttle CLI](#instalación-de-shuttle-cli)
3. [Configuración del Proyecto](#configuración-del-proyecto)
4. [Modificaciones Necesarias](#modificaciones-necesarias)
5. [Configuración de Secretos](#configuración-de-secretos)
6. [Deployment](#deployment)
7. [Solución de Problemas](#solución-de-problemas)
8. [Comandos Útiles](#comandos-útiles)

## 🔧 Prerequisitos

- **Rust** instalado (versión 1.70+)
- **Proyecto Axum** existente
- **Cuenta en Shuttle** (https://shuttle.dev)

## 📦 Instalación de Shuttle CLI

### Windows (PowerShell)
```powershell
iwr https://www.shuttle.dev/install-win | iex
```

### Linux/macOS
```bash
curl -sSfL https://www.shuttle.dev/install | bash
```

### Alternativa con Cargo
```bash
cargo install cargo-shuttle
```

### Verificar instalación
```bash
shuttle --version
```

## ⚙️ Configuración del Proyecto

### 1. Agregar dependencias a `Cargo.toml`

```toml
[dependencies]
# ... tus dependencias existentes ...
shuttle-runtime = "0.56.0"
shuttle-axum = "0.56.0"
```

### 2. Modificar `main.rs`

#### ❌ Antes (configuración local)
```rust
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicialización manual de tracing
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let config = Config::from_env()?;
    let app = create_app().await?;
    
    // Bind manual del servidor
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    
    Ok(())
}
```

#### ✅ Después (configuración Shuttle)
```rust
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // Shuttle maneja automáticamente tracing - NO inicializar manualmente
    
    tracing::info!("Starting application...");
    
    let config = Config::from_secrets(&secrets)?;
    let app = create_app().await?;
    
    // Shuttle maneja automáticamente el servidor HTTP
    Ok(app.into())
}
```

### 3. Modificar configuración para Shuttle

#### Agregar método `from_secrets` en `config.rs`:

```rust
use shuttle_runtime::SecretStore;

impl Config {
    // Método existente para desarrollo local
    pub fn from_env() -> Result<Self, env::VarError> {
        // ... código existente ...
    }

    // Nuevo método para Shuttle
    pub fn from_secrets(secrets: &SecretStore) -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: secrets.get("DATABASE_URL").ok_or(env::VarError::NotPresent)?,
            database_name: secrets.get("DATABASE_NAME").ok_or(env::VarError::NotPresent)?,
            jwt_secret: secrets.get("JWT_SECRET").ok_or(env::VarError::NotPresent)?,
            server_address: secrets
                .get("SERVER_ADDRESS")
                .unwrap_or_else(|| "127.0.0.1:8000".to_string()),
            cors_origins: secrets
                .get("CORS_ORIGINS")
                .unwrap_or_else(|| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}
```

## 🔐 Configuración de Secretos

### 1. Crear archivo `Secrets.toml` en la raíz del proyecto

```toml
# Secrets.toml
DATABASE_URL="mongodb+srv://usuario:password@cluster.mongodb.net/"
DATABASE_NAME="mi_database"
JWT_SECRET="mi_jwt_secret_super_seguro"
SERVER_ADDRESS="127.0.0.1:8000"
CORS_ORIGINS="http://localhost:3000,http://localhost:5173"
RUST_LOG="debug"
```

### ⚠️ Importante: Formato TOML
- **Todos los valores deben estar entre comillas dobles**
- **NO usar** `KEY=value` sino `KEY="value"`

### 2. Agregar `Secrets.toml` a `.gitignore`

```gitignore
# Secrets para Shuttle
Secrets.toml
```

## 🚀 Deployment

### 1. Login a Shuttle
```bash
shuttle login
```

### 2. Inicializar proyecto Shuttle (solo primera vez)
```bash
shuttle init --name mi-proyecto
```

### 3. Deploy
```bash
shuttle deploy
```

### 4. Ver logs
```bash
shuttle logs
```

### 5. Ver status
```bash
shuttle status
```

## 🐛 Solución de Problemas

### Error: "failed to set global default subscriber"

**Problema:** Intentas inicializar tracing manualmente cuando Shuttle ya lo hace.

**Solución:** Eliminar toda inicialización manual de tracing:

```rust
// ❌ ELIMINAR estas líneas
tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(fmt::layer())
    .init();
```

### Error: "Error al cargar la configuración: NotPresent"

**Problema:** Las variables no se cargan desde `Secrets.toml`.

**Soluciones:**
1. Verificar formato TOML (comillas dobles)
2. Usar `Config::from_secrets(&secrets)` en lugar de `Config::from_env()`
3. Agregar parámetro `#[shuttle_runtime::Secrets] secrets: SecretStore` a main

### Error: "TOML parse error"

**Problema:** Formato incorrecto en `Secrets.toml`.

**Solución:** Asegurar que todos los valores estén entre comillas:
```toml
# ✅ Correcto
DATABASE_URL="mi_url"

# ❌ Incorrecto  
DATABASE_URL=mi_url
```

### Error: "trait bound `(): Service` is not satisfied"

**Problema:** Tipo de retorno incorrecto en main.

**Solución:** 
```rust
// ✅ Correcto
async fn main() -> shuttle_axum::ShuttleAxum {
    // ...
    Ok(app.into())
}

// ❌ Incorrecto
async fn main() -> Result<(), Error> {
    // ...
    Ok(())
}
```

## 📝 Comandos Útiles

### Gestión del proyecto
```bash
# Ver status del deployment
shuttle status

# Ver logs en tiempo real
shuttle logs --follow

# Parar el servicio
shuttle stop

# Reiniciar el servicio  
shuttle start

# Ver información del proyecto
shuttle project status

# Eliminar deployment
shuttle delete
```

### Desarrollo local vs Shuttle

```rust
// Para mantener compatibilidad con ambos entornos
impl Config {
    #[cfg(feature = "shuttle")]
    pub fn load(secrets: &SecretStore) -> Result<Self, env::VarError> {
        Self::from_secrets(secrets)
    }
    
    #[cfg(not(feature = "shuttle"))]
    pub fn load() -> Result<Self, env::VarError> {
        Self::from_env()
    }
}
```

## 🔄 Diferencias Clave: Local vs Shuttle

| Aspecto | Desarrollo Local | Shuttle |
|---------|------------------|---------|
| **Variables de entorno** | `.env` + `std::env::var()` | `Secrets.toml` + `SecretStore` |
| **Servidor HTTP** | Manual con `TcpListener` | Automático |
| **Logging** | Configuración manual | Automático |
| **HTTPS** | Ninguno | Automático |
| **Puerto** | Configurable | Asignado automáticamente |
| **Tipo de retorno main** | `Result<(), Error>` | `shuttle_axum::ShuttleAxum` |

## 🎯 Checklist Final

- [ ] ✅ Dependencias `shuttle-runtime` y `shuttle-axum` agregadas
- [ ] ✅ Función main modificada para recibir `SecretStore`
- [ ] ✅ Método `Config::from_secrets()` implementado
- [ ] ✅ Inicialización manual de tracing eliminada
- [ ] ✅ Archivo `Secrets.toml` creado con formato correcto
- [ ] ✅ `Secrets.toml` agregado a `.gitignore`
- [ ] ✅ Shuttle CLI instalado y configurado
- [ ] ✅ Login realizado (`shuttle login`)
- [ ] ✅ Deployment exitoso (`shuttle deploy`)

## 🆘 Recursos Adicionales

- **Documentación oficial:** https://docs.shuttle.dev/
- **Ejemplos:** https://github.com/shuttle-hq/shuttle-examples
- **Discord:** https://discord.com/invite/shuttle
- **GitHub:** https://github.com/shuttle-hq/shuttle

---

**Fecha de creación:** 12 de julio de 2025  
**Proyecto:** Jira Clone Backend  
**Plataforma:** Shuttle.dev
