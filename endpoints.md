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
The curl command to test (via port-forward) is:
```
curl -X POST http://localhost:<target-port>/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "loa
d": 75, "fork": false}'
```
Or for ingress:
```
curl -X POST http://<minikube-ip>/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "loa
d": 75, "fork": false}'
```
## Memory endpoint ##
The CPU test end point is ```/mem-stress```
The parameters are:
- intensity: int (this is the number of threads)
- size: int
- duration: int
The curl command to test (via port-forward) is:
```
curl -X POST http://localhost:<target-port>/mem-stress   -H "Content-Type:application/json"   -d '{"size": 256, "duration": 10}'
```
Or for ingress:
```
curl -X POST http://<minikube-ip>/mem-stress   -H "Content-Type:application/json"   -d '{"size": 256, "duration": 10}'
```
## Disk endpoint ##
The CPU test end point is ```/disk-stress```
The parameters are:
- intensity: int (this is the number of threads)
- size: int
- duration: int
The curl command to test (via port-forward) is:
```
curl -X POST http://localhost:<target-port>/disk-stress   -H "Content-Type:application/json"   -d '{"intensity": 256, "duration": 10}'
```
Or for ingress:
```
curl -X POST http://<minikube-ip>/disk-stress   -H "Content-Type:application/json"   -d '{"intensity": 256, "duration": 10}'
```

## Node list endpoint ##
The GET request to list nodes is ```/nodes```
There are no parameters.
The curl command to test (via port-forward) is:
```
curl http://localhost:<target-port>/nodes
```
Or for ingress:
```
curl http://<minikube-ip>/nodes
```

## Spawn engine endpoint ##
The spawn engine endpoint creates an engine and servce for a specified node. The endpoint is ```/spawn-engine```
The parameter is:
- node_name : String (the name of the node from ```/nodes``` output)
The curl command to test (via port-forward) is:
```
curl -X POST http://localhost:<target-port>/spawn-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```
Or for ingress:
```
curl -X POST http://<minikube-ip>/spawn-engine   -H "Content-Type: application/json"   -d '{"node_name": "<node-name>"}'
```




