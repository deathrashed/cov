# Repository Setup

This directory is ready to become a standalone repository, but it has not been
initialised automatically because it currently lives inside
`/Users/rd/Scripts`, an existing Git worktree.

When ready:

```sh
cd /Users/rd/Scripts/Riley/audio/cov
git init
git add .
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
