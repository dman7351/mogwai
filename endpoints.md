# ENDPOINTS #
### *For dev use* ###

## Port-forwarding ##
To test via port forwarding on the kubernetes cluster use ```kubectl port-forward svc/<serivce> <target-port>:<service-port>```
Service can be:
- ```engine-service``` for testing the engine endpoint
- ```controller-service``` for testing controller endpoint

You should then be able to access this via ```localhost:<target-port>```. 

## CPU endpoint ##
The CPU test end point is ```/cpu-stress```
The parameters are:
- intensity: int
- duration: int
- load: float/int
- flag: boolean
The curl command to test is:
```
curl -X POST http://localhost:<target-port>/cpu-stress   -H "Content-Type:application/json"   -d '{"intensity": 1, "duration": 10, "loa
d": 75, "fork": false}'
```
## Memory endpoint ##
The CPU test end point is ```/mem-stress```
The parameters are:
- size: int
- duration: int
The curl command to test is:
```
curl -X POST http://localhost:<target-port>/mem-stress   -H "Content-Type:application/json"   -d '{"size": 256, "duration": 10}'
```
## Disk endpoint ##
The CPU test end point is ```/disk-stress```
The parameters are:
- size: int
- duration: int
The curl command to test is:
```
curl -X POST http://localhost:<target-port>/disk-stress   -H "Content-Type:application/json"   -d '{"intensity": 256, "duration": 10}'
```




