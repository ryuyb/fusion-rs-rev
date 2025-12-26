# Implementation Plan: Error Handling System Redesign

## Overview

This implementation plan redesigns the error handling system to provide structured error types, automatic database error conversion, anyhow integration, and flexible HTTP response formatting. The implementation will replace the current basic AppError with a comprehensive error handling solution.

## Tasks

- [x] 1. Update core error types and AppResult alias
  - Rewrite `src/error/app_error.rs` with new AppError enum including all variants (NotFound, Duplicate, Validation, BadRequest, UnprocessableContent, Unauthorized, Forbidden, Database, Configuration, ConnectionPool, Internal)
  - Add AppResult<T> type alias for Result<T, AppError>
  - Implement From<anyhow::Error> for AppError conversion
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10, 5.1_

- [ ]* 1.1 Write property tests for AppError structure
  - **Property 1: AppError variant completeness**
  - **Property 2: NotFound error information completeness**
  - **Property 3: Duplicate error information completeness**
  - **Property 4: Validation error information completeness**
  - **Validates: Requirements 1.1, 1.2, 1.3, 3.1, 3.2, 3.3**

- [ ]* 1.2 Write property tests for anyhow integration
  - **Property 9: Anyhow error conversion**
  - **Property 10: Error context preservation**
  - **Property 11: Error source chain preservation**
  - **Validates: Requirements 1.10, 5.1, 5.2, 5.4, 5.5, 6.1**

- [x] 2. Implement database error conversion system
  - Create `src/error/database_converter.rs` with DatabaseErrorConverter struct
  - Implement constraint violation parsing for PostgreSQL error messages
  - Add conversion methods for unique, not null, foreign key, and check constraints
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ]* 2.1 Write property tests for database error conversion
  - **Property 5: Unique constraint conversion**
  - **Property 6: Not null constraint conversion**
  - **Property 7: Foreign key constraint conversion**
  - **Property 8: Database connection error conversion**
  - **Validates: Requirements 2.1, 2.2, 2.3, 2.5**

- [x] 3. Update ErrorResponse DTO with flexible details
  - Rewrite `src/api/dto/error.rs` with new ErrorResponse structure using `Option<serde_json::Value>` for details
  - Add convenience methods for common error patterns (validation_error, not_found_error, duplicate_error)
  - Add builder methods (with_details, with_request_id)
  - _Requirements: 4.11, 4.12, 7.3, 7.4_

- [ ]* 3.1 Write property tests for ErrorResponse serialization
  - **Property 17: JSON serialization consistency**
  - **Property 18: ErrorResponse round-trip property**
  - **Validates: Requirements 7.3, 7.5**

- [x] 4. Implement HTTP response conversion middleware
  - Rewrite `src/api/middleware/error_handler.rs` with updated IntoResponse implementation
  - Add proper status code mapping for all new error variants
  - Implement error message sanitization for external responses
  - Add request ID extraction and inclusion in error responses
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 4.10, 6.4_

- [ ]* 4.1 Write property tests for HTTP response conversion
  - **Property 12: HTTP status code mapping**
  - **Property 13: ErrorResponse structure completeness**
  - **Property 14: Error message sanitization**
  - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 6.4, 6.5**

- [x] 5. Add constraint parsing utilities
  - Create `src/error/constraint_parser.rs` with ConstraintParser struct
  - Implement regex-based parsing for PostgreSQL constraint violation messages
  - Add caching for compiled regex patterns for performance
  - Handle parsing failures gracefully with fallback to generic errors
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ]* 5.1 Write unit tests for constraint parsing
  - Test known PostgreSQL error message formats
  - Test parsing edge cases and malformed messages
  - Test regex pattern caching and performance
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 6. Implement Display and Debug formatting
  - Add proper Display trait implementation for user-friendly error messages
  - Add comprehensive Debug trait implementation for logging
  - Ensure sensitive information is not exposed in Display format
  - _Requirements: 7.1, 7.2, 6.4_

- [ ]* 6.1 Write property tests for error formatting
  - **Property 15: Debug formatting completeness**
  - **Property 16: Display formatting user-friendliness**
  - **Validates: Requirements 7.1, 7.2**

- [x] 7. Update existing code to use new error system
  - Update all repository methods to use AppResult<T> return type
  - Update service layer to use new error variants appropriately
  - Update API handlers to leverage new error conversion
  - Replace old AppError usage throughout the codebase
  - _Requirements: All requirements integration_

- [ ]* 7.1 Write integration tests for error flow
  - Test end-to-end error handling from database to HTTP response
  - Test error context preservation through all layers
  - Test request ID correlation in error responses
  - _Requirements: All requirements integration_

- [ ] 8. Checkpoint - Ensure all tests pass and error handling works correctly
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties using proptest crate
- Unit tests validate specific examples and edge cases
- The implementation maintains backward compatibility where possible
- Database error conversion focuses on PostgreSQL but can be extended for other databases