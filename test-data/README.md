# Test-data

## Run stub with test-data
1. Start wiremock docker by running this command in the root of this project:  
```$ docker-compose up```
2. Add wiremock to mcf list:  
```$ mcf env add wiremock http://localhost:8080 --sso --skip-ssl-validation```
3. Run your test command, for example:  
```$ mcf login wiremock```  
```$ mcf exec wiremock apps```

## How to capture stub data
1. Install mitmporxy  
```https://docs.mitmproxy.org/stable/overview-installation/```
2. Run mitmweb  
```$ mitmweb```
3. Set https_proxy to mitmweb  
```$ export https_proxy=http://localhost:8080```
4. Add your environment to mcf, for example:  
```$ mcf env add your-env-name https://api.company.com --sso --skip-ssl-validation```  
```$ mcf env add your-env-name https://api.company.com --sso```  
```$ mcf env add your-env-name https://api.company.com```  
4. Run the command you want to capture, for example:  
```$ mcf login your-env-name```  
```$ mcf exec your-env-name apps```
5. Unset https_proxy  
```$ unset https_proxy```

## Enable CF tracing  
```export CF_TRACE=true```

## Disable CF tracing
```export CF_TRACE=false```