import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import * as random from "@pulumi/random";

// Get configuration
const config = new pulumi.Config();
const imageTag = config.require("imageTag");
const imageRegistry = config.get("imageRegistry") || "ghcr.io/zortax/tub-modules";
const ingressHost = config.get("ingressHost") || "tub.zortax.de";
const tlsSecretName = config.get("tlsSecretName") || "tub-modules-tls";
const certManagerIssuer = config.get("certManagerIssuer") || "letsencrypt";
const appReplicas = config.getNumber("appReplicas") || 1;

// Namespace
const namespace = new k8s.core.v1.Namespace("tub-modules", {
    metadata: {
        name: "tub-modules",
    },
});

// Generate random passwords
const postgresPassword = new random.RandomPassword("postgres-password", {
    length: 32,
    special: false, // No special chars to avoid URL encoding issues in DATABASE_URL
});

const scraperAuthKey = new random.RandomPassword("scraper-auth-key", {
    length: 64,
    special: false, // No special chars for easier API usage
});

// ConfigMap for non-sensitive environment variables
const appConfigMap = new k8s.core.v1.ConfigMap("app-config", {
    metadata: {
        name: "app-config",
        namespace: namespace.metadata.name,
    },
    data: {
        DATABASE_HOST: "postgres-service",
        DATABASE_PORT: "5432",
        DATABASE_USER: "postgres",
        DATABASE_NAME: "tub_modules",
        DATABASE_MAX_CONNECTIONS: "5",
        LEPTOS_SITE_ADDR: "0.0.0.0:3000",
        RUST_LOG: "info",
    },
}, { dependsOn: [namespace] });

// Secret for sensitive data
const appSecret = new k8s.core.v1.Secret("app-secret", {
    metadata: {
        name: "app-secret",
        namespace: namespace.metadata.name,
    },
    stringData: {
        DATABASE_PASSWORD: postgresPassword.result,
        DATABASE_URL: pulumi.interpolate`postgres://postgres:${postgresPassword.result}@postgres-service:5432/tub_modules`,
        SCRAPER_AUTH_KEY: scraperAuthKey.result,
    },
}, { dependsOn: [namespace] });

// PostgreSQL PersistentVolumeClaim
const postgresPvc = new k8s.core.v1.PersistentVolumeClaim("postgres-pvc", {
    metadata: {
        name: "postgres-pvc",
        namespace: namespace.metadata.name,
    },
    spec: {
        accessModes: ["ReadWriteOnce"],
        resources: {
            requests: {
                storage: "10Gi",
            },
        },
    },
}, { dependsOn: [namespace] });

// PostgreSQL Deployment
const postgresDeployment = new k8s.apps.v1.Deployment("postgres", {
    metadata: {
        name: "postgres",
        namespace: namespace.metadata.name,
        labels: {
            app: "postgres",
        },
    },
    spec: {
        replicas: 1,
        selector: {
            matchLabels: {
                app: "postgres",
            },
        },
        template: {
            metadata: {
                labels: {
                    app: "postgres",
                },
            },
            spec: {
                containers: [{
                    name: "postgres",
                    image: "postgres:16-alpine",
                    ports: [{
                        containerPort: 5432,
                        name: "postgres",
                    }],
                    env: [
                        {
                            name: "POSTGRES_DB",
                            value: "tub_modules",
                        },
                        {
                            name: "POSTGRES_USER",
                            value: "postgres",
                        },
                        {
                            name: "POSTGRES_PASSWORD",
                            valueFrom: {
                                secretKeyRef: {
                                    name: appSecret.metadata.name,
                                    key: "DATABASE_PASSWORD",
                                },
                            },
                        },
                        {
                            name: "PGDATA",
                            value: "/var/lib/postgresql/data/pgdata",
                        },
                    ],
                    volumeMounts: [{
                        name: "postgres-storage",
                        mountPath: "/var/lib/postgresql/data",
                    }],
                    resources: {
                        requests: {
                            cpu: "100m",
                            memory: "256Mi",
                        },
                        limits: {
                            cpu: "500m",
                            memory: "512Mi",
                        },
                    },
                }],
                volumes: [{
                    name: "postgres-storage",
                    persistentVolumeClaim: {
                        claimName: postgresPvc.metadata.name,
                    },
                }],
            },
        },
    },
}, { dependsOn: [namespace, appSecret, postgresPvc] });

