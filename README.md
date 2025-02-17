# MOGWAI: Stress Test Toolkit
This project provides a lightweight, secure, and scalable stress testing toolkit designed for continuous integration and development (CI/CD) pipelines in cloud environments. It can be used to push services and systems to their limits, ensuring maximum uptime and resilience under load.

The project is built using Rust and is designed to run in Kubernetes/Docker environments.

## Prerequisites

### Rust
Rust is the language used in the project, you can install it [here](https://www.rust-lang.org/tools/install)

### Docker
Docker is required to build, run, and push the Docker image for the project.

### Kubernetes (Minikube for local development)
You'll need access to a Kubernetes cluster to run the tests. Right now Minikube has been tested to work, you can find install instructions [here](https://minikube.sigs.k8s.io/docs/)

### GitHub Personal Access Token (PAT)
You can generate a PAT from the Developer setting tab in GitHub. Ensure you grant package permissions and repo permissions.

## Set Up Instructions
### 1. **Clone the Repository**

```bash
   git clone https://github.com/<your-username>/mogwai.git
```
### 2. **Create Registry Secret**
You will need a registry secret in order to pull the image from GitHub:
```bash
    kubectl create secret docker-registry github-registry-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --docker-email=<your-email>
```
### 3. **Run Test in Kubernetes**
Create a YAML (or use the one provided) as:
```yaml
    apiVersion: batch/v1
kind: Job
metadata:
  name: rust-stress-test
spec:
  template:
    spec:
      containers:
      - name: stress-test
        image: ghcr.io/dman7351/mogwai:latest 
        args: ["4", "10"]  # Runs 4 CPU threads for 10 seconds
      imagePullSecrets:
      - name: github-registry-secret  # The secret we created
      restartPolicy: Never # Just means the container will not be restarted
```

Then apply the job as:
```bash
    kubectl apply -f stress-test-job.yaml
```

Then to check the status the job as:
```bash
    kubectl get jobs
```

Then to view the logs of the job use:
```bash
    kubectl logs -l job-name=rust-stress-test
```

Once finished, delete the job (you cannot rerun an already created job):
```bash
    kubectl delete job rust-stress-test
```


Will update once more jobs are added...
