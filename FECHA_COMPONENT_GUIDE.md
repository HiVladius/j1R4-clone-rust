# Componente de Fechas - Guía Simplificada Backend

## ✅ Implementación Completada

### 1. Modelo de Datos

Se agregaron los siguientes campos al modelo `Task`:

```rust
pub struct Task {
    // ... campos existentes ...
    pub start_date: Option<DateTime<Utc>>,      // Fecha de inicio (como "Due date")
    pub end_date: Option<DateTime<Utc>>,        // Fecha de finalización
    pub has_due_date: bool,                     // Toggle para activar/desactivar end_date
}
```

### 2. Esquemas de Validación

**Para crear tareas:**
```rust
pub struct CreateTaskSchema {
    // ... campos existentes ...
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub has_due_date: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,    // Fecha de creación manual (opcional)
    pub updated_at: Option<DateTime<Utc>>,    // Fecha de actualización manual (opcional)
}
```

**Para actualizar tareas:**
```rust
pub struct UpdateTaskSchema {
    // ... campos existentes ...
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<Option<DateTime<Utc>>>, // Permite remover la fecha
    pub has_due_date: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,       // Fecha de actualización manual (opcional)
}
```

### 3. Validaciones Implementadas

- ✅ Si `has_due_date` es `true`, `end_date` debe estar presente
- ✅ Si `has_due_date` es `false`, `end_date` debe ser `None`
- ✅ `start_date` debe ser anterior a `end_date` cuando ambas están presentes

## 📝 Ejemplos de Uso

### Crear una tarea con fechas manuales
```json
POST /api/projects/{project_id}/tasks
{
    "title": "Mi tarea",
    "description": "Descripción de la tarea",
    "start_date": "2025-07-03T09:00:00Z",
    "end_date": "2025-07-10T17:00:00Z",
    "has_due_date": true,
    "created_at": "2025-07-03T08:30:00Z",
    "updated_at": "2025-07-03T08:30:00Z",
    "priority": "Medium",
    "status": "ToDo"
}
```

### Crear una tarea solo con fecha de inicio (automática)
```json
POST /api/projects/{project_id}/tasks
{
    "title": "Mi tarea",
    "start_date": "2025-07-03T09:00:00Z",
    "has_due_date": false,
    "priority": "Medium",
    "status": "ToDo"
    // created_at y updated_at se generarán automáticamente
}
```

### Actualizar fechas con timestamp manual
```json
PATCH /api/tasks/{task_id}
{
    "start_date": "2025-07-04T09:00:00Z",
    "end_date": "2025-07-12T17:00:00Z",
    "updated_at": "2025-07-04T11:15:00Z"
}
```

### Activar fecha de finalización
```json
PATCH /api/tasks/{task_id}
{
    "has_due_date": true,
    "end_date": "2025-07-15T17:00:00Z"
}
```

### Desactivar fecha de finalización
```json
PATCH /api/tasks/{task_id}
{
    "has_due_date": false,
    "end_date": null
}
```

## 📅 **Control Manual de Fechas**

### Fechas de Sistema (created_at / updated_at)

Ahora puedes controlar manualmente cuándo se creó y actualizó una tarea:

**Comportamiento:**
- Si proporcionas `created_at` → usa esa fecha
- Si no la proporcionas → se genera automáticamente con `Utc::now()`
- Si proporcionas `updated_at` → usa esa fecha  
- Si no la proporcionas → se genera automáticamente con `Utc::now()`

**Casos de uso:**
- ✅ Migración de datos de otros sistemas
- ✅ Importación de tareas con fechas históricas
- ✅ Sincronización con sistemas externos
- ✅ Control preciso de timestamps para testing

### Ejemplo de Migración
```json
// Migrar una tarea creada hace una semana
{
    "title": "Tarea migrada",
    "created_at": "2025-06-26T14:30:00Z",
    "updated_at": "2025-06-28T09:15:00Z",
    "start_date": "2025-06-26T14:30:00Z",
    "has_due_date": true,
    "end_date": "2025-07-05T17:00:00Z"
}
```

## 🔗 Integración con Frontend

Para el frontend, el comportamiento debe ser:

1. **Campo "Due date"** → `start_date` (siempre visible)
2. **Toggle "End date"** → `has_due_date` (switch para activar/desactivar)
3. **Campo "End date"** → `end_date` (solo visible cuando toggle está activado)

### Lógica del Frontend:
```javascript
// Cuando el usuario activa el toggle
if (has_due_date) {
    // Mostrar campo end_date
    // Validar que end_date > start_date
} else {
    // Ocultar campo end_date
    // Establecer end_date = null
}
```

## 🎯 Endpoints Disponibles

| Endpoint | Method | Propósito |
|----------|--------|-----------|
| `/api/projects/{id}/tasks` | POST | Crear tarea con fechas |
| `/api/tasks/{id}` | GET | Obtener tarea (incluye fechas) |
| `/api/tasks/{id}` | PATCH | Actualizar fechas |
| `/api/projects/{id}/tasks` | GET | Listar tareas (incluye fechas) |

## 📮 Documentación de Postman

**Ubicación:** `src/postma-collection/enviroment.json`

**Endpoints de prueba:**
- ✅ Create Task (con campos de fecha opcionales)
- ✅ Create Task with Dates (ejemplo con fechas)
- ✅ Update Task Dates (solo actualizar fechas)
- ✅ Remove Task Due Date (desactivar end_date)

## ⚠️ Consideraciones Importantes

- Todas las fechas se manejan en **UTC**
- Las validaciones se ejecutan en **creación y actualización**
- Los **WebSockets** notifican cambios en fechas
- La **serialización** maneja correctamente los campos opcionales
- El backend está **listo para usar** - no se necesitan más cambios

## ⚠️ Consideraciones Importantes

- Todas las fechas se manejan en UTC
- Las validaciones se ejecutan tanto en creación como en actualización
- Los WebSockets notifican cambios en fechas
- La serialización personalizada maneja correctamente los campos opcionales

## 📮 Documentación de Postman

Los nuevos endpoints han sido agregados a la colección de Postman:

**Ubicación:** `src/postma-collection/enviroment.json`

**Endpoints agregados:**
- ✅ Create Task with Dates
- ✅ Update Task Dates  
- ✅ Remove Task Due Date
- ✅ Filter Tasks by Date Range
- ✅ Filter Overdue Tasks Only
- ✅ Filter Tasks with No Due Date
- ✅ Get Overdue Tasks

**Ver documentación detallada:** `src/postma-collection/ENDPOINTS_FECHAS.md`

### Variables de Environment necesarias:
```json
{
  "base_url": "http://localhost:8080",
  "auth_token": "tu_token_aqui",
  "project_id": "id_del_proyecto",
  "task_id": "id_de_tarea"
}
```
