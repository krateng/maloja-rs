# maloja-rs

WIP Rust reimplementation of the absolute spaghetti bowl that is my first project [Maloja](https://github.com/krateng/maloja). I'm using this opportunity to remove cruft and not bother with chaotic messy code just to support every possible previous configuration, so old data will have to be imported instead of simply reusing the existing folder. However, the web API should of course remain consistent.

## Trying it out

It's pretty barebones so far, but you can run the container with the provided `compose.yml` or fetch a built image from github and run it with e.g.
```shell
    docker run \
    --name maloja \
    -p 42010:42010 \
    -v ./data:/data -v ./config:/config -v ./logs:/logs \
    ghcr.io/krateng/maloja-rs:edge
```

Make sure you use `data` and `logs` folders that are writable for the container's 1000 user, and a `config` folder that is at least readable.
Copy an export from the old Maloja into your `data/import` folder.
Visit http://localhost:42010/api_explorer or e.g. http://localhost:42010/artist/1.

The database schema is still changing with no migrations provided, so do not use this as your real instance yet!