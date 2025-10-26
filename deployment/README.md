# TUB Modules Kubernetes Deployment

This directory contains the Pulumi infrastructure code for deploying the TUB Modules application to Kubernetes.

## Prerequisites

- [Pulumi CLI](https://www.pulumi.com/docs/get-started/install/) installed
- [kubectl](https://kubernetes.io/docs/tasks/tools/) configured to access your Kubernetes cluster
- [Node.js](https://nodejs.org/) and npm installed

## Project Structure

- `index.ts` - Main Pulumi program defining all Kubernetes resources
- `Pulumi.yaml` - Pulumi project configuration
- `Pulumi.dev.yaml` - Stack configuration for the dev environment

## Resources Created

This deployment creates the following Kubernetes resources in the `tub-modules` namespace:

### Infrastructure
- **Namespace**: `tub-modules` - Isolated namespace for all resources
- **ConfigMap**: `app-config` - Non-sensitive environment variables
- **Secret**: `app-secret` - Sensitive data (database password, scraper auth key)

### PostgreSQL Database
- **PersistentVolumeClaim**: `postgres-pvc` - 10Gi storage for PostgreSQL data
- **Deployment**: `postgres` - PostgreSQL 16 Alpine container
- **Service**: `postgres-service` - ClusterIP service for database access

### Application
- **Deployment**: `app` - 2 replicas of the TUB Modules application
- **Service**: `app-service` - ClusterIP service on port 80
- **Ingress**: `app-ingress` - Nginx ingress with TLS/HTTPS support via cert-manager

## Security Features

### Random Password Generation
All sensitive credentials are automatically generated using Pulumi's RandomPassword resource:

- **PostgreSQL Password**: 32 characters (alphanumeric only to avoid URL encoding issues)
- **Scraper Auth Key**: 64 characters (alphanumeric only for API compatibility)

These passwords are:
- Generated once during initial deployment
- Stored in Pulumi state
- Automatically injected into Kubernetes Secrets
- Never stored in plain text in the code

### Kubernetes Security
- All sensitive data stored in Kubernetes Secrets
- Environment variables injected from ConfigMaps and Secrets
- PostgreSQL data persisted in a PersistentVolume

## Configuration

### Stack Configuration (Empty Passphrase)

This project uses an **empty passphrase** for the Pulumi stack, making it suitable for public repositories and CI/CD pipelines. All secrets (passwords, auth keys) are generated at deployment time using Pulumi's RandomPassword resource and stored in the Pulumi state.

To work with this stack, set the environment variable:
```bash
export PULUMI_CONFIG_PASSPHRASE=""
```

### Available Configuration Options

| Config Key | Required | Default | Description |
|------------|----------|---------|-------------|
| `imageTag` | Yes | - | Docker image tag to deploy (e.g., "0.1.0") |
| `imageRegistry` | No | `ghcr.io/zortax/tub-modules` | Container registry URL |
| `ingressHost` | No | `tub.zortax.de` | Hostname for the ingress |
| `tlsSecretName` | No | `tub-modules-tls` | Name of the TLS secret (managed by cert-manager) |
| `certManagerIssuer` | No | `letsencrypt` | cert-manager ClusterIssuer name for TLS certificates |
| `appReplicas` | No | `1` | Number of app pod replicas to run |

### Setting Configuration

```bash
# Required: Set the image tag
export PULUMI_CONFIG_PASSPHRASE=""
pulumi config set imageTag "0.1.0"

# Optional: Set custom ingress host
pulumi config set ingressHost "modules.example.com"

# Optional: Use a different registry
pulumi config set imageRegistry "ghcr.io/myorg/tub-modules"
```

## Deployment Instructions

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Stack

```bash
export PULUMI_CONFIG_PASSPHRASE=""
pulumi stack select dev  # or create a new stack
pulumi config set imageTag "0.1.0"
```

### 3. Preview Changes

```bash
pulumi preview
```

### 4. Deploy

```bash
pulumi up
```

### 5. View Outputs

After deployment, Pulumi will output important information:

```bash
pulumi stack output
```

Outputs include:
- `namespaceName` - The Kubernetes namespace
- `postgresPasswordOutput` - Generated PostgreSQL password (marked as secret)
- `scraperAuthKeyOutput` - Generated scraper auth key (marked as secret)
- `appServiceName` - Name of the app service
- `ingressHostname` - Configured ingress hostname
- `ingressUrl` - Full HTTPS URL to access the application
- `tlsSecret` - Name of the TLS secret managed by cert-manager

### 6. Access Generated Secrets

To retrieve the generated passwords:

```bash
# Get all stack outputs (secrets will be marked as [secret])
pulumi stack output

# Get a specific secret value
pulumi stack output postgresPasswordOutput --show-secrets
pulumi stack output scraperAuthKeyOutput --show-secrets
```

## Updating the Deployment

### Update Image Tag

```bash
export PULUMI_CONFIG_PASSPHRASE=""
pulumi config set imageTag "0.2.0"
pulumi up
```

### Update Ingress Host

```bash
pulumi config set ingressHost "new-host.example.com"
pulumi up
```

## Resource Dependencies

The deployment ensures proper resource ordering through `dependsOn`:

1. **Namespace** is created first
2. **ConfigMap** and **Secret** depend on the namespace
3. **PostgreSQL PVC** depends on the namespace
4. **PostgreSQL Deployment** depends on namespace, secret, and PVC
5. **PostgreSQL Service** depends on namespace and PostgreSQL deployment
6. **App Deployment** depends on namespace, ConfigMap, Secret, and PostgreSQL Service
7. **App Service** depends on namespace and App Deployment
8. **Ingress** depends on namespace and App Service

This ensures that:
- The database is ready before the app starts
- ConfigMaps and Secrets exist before deployments reference them
- Services are created after their corresponding deployments

## Environment Variables

### ConfigMap (Non-sensitive)
- `DATABASE_HOST=postgres-service`
- `DATABASE_PORT=5432`
- `DATABASE_USER=postgres`
- `DATABASE_NAME=tub_modules`
- `DATABASE_MAX_CONNECTIONS=5`
- `LEPTOS_SITE_ADDR=0.0.0.0:3000`
- `RUST_LOG=info`

### Secret (Sensitive)
- `DATABASE_PASSWORD` (auto-generated, 32 chars)
- `DATABASE_URL` (auto-generated connection string)
- `SCRAPER_AUTH_KEY` (auto-generated, 64 chars)

## Troubleshooting

### Check Pod Status

```bash
kubectl get pods -n tub-modules
```

### View Pod Logs

```bash
# App logs
kubectl logs -n tub-modules -l app=tub-modules

# PostgreSQL logs
kubectl logs -n tub-modules -l app=postgres
```

### Check Service Endpoints

```bash
kubectl get endpoints -n tub-modules
```

### Describe Resources

```bash
kubectl describe deployment app -n tub-modules
kubectl describe deployment postgres -n tub-modules
```

### Check Ingress

```bash
kubectl get ingress -n tub-modules
kubectl describe ingress app-ingress -n tub-modules
```

## Cleanup

To destroy all resources:

```bash
export PULUMI_CONFIG_PASSPHRASE=""
pulumi destroy
```

## Notes

- The PostgreSQL deployment uses a single replica with persistent storage
- The app deployment replica count is configurable (default: 1, can be increased for high availability)
- Health checks (liveness and readiness probes) are configured for the app
- **Prerequisites**: The deployment assumes the following are already installed in the cluster:
  - Nginx ingress controller
  - cert-manager with a configured ClusterIssuer (default: `letsencrypt`)
- TLS certificates are automatically provisioned by cert-manager
- All resources are created in the `tub-modules` namespace for isolation

## CI/CD Integration

Since this project uses an empty passphrase, it can be easily integrated into CI/CD pipelines:

```bash
# In your CI/CD pipeline
export PULUMI_CONFIG_PASSPHRASE=""
npm install
pulumi preview  # or pulumi up --yes for auto-approval
```

## Accessing the Application

After deployment, the application will be available at the configured ingress host with HTTPS support.

### Setup Steps

1. **Configure DNS**: Point your domain (e.g., `tub.zortax.de`) to your ingress controller's external IP
   ```bash
   # Get the ingress controller IP
   kubectl get svc -n ingress-nginx ingress-nginx-controller
   ```

2. **Wait for TLS certificate**: cert-manager will automatically request and configure a TLS certificate from Let's Encrypt
   ```bash
   # Check certificate status
   kubectl get certificate -n tub-modules
   kubectl describe certificate tub-modules-tls -n tub-modules
   ```

3. **Access the application**: Once the certificate is ready, access your application at:
   ```
   https://tub.zortax.de/
   ```
   (Or whatever hostname you configured in `ingressHost`)

### Troubleshooting TLS

If the certificate is not being issued:

```bash
# Check certificate status
kubectl get certificate -n tub-modules

# Check certificate request
kubectl get certificaterequest -n tub-modules

# Check cert-manager logs
kubectl logs -n cert-manager -l app=cert-manager

# Check the ClusterIssuer exists
kubectl get clusterissuer
```
