# Example Project
## Dependency
```mermaid
graph LR;
    pybin--config-->pylib--config-->pylib2
    pylib--pyproject-->numpy
    pylib2--pyproject-->numpy
```
- python dependency in *python crate* (`pylib`, `pylib2`) should be specified in `crate_python::config!`.
- other python dependency (`numpy`) should be  specified in `pyproject.toml`   