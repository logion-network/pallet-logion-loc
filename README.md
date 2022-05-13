# pallet-logion-loc
The pallet to manage and query logion Legal Officer Cases (LOC).

A LOC has one the types: `Transaction`, `Collection` or `Identity`.
* It can be Open, Closed (no more updates allowed) or Void (no more valid).
* It contains metadata, hash (sha-256) of files or links to other LOCs.
Additionally, Collection LOC also contains collection items, identified by a hash.
All those items also have a public description.
* Most of the operations are allowed only for the Legal Officer, owner of the LOC. A Wallet User can only add item to a Collection LOC he/she is the requester of. 

This pallet provides entry points to 
* Create, close or void (and possibly replace) a LOC.
* Add metadata, files, links and collection items.

## Use, Build and Publish
Details on how to use, build and publish can be found [here](https://github.com/logion-network/logion-shared#readme)
