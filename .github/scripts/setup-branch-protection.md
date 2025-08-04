# Configuración de Branch Protection Rules

Para configurar las reglas de protección de la rama `main`, sigue estos pasos:

## Opción 1: Via GitHub CLI (Recomendado)

```bash
# Instalar GitHub CLI si no lo tienes
# Windows: winget install GitHub.cli
# macOS: brew install gh
# Linux: sudo apt install gh

# Autenticarse
gh auth login

# Configurar branch protection para main
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Test Suite","Security Audit"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":false}' \
  --field restrictions=null
```

## Opción 2: Via GitHub Web Interface

1. Ve a tu repositorio en GitHub
2. Settings → Branches
3. Click "Add rule" junto a "Branch protection rules"
4. Branch name pattern: `main`
5. Configura las siguientes opciones:

### ✅ Require a pull request before merging
- [x] Require approvals: 1
- [x] Dismiss stale pull request approvals when new commits are pushed
- [ ] Require review from code owners (opcional)

### ✅ Require status checks to pass before merging
- [x] Require branches to be up to date before merging
- Seleccionar status checks:
  - `Test Suite`
  - `Security Audit`
  - `coverage` (opcional)

### ✅ Require conversation resolution before merging
- [x] Require conversation resolution before merging

### ✅ Otras opciones recomendadas
- [x] Include administrators
- [x] Allow force pushes (solo si es necesario)
- [x] Allow deletions (desmarcar para mayor seguridad)

## Verificación

Después de configurar, puedes verificar con:

```bash
gh api repos/:owner/:repo/branches/main/protection
```

## Beneficios de estas reglas

1. **Code Review obligatorio**: Previene cambios directos a main
2. **CI/CD gates**: Asegura que tests y auditorías pasen antes del merge
3. **Conversaciones resueltas**: Garantiza que todos los comentarios sean addressed
4. **Administradores incluidos**: Aplica las reglas a todos, incluyendo admins
