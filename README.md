# MOGWAI: Stress Test Toolkit *IN DEVELOPMENT*
This project provides a lightweight, secure, and scalable stress testing toolkit designed for continuous integration and development (CI/CD) pipelines in cloud environments. It can be used to push services and systems to their limits, ensuring maximum uptime and resilience under load.

The project is built with three core components: frontend, controller, engine.

### ENGINE ###
The engine is a REST API enabled application that routes requests to the appropriate stress-testing module. It currently supports 3 tests: cpu, memory, and disk I/O. It also has a task registry to keep track of running tasks and stop them (registry is scoped to per engine instance).

### CONTROLLER ###
The controller is a REST API enabled application that can spawn/remove engine pods in the cluster and route requests to their specific pod.
It also adapts the task listing/stopping for node specification (see endpoints.md).

### GUI/CLI ###
The GUI/CLI are local components that connect to a user-specified URL for request sending.

## Prerequisites

### Rust
Rust is the language used in the project, you can install it [here](https://www.rust-lang.org/tools/install).

### Docker
Docker is required to build, run, and push the Docker image for the project. Docker is used to containerize our applications for both local and Kubernetes integration. When edits are made to the source code, a new image will need to be generated and pushed to the registry (GitHub Packages for our project) in order to be pulled inside the cluster. 

A complete working prototype will be set as the repository package (public) and development images should be kept private.

### Kubernetes (Minikube for local development)
You'll need access to a Kubernetes cluster to run integration tests. Right now Minikube has been tested to work, you can find install instructions [here](https://minikube.sigs.k8s.io/docs/) for local development. 

### GitHub Personal Access Token (PAT)
You can generate a PAT from the Developer setting tab in GitHub. Ensure you grant package permissions and repo permissions. This will be needed for package pushing and pulling.

You can set your token as an environment variable as well with:
```bash
export GITHUB_TOKEN=your_personal_access_token
echo $GITHUB_TOKEN 
```
**NOTE:**This will be temporary, for persistance, edit the bashrc file on your system to include the token (recommended).

After creating and storing your PAT, you can login to GitHub packages with this command:
```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u <your_github_username> --password-stdin
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
### Pushing/Pulling Packages to GitHub Packages

To build an image, ensure a Dockerfile is present. Then run:
```bash
docker build -t <image-name> .
```
After verifying this image works, you can then tag it for pushing:
```bash
docker tag <image-name> ghcr.io/<github-username>/<image-name>:<tag> 
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

### 3b. **Run engine as Docker service**
First build the image ([see GitHub Packages section](#pushingpulling-packages-to-github-packages)). You can then run the image with:
```bash
docker run -p <external-port>:<internal-port> <image-name>
```
This will expose the Docker app as a service with which you can use the same ```curl``` method as before, for example:
``` bash
curl -X POST http://localhost:8080/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "load": 75, "fork": false}'
```

### 3c. **Run Engine Deployment in Kubernetes**

In the ```kubernetes``` folder are some YAMLs to be used. Ensure minikube is running with ```minikube status``` AND that you modify them to pull your private development images OR the public package attached to the repository. The public images is:
```bash
ghcr.io/dman/mogwai-engine:latest
```

Then once ready, apply the files:
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
These should display the appropriate pods/service. If you see an ```ImagePullBackOff``` error under ```STATUS```, it means that the pod/deployment was unable to pull the docker image. Ensure you have modified the ```image``` line in ```engine-service.yaml``` to have **YOUR** private docker image OR the public package attached to the repository.

Also verify you have your secrets with ```kubectl get secrets```.

If the pods and services are running, you can now port forward the engine service with:
```bash
kubectl port-forward svc/engine-service <external-port>:8080
```

You can now test the endpoint using curl like:
```bash
curl -X POST http://localhost:8080/cpu-stress -H "Content-Type: application/json" -d '{"intensity": 4, "duration": 10, "load": 100}'

```
At the end of testing, it is recommend to remove the engine and service, as the controller can handle engine spawning and removal.

--for devs - read endpoints for additional information

### 3d. **Run Controller Deployment in Kubernetes**

Before testing the controller, **ensure that the engine deployment is operational** *see section 3c*. In the ```kubernetes``` folder are some YAMLs to be used. Ensure minikube is running with ```minikube status``` AND that you modify them to pull your private development images OR the public package attached to the repository. The public package is:
```bash
ghcr.io/dman7351/mogwai-controller:latest
```
Then once ready, apply the files:
```bash
kubectl apply -f controller-deployment.yaml
kubectl apply -f controller-service.yaml
kubectl apply -f controller-ingress.yaml
kubectl apply -f controller-rbac.yaml
```

Confirm the status of these operations with:
```bash
kubectl get deployments
kubectl get pods
kubectl get svc
kubectl get sa
```
These should display the appropriate pods/service. If you see an ```ImagePullBackOff``` error under ```STATUS```, it means that the pod/deployment was unable to pull the docker image. Ensure you have modified the ```image``` line in ```engine-service.yaml``` to have **YOUR** private docker image OR the public package attached to the repository.

Also verify you have your secrets with ```kubectl get secrets```. 

If the pods and services are running, you can now port forward the controller service with:
```bash
kubectl port-forward svc/controller-service <external-port>:8081
```

You can now test the endpoint using curl like:
```
curl -X POST http://localhost:8081/cpu-stress -H "Content-Type: application/json" -d '{"intensity": 4, "duration": 10, "load": 100}'

```

To test via the ingress, ensure that the ingress addon is enabled and the ingress appears with:
```bash
kubectl get ingress
```

With the ingress running, the controller is now reachable through the cluster IP. You can test this endpoint with either a frontend that sends requests to the minikube IP (should be <192.168.49.2>) or ```curl```, for example:
```
curl -X POST http://192.168.49.2/cpu-stress -H "Content-Type: application/json" -d '{"intensity": 4, "duration": 10, "load": 100}'
```

If you cannot ping your cluster IP, just use the port-forwarding method to test the controller.

## 4. **Test with cURL**

For a full list of cURL command and application functionality, see ```endpoints.md```.

### 5. **Run with CLI/GUI**

First ensure your engine or controller deployment is running ([see section 3c and 3d](#3c-run-engine-deployment-in-kubernetes)). Then navigate to ```cli``` directory or ```gui```. Once there run:
```bash
cargo run
```

The first input will ask for the URL endpoint, enter the appropriate one. For example, if you are testing via Ingress, type ```http://192.168.49.2``` or if port-forwarding use ```http://localhost:<port>```.

