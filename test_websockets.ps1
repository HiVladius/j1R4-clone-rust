# Script de prueba para WebSockets - Jira Clone Backend
# Ejecuta este script mientras tienes la página websocket_test.html abierta y conectada

$baseUrl = "http://127.0.0.1:3000/api"
$wsTestUrl = "file:///c:/Users/vladm/OneDrive/Escritorio/Dev/jira_clone_backend/websocket_test.html"

Write-Host "🚀 Script de prueba WebSocket - Jira Clone" -ForegroundColor Green
Write-Host "📋 Asegúrate de tener la página de prueba WebSocket abierta y conectada" -ForegroundColor Yellow
Write-Host "🌐 URL de prueba: $wsTestUrl" -ForegroundColor Cyan
Write-Host ""

# Función para hacer requests HTTP
function Invoke-ApiRequest {
    param(
        [string]$Method,
        [string]$Uri,
        [hashtable]$Headers = @{},
        [object]$Body = $null
    )
    
    try {
        $params = @{
            Method = $Method
            Uri = $Uri
            Headers = $Headers
        }
        
        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
            $params.Headers["Content-Type"] = "application/json"
        }
        
        $response = Invoke-RestMethod @params
        return $response
    }
    catch {
        Write-Host "❌ Error en request $Method $Uri : $($_.Exception.Message)" -ForegroundColor Red
        if ($_.Exception.Response) {
            $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
            $responseBody = $reader.ReadToEnd()
            Write-Host "📄 Response body: $responseBody" -ForegroundColor Red
        }
        return $null
    }
}

# 1. Registrar usuario de prueba
Write-Host "1️⃣ Registrando usuario de prueba..." -ForegroundColor Blue
$registerData = @{
    name = "Test User WS"
    email = "testws@example.com"
    password = "password123"
}

$registerResponse = Invoke-ApiRequest -Method "POST" -Uri "$baseUrl/auth/register" -Body $registerData

if (-not $registerResponse) {
    Write-Host "⚠️ El usuario probablemente ya existe, intentando login..." -ForegroundColor Yellow
}

# 2. Hacer login para obtener token
Write-Host "2️⃣ Haciendo login..." -ForegroundColor Blue
$loginData = @{
    email = "testws@example.com"
    password = "password123"
}

$loginResponse = Invoke-ApiRequest -Method "POST" -Uri "$baseUrl/auth/login" -Body $loginData

if (-not $loginResponse -or -not $loginResponse.token) {
    Write-Host "❌ No se pudo obtener el token de autenticación" -ForegroundColor Red
    exit 1
}

$token = $loginResponse.token
$authHeaders = @{
    "Authorization" = "Bearer $token"
}

Write-Host "✅ Login exitoso, token obtenido" -ForegroundColor Green

# 3. Crear un proyecto
Write-Host "3️⃣ Creando proyecto..." -ForegroundColor Blue
$projectData = @{
    name = "Proyecto WebSocket Test"
    description = "Proyecto para probar WebSockets"
    status = "active"
}

$projectResponse = Invoke-ApiRequest -Method "POST" -Uri "$baseUrl/projects" -Headers $authHeaders -Body $projectData

if (-not $projectResponse -or -not $projectResponse.id) {
    Write-Host "❌ No se pudo crear el proyecto" -ForegroundColor Red
    exit 1
}

$projectId = $projectResponse.id
Write-Host "✅ Proyecto creado con ID: $projectId" -ForegroundColor Green

# Pausa para observar
Write-Host ""
Write-Host "⏱️ Pausa 3 segundos para que observes los logs..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# 4. Crear primera tarea (debería emitir TASK_CREATED)
Write-Host "4️⃣ Creando primera tarea (verifica WebSocket)..." -ForegroundColor Blue
$taskData1 = @{
    title = "Tarea WebSocket Test 1"
    description = "Esta tarea debería emitir un evento TASK_CREATED"
    status = "todo"
    priority = "high"
}

$taskResponse1 = Invoke-ApiRequest -Method "POST" -Uri "$baseUrl/projects/$projectId/tasks" -Headers $authHeaders -Body $taskData1

if ($taskResponse1) {
    Write-Host "✅ Tarea 1 creada con ID: $($taskResponse1.id)" -ForegroundColor Green
    Write-Host "🔍 Verifica que aparezca un mensaje TASK_CREATED en el WebSocket" -ForegroundColor Cyan
} else {
    Write-Host "❌ No se pudo crear la tarea 1" -ForegroundColor Red
}

# Pausa
Start-Sleep -Seconds 3

# 5. Crear segunda tarea
Write-Host "5️⃣ Creando segunda tarea..." -ForegroundColor Blue
$taskData2 = @{
    title = "Tarea WebSocket Test 2"
    description = "Esta es otra tarea para probar WebSockets"
    status = "in_progress"
    priority = "medium"
}

$taskResponse2 = Invoke-ApiRequest -Method "POST" -Uri "$baseUrl/projects/$projectId/tasks" -Headers $authHeaders -Body $taskData2

if ($taskResponse2) {
    Write-Host "✅ Tarea 2 creada con ID: $($taskResponse2.id)" -ForegroundColor Green
} else {
    Write-Host "❌ No se pudo crear la tarea 2" -ForegroundColor Red
}

# Pausa
Start-Sleep -Seconds 3

# 6. Actualizar primera tarea (debería emitir TASK_UPDATED)
if ($taskResponse1) {
    Write-Host "6️⃣ Actualizando primera tarea (verifica TASK_UPDATED)..." -ForegroundColor Blue
    $updateData = @{
        title = "Tarea WebSocket Test 1 - ACTUALIZADA"
        status = "in_progress"
        priority = "low"
    }
    
    $updateResponse = Invoke-ApiRequest -Method "PATCH" -Uri "$baseUrl/tasks/$($taskResponse1.id)" -Headers $authHeaders -Body $updateData
    
    if ($updateResponse) {
        Write-Host "✅ Tarea 1 actualizada" -ForegroundColor Green
        Write-Host "🔍 Verifica que aparezca un mensaje TASK_UPDATED en el WebSocket" -ForegroundColor Cyan
    } else {
        Write-Host "❌ No se pudo actualizar la tarea 1" -ForegroundColor Red
    }
}

# Pausa
Start-Sleep -Seconds 3

# 7. Eliminar segunda tarea (debería emitir TASK_DELETED)
if ($taskResponse2) {
    Write-Host "7️⃣ Eliminando segunda tarea (verifica TASK_DELETED)..." -ForegroundColor Blue
    
    $deleteResponse = Invoke-ApiRequest -Method "DELETE" -Uri "$baseUrl/tasks/$($taskResponse2.id)" -Headers $authHeaders
    
    Write-Host "✅ Tarea 2 eliminada" -ForegroundColor Green
    Write-Host "🔍 Verifica que aparezca un mensaje TASK_DELETED en el WebSocket" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "🎉 Prueba completada!" -ForegroundColor Green
Write-Host "📊 Deberías haber visto los siguientes eventos en WebSocket:" -ForegroundColor Yellow
Write-Host "   - TASK_CREATED (2 veces)" -ForegroundColor White
Write-Host "   - TASK_UPDATED (1 vez)" -ForegroundColor White  
Write-Host "   - TASK_DELETED (1 vez)" -ForegroundColor White
Write-Host ""
Write-Host "💡 Si no ves los mensajes, revisa:" -ForegroundColor Cyan
Write-Host "   - Que la página WebSocket esté conectada" -ForegroundColor White
Write-Host "   - Los logs del servidor backend" -ForegroundColor White
Write-Host "   - La consola del navegador" -ForegroundColor White
