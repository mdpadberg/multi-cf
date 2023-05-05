# Test-data

## Run stub with test-data
Start wiremock docker by running
```$ ./run-wiremock-docker.sh```

## How to capture stub data
1. Install mitmporxy
```https://docs.mitmproxy.org/stable/overview-installation/```
2. Run mitmweb
```$ mitmweb```
3. Set https_proxy to mitmweb
```$ export https_proxy=http://localhost:8080```
4. Run the command you want to capture, for example:
```$ mcf login wiremock```
```$ mcf exec wiremock apps```
5. Unset https_proxy
```$ unset https_proxy```