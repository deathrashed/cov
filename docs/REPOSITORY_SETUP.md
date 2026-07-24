# Repository Setup

This directory is a standalone Git repository on the `rust-rewrite` branch.

When ready:

```sh
git status
git add .
```

Before publishing, choose a licence. Do not copy AMI-COV implementation code
without accounting for its AGPL-3.0 licence.

Suggested first release checks:

```sh
make verify
make doctor
./install.sh
```
