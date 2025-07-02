# Script simple de prueba WebSocket
$baseUrl = "http://127.0.0.1:3000/api"

Write-Host "Iniciando prueba WebSocket..." -ForegroundColor Green

# Registrar usuario
Write-Host "1. Registrando usuario..." -ForegroundColor Blue
$registerBody = @{
    name = "Test User WS"
    email = "testws@example.com"
    password = "password123"
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/auth/register" -Method POST -Body $registerBody -ContentType "application/json"
    Write-Host "Usuario registrado exitosamente" -ForegroundColor Green
} catch {
    Write-Host "Usuario ya existe, continuando..." -ForegroundColor Yellow
}

# Login
Write-Host "2. Haciendo login..." -ForegroundColor Blue
$loginBody = @{
    email = "testws@example.com"
    password = "password123"
} | ConvertTo-Json

$loginResponse = Invoke-RestMethod -Uri "$baseUrl/auth/login" -Method POST -Body $loginBody -ContentType "application/json"
$token = $loginResponse.token
$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

Write-Host "Login exitoso" -ForegroundColor Green

# Crear proyecto
Write-Host "3. Creando proyecto..." -ForegroundColor Blue
$projectBody = @{
    name = "Proyecto WebSocket Test"
    description = "Proyecto para probar WebSockets"
    status = "active"
} | ConvertTo-Json

$projectResponse = Invoke-RestMethod -Uri "$baseUrl/projects" -Method POST -Body $projectBody -Headers $headers
$projectId = $projectResponse.id
Write-Host "Proyecto creado: $projectId" -ForegroundColor Green

# Crear tarea 1
Write-Host "4. Creando tarea 1 (verifica TASK_CREATED en WebSocket)..." -ForegroundColor Blue
$taskBody1 = @{
    title = "Tarea WebSocket Test 1"
    description = "Esta tarea deberia emitir TASK_CREATED"
    status = "todo"
    priority = "high"
} | ConvertTo-Json

$taskResponse1 = Invoke-RestMethod -Uri "$baseUrl/projects/$projectId/tasks" -Method POST -Body $taskBody1 -Headers $headers
Write-Host "Tarea 1 creada: $($taskResponse1.id)" -ForegroundColor Green

Start-Sleep -Seconds 2

# Actualizar tarea
Write-Host "5. Actualizando tarea 1 (verifica TASK_UPDATED)..." -ForegroundColor Blue
$updateBody = @{
    title = "Tarea WebSocket Test 1 - ACTUALIZADA"
    status = "in_progress"
} | ConvertTo-Json

$updateResponse = Invoke-RestMethod -Uri "$baseUrl/tasks/$($taskResponse1.id)" -Method PATCH -Body $updateBody -Headers $headers
Write-Host "Tarea actualizada" -ForegroundColor Green

Start-Sleep -Seconds 2

# Crear tarea 2
Write-Host "6. Creando tarea 2..." -ForegroundColor Blue
$taskBody2 = @{
    title = "Tarea WebSocket Test 2"
    description = "Segunda tarea para prueba"
    status = "todo"
    priority = "medium"
} | ConvertTo-Json

$taskResponse2 = Invoke-RestMethod -Uri "$baseUrl/projects/$projectId/tasks" -Method POST -Body $taskBody2 -Headers $headers
Write-Host "Tarea 2 creada: $($taskResponse2.id)" -ForegroundColor Green

Start-Sleep -Seconds 2

# Eliminar tarea 2
Write-Host "7. Eliminando tarea 2 (verifica TASK_DELETED)..." -ForegroundColor Blue
Invoke-RestMethod -Uri "$baseUrl/tasks/$($taskResponse2.id)" -Method DELETE -Headers $headers
Write-Host "Tarea 2 eliminada" -ForegroundColor Green

Write-Host ""
Write-Host "Prueba completada! Verifica los mensajes WebSocket en tu navegador" -ForegroundColor Green
Write-Host "Deberias ver: TASK_CREATED (2x), TASK_UPDATED (1x), TASK_DELETED (1x)" -ForegroundColor Cyan
