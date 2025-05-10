# ENDPOINTS #
### *For dev use* ###

## Using ingress ##

After applying the ingress you should be able to send requests via the minikube ip as in ```curl -v http://<minikube-ip>/nodes```

## Port-forwarding ##
To test via port forwarding on the kubernetes cluster use ```kubectl port-forward svc/<serivce> <target-port>:<service-port>```
Service can be:
- ```engine-service``` for testing the engine endpoint
- ```controller-service``` for testing controller endpoint

You should then be able to access this via ```localhost:<target-port>```. 

## CPU endpoint ##
The CPU test end point is ```/cpu-stress```
The parameters are:
- intensity: int (this is the number of threads)
- duration: int
- load: float/int
- flag: boolean
- node: String (node name from ```/nodes``` output)
The curl command to test (via port-forward) is:
```bash
curl -X POST http://localhost:<target-port>/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "loa
d": 75, "fork": false, "node":"<node name>"}'
```
Or for ingress:
```bash
curl -X POST http://<minikube-ip>/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "loa
d": 75, "fork": false, "node":"<node name>"}'
```
## Memory endpoint ##
The CPU test end point is ```/mem-stress```
The parameters are:
- intensity: int (this is the number of threads)
- size: int
- duration: int
- node: String (node name from ```/nodes``` output)
The curl command to test (via port-forward) is:
```bash
curl -X POST http://localhost:<target-port>/mem-stress   -H "Content-Type:application/json"   -d '{"size": 256, "duration": 10, "node":"<node name>"}'
```
Or for ingress:
```bash
curl -X POST http://<minikube-ip>/mem-stress   -H "Content-Type:application/json"   -d '{"size": 256, "duration": 10, "node":"<node name>"}'
```
## Disk endpoint ##
The CPU test end point is ```/disk-stress```
The parameters are:
- intensity: int (this is the number of threads)
- size: int
- duration: int
- node: String (node name from ```/nodes``` output)
The curl command to test (via port-forward) is:
```bash
curl -X POST http://localhost:<target-port>/disk-stress   -H "Content-Type:application/json"   -d '{"intensity": 256, "duration": 10, "node":"<node name>"}'
```
Or for ingress:
```bash
curl -X POST http://<minikube-ip>/disk-stress   -H "Content-Type:application/json"   -d '{"intensity": 256, "duration": 10, "node":"<node name>"}'
```

## Node list endpoint ##
The GET request to list nodes is ```/nodes```
There are no parameters.
The curl command to test (via port-forward) is:
```bash
curl http://localhost:<target-port>/nodes
```
Or for ingress:
```bash
curl http://<minikube-ip>/nodes
```

## Spawn engine endpoint ##
The spawn engine endpoint creates an engine and servce for a specified node. The endpoint is ```/spawn-engine```
The parameter is:
- node_name : String (the name of the node from ```/nodes``` output)
The curl command to test (via port-forward) is:
```bash
curl -X POST http://localhost:<target-port>/spawn-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```
Or for ingress:
```bash
curl -X POST http://<minikube-ip>/spawn-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```

## Remove engine endpoint ##
The remove engine endpoint removes an engine and service for a specified node. The endpoint is ```/remove-engine```
The parameter is:
- node_name : String (the name of the node from ```/nodes``` output)
The curl command to test (via port-forward) is:
```bash
curl -X POST http://localhost:<target-port>/remove-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```
Or for ingress:
```bash
curl -X POST http://<minikube-ip>/remove-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```

## List tasks endpoint ##
This endpoint lists the running tasks on a specific engine instance. There are no json paramters.
If connecting to engine itself (via local run on port-forward in cluster), the endpoint is ```/tasks```:
```bash
curl http://localhost:<target-port>/tasks
```
If connecting through the controller, the curl command is changed to a POST request at the endpoint ```tasks/<node>```:
```bash
curl -X POST http://<minikube-ip>/tasks/<node> # for ingress
curl -X POST http://localhost:<target-port>/tasks/<node> # for port forward
```

## Stop task endpoint ##
This endpoint will stop the running test based on a given test ID. There are no json parameters.
If connectiong to the engine itself (via local run or port-forward in cluster), the endpoint is ```/stop/<task-ID>```:
```bash
curl -X POST http://localhost:<target-port>/stop/<task-ID>
```
If connecting through the controller, the endpoint is ```/stop/<node>/<task-ID>```:
```bash
curl -X POST http://<minikube-ip>/stop/<node>/<task_ID> # for ingress
curl -X POST http://localhost:<target-port>/stop/<node>/<task_ID> # for port forward
```

## Stop all tasks endpoint ##
This endpoint will stop all running tasks. There are no json parameters.
If connecting to the engine itself (via local run or port-forward in cluster), the endpoint is ```/stop-all```:
```bash
curl -X POST http://localhost:<target-port>/stop-all
```
If connecting through the controller, the endpoint is ```/stop-all```:
```bash
curl -X POST http://<minikube-ip>/stop-all # for ingress
curl -X POST http://localhost:<target-port>/stop-all # for port forward
```



