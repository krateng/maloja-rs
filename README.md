# maloja-rs

Maloja-rs is the reimplementation of the absolute spaghetti bowl that is my first project [Maloja](https://github.com/krateng/maloja).
It is, at the moment, more of a learning experience, but if I'm lucky it should eventually replace the original.
I started with Maloja when I didn't have the slightest clue of how to structure a project or make code maintainable,
and the codebase shows that to this day.


### Backwards Compatibility

I'm using this opportunity to remove cruft and not bother with chaotic messy code just to support every possible previous configuration.
So if this ever becomes production-ready, data will have to be imported from an old installation instead of just reusing the same data folder.
However, the web API should of course remain consistent.

## Trying it out

It's pretty barebones so far, but you can run the container with the provided `compose.yml`.
Copy an export from the old Maloja into your `/data/import` folder.
Visit http://localhost:42010/api_explorer or e.g. http://localhost:42010/artist/1.