# Assembly loading
Loading assemblies is done through an AssemblyLoadContext in both native and
managed code. The native ALC is essentially just a handle (hash of name) to the
managed ALC.

ALC.Resolving is an event handler for if the ALC cannot find the requested
Assembly. In C# such an event handler can have multiple callbacks assigned to it
which it will call in the order they are added. Similarly ALC.Unloading is an
event handler for when the ALC is unloaded.

ALCs can hold a maximum of one version of an Assembly, however multiple versions
can be loaded at once, as long as they are loaded in different ALCs.

An ALC should be constructed with a Name and a Bool saying whether or not it
allows unloading.

The ALC class provides a static function to find which ALC has loaded a specific
Assembly, so maybe Coral is doing too much in that regard. The ALC class does
not provide a cheap way to seach for an Assembly within the ALC though, so they
should probably be cached locally in the AssemblyLoader class.

Maybe it is worth it, to make sure AssemblyLoader does not allow for unloading
Sharpen.Managed? Even though it is essentially always possible to just manually
unload it. But otherwise why is Coral caching Coral.Managed?

It seems ALC.Resolving is really mostly meant for finding Assemblies/DLLs which are
stored outside of the regular search-paths of C#, which might be the case in a DIST
build of a game/game engine. However Coral also seems to check all sorts of caches
inside AssemblyLoader and ALC.All.

All in all AssemblyLoader in Coral looks like a huge mess, so Sharpen.Managed's
implementation should probably not necessarily be based on Coral.

For now the plan is:
 - [x] Dictionary s_AssemblyContexts - Key: hash of ALC.Name, Value: ALC
 - [x] Double Dictionary s_AssemblyCache - Key: ALC hash + Assembly hash, Value: Assembly
 - [x] s_LastLoadStatus
 - [x] s_AssemblyLoadErrorLookup?
 - [x] Static constructor, which sets up ErrorLookup, but does not cache SharpenAssemblyLoadContext
 - [ ] Potentially a more sensible version of ResolveAssembly
 - [x] CreateAssemblyLoadContext
 - [x] UnloadAssemblyLoadContext
 - [x] LoadAssembly
 - [x] LoadAssemblyFromMemory?
 - [x] GetLastLoadStatus
 - [x] GetAssemblyName
 - [ ] Remember good exception handling + logging ("[AssemblyLoader]: blah blah blah")


# Types
TODO:
 - [ ] Research how to get all of the type info from C#
 - [ ] Research a better way to represent Type in rust, which does not require the use of Arc<Type> (Also for ManagedAssembly)
 - [X] Make sure to load all Types from an assembly into the TypeCache on load
 - [ ] Maybe stop using the TypeCache? Is there a better way to handle Types
 - [X] Implement the native side


# Managed Objects and method invocation
TODO:
 - [ ] Research how Coral handles this, and if that is the optimal solution?
 - [X] Base managed implementation
 - [X] Base native implementation
 - [X] SUCCESS!!!! Have example running fully (-icalls)


# Marshalling
TODO:
 - [ ] Read through the Coral code
 - [ ] Fix the array issue
 - [ ] Think of a better way
 - [ ] Rewrite in a better way?


# UniqueIdList
TODO:
 - [ ] Consider if this is necessary? and if there is a better way?
 - [ ] Implement something myself instead of just copying from Coral