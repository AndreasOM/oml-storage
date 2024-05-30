# TODO

## InProgress

- [ ] Add `metadata`feature
	- [ ] Remember highest seen id
	- [ ] Fix IDs?!

## ToDo


- [ ] Implement dynamodb backed storage
	- [ ] #dynamodb_storage Implement exists
	- [ ] #dynamodb_storage Check existing lock when locking

- [ ] #disk_storage Check if base path exists
- [ ] #disk_storage Improve error handling
- [ ] Add feature flags to enable storage backends
- [ ] Add (streaming) iterator for all ids
- [ ] Add `destroy` method to delete items for good (with extra protection)

## Done

- [x] Add `display_lock` for debugging


## Released

### 0.1.6-alpha - 2024-02-07
- [x] Add getter for all ids

### 0.1.5-alpha - 2024-01-06
- [x] Start adding some very basic documentation
- [x] #disk_storage Allow ensuring folder exists
- [x] #disk_storage Improve error reporting

- [x] Add test to ensure backend implementations `debug`
- [x] Cleanup unused semaphore from DynamoDB backend

### 0.1.4-alpha - 2024-01-05
- [x] Add simple command line parameters to run demo on different backends
- [x] Implement null storage

### v0.1.2-alpha - 2024-01-03
- [x] Implement disk backed storage
- [x] save requires lock
- [x] verify_lock
- [x] Stub out basic interface
	- [x] create
	- [x] exist
	- [x] NO: delete
	- [x] lock
	- [x] unlock
	- [x] force_unlock
	- [x] load
	- [x] save (!)
- [x] Set up basic project
