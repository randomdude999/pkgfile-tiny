# pkgfile_ng

i'm not sure the "ng" suffix is really deserved.

this is a version of [pkgfile](https://github.com/falconindy/pkgfile) which is much more limited, but also much faster.

the original pkgfile has to scan the entire files database for each query. this takes a few hundred milliseconds even if the database is already cached in RAM. this annoyed me, as it meant my terminal locked up for up to a second every time i typo'd a command. this version works just as well when using it as a command-not-found handler, but it has none of the other features of pkgfile. notably, it filters out non-executable files while building the database, as a way to save space (otherwise, the community files DB would be a few hundred MB).

usage: `pkgfile_ng` with no arguments updates the database. the database location is hardcoded in `main.rs`. `pkgfile_ng <query>` outputs the packages that contain a binary with the specified basename, i.e. the exact same thing as `pkgfile -v -b <query>`.
