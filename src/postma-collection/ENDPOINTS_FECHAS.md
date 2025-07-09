# Endpoints de Fechas - Documentación Postman

## 📋 Nuevos Endpoints Agregados

Los siguientes endpoints han sido agregados a la colección de Postman para manejar fechas en las tareas:

### 1. **Create Task with Dates**
```
POST /api/projects/{{project_id}}/tasks
```

**Body ejemplo:**
```json
{
  "title": "Task with Dates",
  "description": "Task with start and end dates",
  "status": "ToDo",
  "priority": "Medium",
  "start_date": "2025-01-01T09:00:00Z",
  "end_date": "2025-01-15T17:00:00Z",
  "has_due_date": true
}
```

### 2. **Update Task Dates**
```
PATCH /api/tasks/{{task_id}}
```

**Body ejemplo:**
```json
{
  "start_date": "2025-01-02T09:00:00Z",
  "end_date": "2025-01-20T17:00:00Z"
}
```

### 3. **Remove Task Due Date**
```
PATCH /api/tasks/{{task_id}}
```

**Body ejemplo:**
```json
{
  "has_due_date": false,
  "end_date": null
}
```

### 4. **Filter Tasks by Date Range**
```
GET /api/projects/{{project_id}}/tasks/filter?start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z
```

**Query Parameters:**
- `start_date`: Filtrar tareas desde esta fecha
- `end_date`: Filtrar tareas hasta esta fecha

### 5. **Filter Overdue Tasks Only**
```
GET /api/projects/{{project_id}}/tasks/filter?overdue_only=true
```

**Query Parameters:**
- `overdue_only`: Solo devolver tareas vencidas

### 6. **Filter Tasks with No Due Date**
```
GET /api/projects/{{project_id}}/tasks/filter?no_due_date=true
```

**Query Parameters:**
- `no_due_date`: Solo devolver tareas sin fecha de vencimiento

### 7. **Get Overdue Tasks**
```
GET /api/projects/{{project_id}}/tasks/overdue
```

Endpoint dedicado para obtener todas las tareas vencidas de un proyecto.

## 🔧 Configuración de Variables

Asegúrate de tener las siguientes variables configuradas en tu environment de Postman:

- `base_url`: `http://localhost:8080`
- `auth_token`: Token obtenido del login
- `project_id`: ID del proyecto actual
- `task_id`: ID de la tarea para operaciones específicas

## 📝 Ejemplos de Uso Completos

### Escenario 1: Crear tarea con deadline
1. Ejecutar "Create Task with Dates"
2. Verificar que `task_id` se guarde automáticamente
3. Usar "Get Task by ID" para verificar la creación

### Escenario 2: Actualizar fechas de tarea existente
1. Crear una tarea básica con "Create Task"
2. Usar "Update Task Dates" para agregar fechas
3. Verificar cambios con "Get Task by ID"

### Escenario 3: Remover fecha de vencimiento
1. Tener una tarea con fecha de vencimiento
2. Ejecutar "Remove Task Due Date"
3. Verificar que `has_due_date` sea `false` y `end_date` sea `null`

### Escenario 4: Filtrar tareas por criterios de fecha
1. Crear varias tareas con diferentes fechas
2. Usar los diferentes endpoints de filtrado
3. Comparar resultados según los criterios aplicados

## ⚡ Scripts de Test Automatizados

Los endpoints incluyen scripts de test que:

- ✅ Guardan automáticamente el `task_id` en variables de environment
- ✅ Verifican códigos de respuesta esperados
- ✅ Validan estructura de respuesta JSON

## 🎯 Próximos Pasos

1. **Importar la colección actualizada** en Postman
2. **Configurar las variables** de environment
3. **Ejecutar los tests** en orden secuencial
4. **Validar** que todos los endpoints funcionen correctamente

## 📚 Referencia Rápida

| Endpoint | Method | Propósito |
|----------|--------|-----------|
| `/tasks` | POST | Crear tarea con fechas |
| `/tasks/{id}` | PATCH | Actualizar fechas |
| `/tasks/filter` | GET | Filtrar por fecha |
| `/tasks/overdue` | GET | Obtener vencidas |

Todos los endpoints requieren autenticación via Bearer token.
