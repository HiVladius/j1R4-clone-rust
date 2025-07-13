# API de Rangos de Fechas para Tareas

Esta documentación describe los endpoints para gestionar fechas de inicio y fin de tareas sin modificar el modelo principal de Task.

## Endpoints

### 1. Establecer/Actualizar Rango de Fechas para una Tarea
**POST** `/api/tasks/{task_id}/date-range`

Crea o actualiza el rango de fechas para una tarea específica.

#### Parámetros
- `task_id`: ID de la tarea (String)

#### Cuerpo de la Solicitud
```json
{
  "start_date": "2024-01-15T09:00:00Z",  // Opcional: Fecha de inicio
  "end_date": "2024-01-30T17:00:00Z"     // Opcional: Fecha de fin
}
```

#### Respuesta
```json
{
  "task_id": "507f1f77bcf86cd799439011",
  "start_date": "2024-01-15T09:00:00Z",
  "end_date": "2024-01-30T17:00:00Z"
}
```

### 2. Obtener Rango de Fechas de una Tarea
**GET** `/api/tasks/{task_id}/date-range`

Obtiene el rango de fechas asociado a una tarea.

#### Parámetros
- `task_id`: ID de la tarea (String)

#### Respuesta
```json
{
  "task_id": "507f1f77bcf86cd799439011",
  "start_date": "2024-01-15T09:00:00Z",
  "end_date": "2024-01-30T17:00:00Z"
}
```

Si no hay rango de fechas asociado:
```json
null
```

### 3. Actualizar Parcialmente Rango de Fechas de una Tarea
**PATCH** `/api/tasks/{task_id}/date-range`

Actualiza parcialmente el rango de fechas de una tarea existente. Permite actualizar solo la fecha de inicio, solo la fecha de fin, o ambas.

#### Parámetros
- `task_id`: ID de la tarea (String)

#### Cuerpo de la Solicitud
```json
{
  "start_date": "2024-01-20T10:00:00Z",  // Opcional: Nueva fecha de inicio
  "end_date": "2024-02-05T16:00:00Z"     // Opcional: Nueva fecha de fin
}
```

**Ejemplos de uso:**

Actualizar solo la fecha de inicio:
```json
{
  "start_date": "2024-01-20T10:00:00Z"
}
```

Actualizar solo la fecha de fin:
```json
{
  "end_date": "2024-02-05T16:00:00Z"
}
```

Eliminar una fecha específica (establecer como null):
```json
{
  "start_date": null
}
```

#### Respuesta
```json
{
  "task_id": "507f1f77bcf86cd799439011",
  "start_date": "2024-01-20T10:00:00Z",
  "end_date": "2024-02-05T16:00:00Z"
}
```

#### Errores Específicos
- **404 Not Found**: Si no existe un rango de fechas para la tarea especificada
- **400 Bad Request**: Si no se proporciona ningún campo para actualizar

### 4. Eliminar Rango de Fechas de una Tarea
**DELETE** `/api/tasks/{task_id}/date-range`

Elimina el rango de fechas asociado a una tarea.

#### Parámetros
- `task_id`: ID de la tarea (String)

#### Respuesta
Status: `204 No Content`

### 5. Obtener Todos los Rangos de Fechas de un Proyecto
**GET** `/api/projects/{project_id}/date-ranges`

Obtiene todos los rangos de fechas de las tareas pertenecientes a un proyecto.

#### Parámetros
- `project_id`: ID del proyecto (String)

#### Respuesta
```json
[
  {
    "task_id": "507f1f77bcf86cd799439011",
    "start_date": "2024-01-15T09:00:00Z",
    "end_date": "2024-01-30T17:00:00Z"
  },
  {
    "task_id": "507f1f77bcf86cd799439012",
    "start_date": "2024-02-01T09:00:00Z",
    "end_date": "2024-02-15T17:00:00Z"
  }
]
```

### 6. Obtener Tarea Completa con Rango de Fechas
**GET** `/api/tasks/{task_id}/full`

Obtiene la información completa de una tarea incluyendo su rango de fechas.

#### Parámetros
- `task_id`: ID de la tarea (String)

