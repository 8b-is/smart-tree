# Changelog

All notable changes to Smart Tree will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1] - 2024-01-XX

### Added
- Enhanced search functionality with line and column information
  - Search results now show exact line number and column position
  - Display format: `[SEARCH:L<line>:C<column>]` for single matches
  - Multiple matches show count: `[SEARCH:L<line>:C<column>,<count>x]`
  - Truncated results indicated: `[SEARCH:L<line>:C<column>,<count>x,TRUNCATED]`
- Improved search performance with truncation at 100 matches per file

### Fixed
- Search filtering now properly excludes files without matches
- Fixed issue where `--type` filter would show all files of that type even without search matches
- Search results are now properly filtered in both streaming and non-streaming modes

### Changed
- Search match structure improved for better performance and usability
- Reduced maximum search matches per file from 1000 to 100 for better performance

## [1.1.0] - 2024-01-XX

### Added
- Partnership documentation updates
- MCP (Model Context Protocol) as default feature
- OpenAPI specification for MCP server

## [1.0.2] - 2024-01-XX

### Added
- GitHub release management
- New version system

## [1.0.0] - 2024-01-XX

### Added
- Initial release with all core features
- Multiple output formats (classic, hex, json, csv, tsv, ai)
- Advanced filtering options
- MCP server integration
- Cross-platform support 