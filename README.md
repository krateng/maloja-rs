# maloja-rs

Maloja-rs is the reimplementation of the absolute spaghetti bowl that is my first project [Maloja](https://github.com/krateng/maloja).
It is, at the moment, more of a learning experience, but if I'm lucky it should eventually replace the original.

### Why?
I started with Maloja when I didn't have the slighest clue of how to structure a project or make code maintainable, and the codebase shows that to this day.
I was in love with Python's playful elegance and the ability to \_\_magic\_\_ everything into anything, add layer upon layer of metaprogramming to avoid repeating even one line and fix every problem by monkey-patching some class beyond the limits of the Geneva Convention.
Well, no longer. As I sit on my porch, smoking some *Old Toby* and stroking my grey beard, I realize those days of playing it hard and fast with the rules are behind me and I now need strict typing, a compiler that forces me to think of every case ahead of time, and no more hash maps.

So, friendship ended with Python, now Rust is my best friend!

### Backwards Compatibility

I'm using this opportunity to remove cruft and not bother with chaotic messy code just to support every possible previous configuration.
So if this ever becomes production-ready, data will have to be imported from an old installation instead of just reusing the same data folder.
However, the web API should of course remain consistent.