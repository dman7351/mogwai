# MOGWAI: Stress Test Toolkit
This project provides a lightweight, secure, and scalable stress testing toolkit designed for continuous integration and development (CI/CD) pipelines in cloud environments. It can be used to push services and systems to their limits, ensuring maximum uptime and resilience under load.

The project is built using Rust and is designed to run in Kubernetes/Docker environments.

## Prerequisites

### Rust
Rust is the language used in the project, you can install it [here](https://www.rust-lang.org/tools/install)

### Docker
Docker is required to build, run, and push the Docker image for the project. Docker is used to containerize our application for both local and Kubernetes integration. When edits are made to the source code, a new image will need to be generated and pushed to a registry (GitHub Packages for our project). The latest image will be pushed as the repo package (keep unfinished builds private).

### Kubernetes (Minikube for local development)
You'll need access to a Kubernetes cluster to run integration tests. Right now Minikube has been tested to work, you can find install instructions [here](https://minikube.sigs.k8s.io/docs/) for local development. A more accurate, professional cluster (with test apps) will be set up on testing machine.

### GitHub Personal Access Token (PAT)
You can generate a PAT from the Developer setting tab in GitHub. Ensure you grant package permissions and repo permissions. This will be needed for package pushing and pulling.

## Set Up Instructions
### 1. **Clone the Repository**

```bash
   git clone https://github.com/<your-username>/mogwai.git
```
### 2. **Start Minikube**
    With Minikube installed, run:
```bash
    minikube start
```
### 3. **Create Registry Secret**
You will need a registry secret in order to pull an image from GitHub Packages (not needed if pulling the public image):
```bash
    kubectl create secret docker-registry github-registry-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --docker-email=<your-email>
```
You can set your token as an environment variable as well with:
```bash
    export GITHUB_TOKEN=your_personal_access_token
    echo $GITHUB_TOKEN #To test if it is set 
```
### 3a. **Run Test as Rust executable** 
To run the test as a Rust executable:
```bash
    cargo run --release --<thread count> <duration in seconds>
```

### 3b. **Run Test as a Docker container**
To first build the image use:
```bash
    docker build -t <image name> # i.e rust-stress-test
```
To then run the built image use:
```bash
    docker run --rm <image name> <thread count> <duration in seconds>
```

### Pushing/Pulling Packages to GitHub Packages

After creating and storing your PAT, you can login to GitHub packages with this command:
```bash
echo GITHUB_TOKEN | docker login ghcr.io -u <your_github_username> --password-stdin
```
Then tag your container image (the docker container you built) with:
```bash
docker tag <image name> ghcr.io/<your_github_username>/mogwai:latest
```

You can verify if the tag worked with:
```bash
docker images | grep ghcr.io
```

To push the image run:
```bash
docker push ghcr.io/<your_github_username>/mogwai:latest
```
After pushing you should see the package in either your personal packages or in the repo if pushing the product build.

To pull the image run:
```bash
docker pull ghcr.io/<github_username>/mogwai:latest
```
Ensure you are authenticated (github token login) if you are pulling a private repo.

### 3c. **Run Test in Kubernetes**
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
        image: ghcr.io/dman7351/mogwai:latest #This is the public image tied to the project repo (change this for local testing to match your private image)
        args: ["4", "10"]  # Runs 4 CPU threads for 10 seconds
      imagePullSecrets:
      - name: github-registry-secret  # The secret we created
      restartPolicy: Never # Just means the container will not be restarted
```

Then apply the job as:
```bash
    kubectl apply -f stress-test-job.yaml
```

To check the status the job use:
```bash
    kubectl get jobs
```

To view the logs of the job use:
```bash
    kubectl logs -l job-name=<job name> #i.e rust-stress-test
```

Once finished, delete the job (you cannot rerun an already created job):
```bash
    kubectl delete job rust-stress-test
```


Will update once more tests are added...
