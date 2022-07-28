# NACOS perf utils

快速模拟注册 Nacos 实例，使用 [`openapi`](https://nacos.io/zh-cn/docs/open-api.html)

## Usage
```
$ nacos-perf-utils --help
```

### 注册虚拟实例
```
$ nacos-perf-utils instance -p 10000 -n 10  http://<YOUR-NACOS-IP>:8848
```


### 注册虚拟实例(带验证)
```
$ nacos-perf-utils instance -p 10000 -n 10 --username <NACOS_USER> --password <NACOS_PWD>  http://<YOUR-NACOS-IP>:8848
```