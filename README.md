## Sharpen
Sharpen is an attempt at creating a rust library akin to StudioCherno's Coral or Xamarin's Mono.

Essentially Sharpen should be as close to a pure rust implementation of C# scripting library as possibly.

## Credit
Currently the entire C# part of the library is just copy/paste from Coral, and the rust side is essentially a line-by-line port of the C++ in Coral.

I do plan to change that.

## Future
 - [ ] Use proper rust error handling where possible instead of just logging the errors from C#
 - [ ] Write Sharpen.Managed to replace Coral.Managed.
 - [ ] Make sharpen_native more rust-like/less C++-like