# SysQL

A small SQL-like language for recording and querying Linux process events.

```
sysql "RECORD processes WHERE name = 'python3' FOR 60"
sysql "SELECT pid, name FROM processes WHERE name = 'python3'"
```