#### Respuesta
```json
{
  "task": {
    "_id": "507f1f77bcf86cd799439011",
    "project_id": "507f1f77bcf86cd799439010",
    "title": "Implementar autenticación",
    "description": "Desarrollar sistema de login y registro",
    "status": "InProgress",
    "priority": "High",
    "assignee_id": "507f1f77bcf86cd799439009",
    "reporter_id": "507f1f77bcf86cd799439008",
    "created_at": "2024-01-01T10:00:00Z",
    "updated_at": "2024-01-10T14:30:00Z"
  },
  "date_range": {
    "task_id": "507f1f77bcf86cd799439011",
    "start_date": "2024-01-15T09:00:00Z",
    "end_date": "2024-01-30T17:00:00Z"
  }
}
```

## Características Importantes

### 1. Separación de Modelos
- Los rangos de fechas se almacenan en una colección separada (`task_date_ranges`)
- No afecta las tareas existentes ni el modelo principal de Task
- Permite flexibilidad para futuras funcionalidades

### 2. Validación de Permisos
- Todos los endpoints verifican que el usuario tenga acceso al proyecto de la tarea
- Se reutiliza el sistema de permisos existente

### 3. Fechas Opcionales
- Tanto `start_date` como `end_date` son opcionales
- Permite casos donde solo se conoce una fecha (inicio o fin)

### 4. Compatibilidad
- No requiere migración de datos existentes
- Las tareas sin rangos de fechas continúan funcionando normalmente
- Retrocompatible con el sistema existente

## Ejemplo de Uso desde el Frontend

### Crear/Actualizar Rango de Fechas
```javascript
const setTaskDateRange = async (taskId, startDate, endDate) => {
  const response = await fetch(`/api/tasks/${taskId}/date-range`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      start_date: startDate,
      end_date: endDate
    })
  });
  
  return response.json();
};
```

### Actualizar Parcialmente Rango de Fechas
```javascript
const updateTaskDateRange = async (taskId, updates) => {
  const response = await fetch(`/api/tasks/${taskId}/date-range`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify(updates)
  });
  
  return response.json();
};

// Ejemplos de uso:
// Actualizar solo fecha de inicio
await updateTaskDateRange(taskId, { start_date: "2024-01-20T10:00:00Z" });

// Actualizar solo fecha de fin
await updateTaskDateRange(taskId, { end_date: "2024-02-05T16:00:00Z" });

// Eliminar fecha de inicio (establecer como null)
await updateTaskDateRange(taskId, { start_date: null });

// Actualizar ambas fechas
await updateTaskDateRange(taskId, { 
  start_date: "2024-01-20T10:00:00Z",
  end_date: "2024-02-05T16:00:00Z"
});
```

### Obtener Tarea con Fechas
```javascript
const getTaskWithDates = async (taskId) => {
  const response = await fetch(`/api/tasks/${taskId}/full`, {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  return response.json();
};
```

### Obtener Todas las Fechas de un Proyecto
```javascript
const getProjectDateRanges = async (projectId) => {
  const response = await fetch(`/api/projects/${projectId}/date-ranges`, {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  return response.json();
};
```

## Estructura de la Base de Datos

### Colección: `task_date_ranges`
```json
{
  "_id": ObjectId("..."),
  "task_id": ObjectId("507f1f77bcf86cd799439011"),
  "start_date": ISODate("2024-01-15T09:00:00Z"),
  "end_date": ISODate("2024-01-30T17:00:00Z")
}
```

### Índices Recomendados
- `task_id`: Índice único para búsquedas rápidas por tarea
- `start_date, end_date`: Índices compuestos para consultas de rango temporal

## Manejo de Errores

### Errores Comunes
- **400 Bad Request**: ID de tarea inválido
- **401 Unauthorized**: Token de autenticación inválido
- **403 Forbidden**: Sin permisos para acceder a la tarea/proyecto
- **404 Not Found**: Tarea no encontrada
- **500 Internal Server Error**: Error interno del servidor

### Respuestas de Error
```json
{
  "error": "ID de tarea inválido",
  "message": "El ID proporcionado no es un ObjectId válido"
}
```
