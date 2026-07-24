# Repository Setup

This repository can be cloned or placed in any location on macOS.

When initializing a new clone:

```sh
cd /path/to/cov
git status
```

Before publishing, choose a licence. Do not copy AMI-COV implementation code
without accounting for its AGPL-3.0 licence.

Suggested first release checks:

```sh
make check
make doctor
./install.sh
```
