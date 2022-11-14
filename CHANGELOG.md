# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]
### Added
- `TskVsPartHandle` to handle io for the partition volume

### Fixed
- Handling of null pointers returned from `tsk_error_get`

### Changed
- tests work of of test images instead of logical volume
- renamed `TskImg::from_source` to `TskImg::from_utf8_sing` to better fit with libtsk function name

## [0.2.0]
### Added
- `TskImg::from_tsk_img_info_ptr()` to create a TskImg from `NonNull<tsk::TSK_IMG_INFO>`
- imported `tsk_img_open_external` tsk function
- imported tsk structs now derive debug and default

## [0.1.0]
### Added
- Initial Version