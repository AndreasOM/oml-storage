# oml-storage

A very simple wrapper to handle locked storage of items.


## Warning

This crate is still very much in flux,
and things might change a lot.

We do use it in production for one of our games,
so it _should_ be *good enough*.

## Examples
For Examples check [oml-storage-examples](https://github.com/AndreasOM/oml-storage-examples).

## Future

- [ ] Considering merging this and the examples into a single workspace.
- [ ] Considering adding an explicit StorageItemId trait, and include some default implementations.

## Changes

### 0.4.1
- Added lock_new, which will return AlreadyExists if the item already exists.
    - Only implemented for DiskStorage for now.

## Breaking Changes

## 0.2.x -> 0.3.x

### metadata_highest_seen_id return Option<ITEM::ID>
	
metadata_highest_seen_id returns an Option<ITEM::ID> now,
which will be None if we haven't seen any Id yet.


## 0.1.x -> 0.2.x

### Replaced &str ID with ITEM::ID

- [ ] You will need to implement `make_id` and `generate_next_id` for you Items!
- [ ] Consider extra careful testing when using anything but String for ITEM::ID
