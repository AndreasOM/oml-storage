# TODO

## InProgress

- [ ] Implement dynamodb backed storage
	- [ ] #dynamodb_storage Implement exists
	- [ ] #dynamodb_storage Check existing lock when locking

## ToDo
- [ ] #disk_storage Check if base path exists
- [ ] #disk_storage Improve error handling
- [ ] Add feature flags to enable storage backends

## Done
- [x] Start adding some very basic documentation
- [x] #disk_storage Allow ensuring folder exists
- [x] #disk_storage Improve error reporting

- [x] Add test to ensure backend implementations `debug`
- [x] Cleanup unused semaphore from DynamoDB backend

## Released

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
