# MOGWAI: Stress Test Toolkit
This project provides a lightweight, secure, and scalable stress testing toolkit designed for continuous integration and development (CI/CD) pipelines in cloud environments. It can be used to push services and systems to their limits, ensuring maximum uptime and resilience under load.

The project is built with three core components: frontend, controller, engine. The controller is a python application with the goal of managing communication between the external frontend and internal engine. The engine is a Rust based stress-testing application, deployed internally on the cluster. The frontend is a standalone application that accesses an Ingress service to communicate with the controller.

## Prerequisites

### Rust
Rust is the language used in the project, you can install it [here](https://www.rust-lang.org/tools/install).

### Python
Python is used to build the controller application. Required dependencies are listed under:
```bash
/controller/requirements.txt
```

### Docker
Docker is required to build, run, and push the Docker image for the project. Docker is used to containerize our applications for both local and Kubernetes integration. When edits are made to the source code, a new image will need to be generated and pushed to the registry (GitHub Packages for our project) in order to be pulled inside the cluster. 

A complete working prototype will be set as the repository package (public) and development images should be kept private.

### Kubernetes (Minikube for local development)
You'll need access to a Kubernetes cluster to run integration tests. Right now Minikube has been tested to work, you can find install instructions [here](https://minikube.sigs.k8s.io/docs/) for local development. A more accurate, professional cluster (with test apps) will be set up on testing machine.

### GitHub Personal Access Token (PAT)
You can generate a PAT from the Developer setting tab in GitHub. Ensure you grant package permissions and repo permissions. This will be needed for package pushing and pulling.

You can set your token as an environment variable as well with:
```bash
    export GITHUB_TOKEN=your_personal_access_token
    echo $GITHUB_TOKEN #To test if it is set 
```
**NOTE:**This will be temporary, for persistance, edit the bashrc file on your system to include the token (recommended).

After creating and storing your PAT, you can login to GitHub packages with this command:
```bash
echo GITHUB_TOKEN | docker login ghcr.io -u <your_github_username> --password-stdin
```

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

Once running ensure to enable all necessary addons (metrics-server and ingress currently):

```bash
minikube addons enable metrics-server
minikube addons enable ingress
```
### 3. **Create Registry Secret**
You will need a registry secret in order to pull an image from GitHub Packages (not needed if pulling the public image) into the Kubernetes cluster:

```bash
    kubectl create secret docker-registry github-registry-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --docker-email=<your-email>
```

### 3a. **Run engine as Rust service** 
To run the Rust code as a local service:
```bash
    cargo run
```
This will expose port 8080 to which you can make curl POST requests for testing, for example:
``` bash
curl -X POST http://localhost:8080/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "load": 75, "fork": false}'
```

### 3b. **Run engine as Docker service**
First build the image (see Docker section). You can then run the image with:
```bash
docker run -p <external-port>:<internal-port> <image-name>
```
This will expose the Docker app as a service with which you can use the same ```curl``` method as before, for example:
``` bash
curl -X POST http://localhost:8080/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "load": 75, "fork": false}'
```
### Pushing/Pulling Packages to GitHub Packages

To build an image, ensure a Dockerfile is present. Then run:
```bash
docker build -t <image-name> .
# If you get platform error, see buildx documentation for specifying your platform
```
After verifying this image works, you can then tag it for pushing:
```bash
docker tag <image-name> ghcr.io/<github-username>/<image-name>:<tag> 
# The tag part is optional but good for version control
```
You can verify if the tag worked with:
```bash
docker images | grep ghcr.io
```
You can then push to the github registry with:
```bash
docker push ghcr.io/<github-username>/<image-name>:<tag>
```
After pushing you should see the package in either your personal packages or in the repo if pushing the production build.

Now you can pull with ```docker pull <image-name>``` or test for successful deployment in Kubernetes. NOTE: Ensure you are authenticated (github token login) if you are pulling a private repo.

### 3c. **Run Engine Deployment in Kubernetes**

In the ```kubernetes``` folder are some YAMLs to be used. Ensure minikube is running with ```minikube status```. Then once ready, apply the files:
```bash
kubectl apply -f engine-deployment.yaml
kubectl apply -f engine-service.yaml
```

Confirm the status of these operations with:
```bash
kubectl get deployments
kubectl get pods
kubectl get svc
```
These should display the appropriate pods/service.

Now for running the controller integration, use ```kubectl apply -f <file>``` on ```controller-deployment```, ```controller-service```, and ```controller-ingress```. Ensure the creation of the controller pod, service, and ingress was successful.

At this point, the controller now has a reachable hostname at ```controller.local```. You can test this endpoint with either a frontend that sends requests to this hostname or ```curl```, for example:
```bash
curl -X POST http://controller.local/cpu-stress -H "Content-Type: application/json" -d '{"intensity": 4, "duration": 10, "load": 100}'
```

***WORK IN PROGRESS***