// PostgreSQL Service
const postgresService = new k8s.core.v1.Service("postgres-service", {
    metadata: {
        name: "postgres-service",
        namespace: namespace.metadata.name,
    },
    spec: {
        selector: {
            app: "postgres",
        },
        ports: [{
            port: 5432,
            targetPort: 5432,
            protocol: "TCP",
        }],
        type: "ClusterIP",
    },
}, { dependsOn: [namespace, postgresDeployment] });

// App Deployment
const appDeployment = new k8s.apps.v1.Deployment("app", {
    metadata: {
        name: "app",
        namespace: namespace.metadata.name,
        labels: {
            app: "tub-modules",
        },
    },
    spec: {
        replicas: appReplicas,
        selector: {
            matchLabels: {
                app: "tub-modules",
            },
        },
        template: {
            metadata: {
                labels: {
                    app: "tub-modules",
                },
            },
            spec: {
                containers: [{
                    name: "app",
                    image: `${imageRegistry}:${imageTag}`,
                    ports: [{
                        containerPort: 3000,
                        name: "http",
                    }],
                    envFrom: [
                        {
                            configMapRef: {
                                name: appConfigMap.metadata.name,
                            },
                        },
                        {
                            secretRef: {
                                name: appSecret.metadata.name,
                            },
                        },
                    ],
                    resources: {
                        requests: {
                            cpu: "100m",
                            memory: "128Mi",
                        },
                        limits: {
                            cpu: "1000m",
                            memory: "512Mi",
                        },
                    },
                    livenessProbe: {
                        httpGet: {
                            path: "/",
                            port: 3000,
                        },
                        initialDelaySeconds: 30,
                        periodSeconds: 10,
                    },
                    readinessProbe: {
                        httpGet: {
                            path: "/",
                            port: 3000,
                        },
                        initialDelaySeconds: 10,
                        periodSeconds: 5,
                    },
                }],
            },
        },
    },
}, { dependsOn: [namespace, appConfigMap, appSecret, postgresService] });

// App Service
const appService = new k8s.core.v1.Service("app-service", {
    metadata: {
        name: "app-service",
        namespace: namespace.metadata.name,
    },
    spec: {
        selector: {
            app: "tub-modules",
        },
        ports: [{
            port: 80,
            targetPort: 3000,
            protocol: "TCP",
        }],
        type: "ClusterIP",
    },
}, { dependsOn: [namespace, appDeployment] });

// Ingress
const appIngress = new k8s.networking.v1.Ingress("app-ingress", {
    metadata: {
        name: "app-ingress",
        namespace: namespace.metadata.name,
        annotations: {
            "cert-manager.io/cluster-issuer": certManagerIssuer,
        },
    },
    spec: {
        ingressClassName: "nginx",
        tls: [{
            hosts: [ingressHost],
            secretName: tlsSecretName,
        }],
        rules: [{
            host: ingressHost,
            http: {
                paths: [{
                    path: "/",
                    pathType: "Prefix",
                    backend: {
                        service: {
                            name: appService.metadata.name,
                            port: {
                                number: 80,
                            },
                        },
                    },
                }],
            },
        }],
    },
}, { dependsOn: [namespace, appService] });

// Export important values
export const namespaceName = namespace.metadata.name;
export const postgresPasswordOutput = postgresPassword.result;
export const scraperAuthKeyOutput = scraperAuthKey.result;
export const appServiceName = appService.metadata.name;
export const ingressHostname = ingressHost;
export const ingressUrl = pulumi.interpolate`https://${ingressHost}`;
export const tlsSecret = tlsSecretName;
