# Specta Swift Refactor Plan

**Status**: Draft  
**Created**: October 3, 2025  
**Goal**: Transform the current working implementation into production-ready, senior engineer-level, open-source grade code.

---

## Current State Assessment

### ✅ What's Working

- All tests passing (100%)
- Basic type generation (structs, enums, primitives)
- String enum raw values
- Adjacently tagged enums
- CodingKeys generation
- Enum variant coding keys
- Duplicate name handling with 4 strategies
- JsonValue and serde_json::Value support
- Custom struct field initializers
- Comprehensive test coverage

### 🚨 Critical Issues

1. **170 lines of dead code removed** (export_type function)
2. **Tuple variants**: Stub implementations with `fatalError` - BLOCKING ISSUE
3. **Code organization**: Single 1,408-line `primitives.rs` file
4. **Code duplication**: Repeated patterns throughout
5. **Limited documentation**: Minimal inline comments and no module docs
6. **Error handling**: Inconsistent error messages and contexts
7. **Type complexity**: Nested match statements 5+ levels deep

### 📊 Current Metrics

- **File size**: `primitives.rs` = 1,408 lines
- **Test coverage**: 9 test files, all passing
- **Dead code**: 0 lines (after cleanup)
- **TODO count**: Multiple tuple variant TODOs causing runtime crashes

---

## Refactor Goals

### 1. Code Quality Standards

- [ ] Senior engineer-level architecture
- [ ] Open-source grade documentation
- [ ] Production-ready error handling
- [ ] Maintainable and extensible design
- [ ] Zero technical debt
- [ ] All tests passing

### 2. Modular File Structure

Transform monolithic `primitives.rs` into logical modules:

```
src/
├── lib.rs                    # Public API surface
├── error.rs                  # Error types (✓ exists)
├── swift.rs                  # Swift config (✓ exists)
│
├── datatype/                 # Data type handling
│   ├── mod.rs               # Module exports
│   ├── struct_gen.rs        # Struct generation
│   ├── enum_gen.rs          # Enum generation
│   ├── tuple_gen.rs         # Tuple variant generation (NEW)
│   └── primitives.rs        # Primitive type mapping
│
├── codable/                  # Codable implementation
│   ├── mod.rs               # Module exports
│   ├── struct_codable.rs    # Struct Codable
│   ├── enum_codable.rs      # Enum Codable
│   ├── adjacently_tagged.rs # Adjacently tagged enums
│   └── coding_keys.rs       # CodingKeys generation
│
├── naming/                   # Naming strategies
│   ├── mod.rs               # Module exports
│   ├── strategies.rs        # DuplicateNameStrategy
│   ├── case_conversion.rs   # snake_case to camelCase
│   └── resolver.rs          # Name conflict resolution
│
├── special_types/            # Special type handling
│   ├── mod.rs               # Module exports
│   ├── json_value.rs        # JsonValue handling
│   ├── duration.rs          # Duration handling
│   └── serde_json.rs        # serde_json::Value
│
└── utils/                    # Utilities
    ├── mod.rs               # Module exports
    ├── formatting.rs        # Code formatting helpers
    ├── validation.rs        # Type validation
    └── testing.rs           # Test utilities

```

### 3. Architecture Principles

#### **Separation of Concerns**

- **Generation layer**: Pure type-to-string conversion
- **Codable layer**: Protocol implementation logic
- **Naming layer**: Conflict resolution and case conversion
- **Validation layer**: Type checking and error prevention

#### **Design Patterns**

- **Builder pattern**: For complex type construction
- **Strategy pattern**: For naming and generation strategies
- **Visitor pattern**: For recursive type traversal
- **Factory pattern**: For type-specific generators

#### **Error Handling**

- Comprehensive error types with context
- Never use `panic!` or `unwrap()` in production paths
- Proper error propagation with `Result<T, Error>`
- Helpful error messages for debugging

---

## Detailed Refactor Tasks

### Phase 1: Foundation & Planning ✅

- [x] Document current state
- [x] Identify code smells
- [x] Remove dead code
- [x] Create refactor plan
- [x] Establish success criteria

### Phase 2: Module Structure (Week 1)

#### 2.1 Create Module Skeleton ✅

- [x] Create directory structure
- [x] Add `mod.rs` files with exports
- [x] Update `lib.rs` with new module imports
- [x] Ensure tests still pass (43/43 passing)

#### 2.2 Extract Data Type Generation

- [ ] Create `datatype/struct_gen.rs`
  - Extract struct generation logic
  - Add struct builder pattern
  - Document struct field handling
  - Add unit tests
- [ ] Create `datatype/enum_gen.rs`

  - Extract enum generation logic
  - Separate enum types (unit, tuple, struct, string)
  - Add enum builder pattern
  - Document variant handling
  - Add unit tests

- [ ] Create `datatype/tuple_gen.rs` ⚠️ CRITICAL

  - **Implement tuple variant support**
  - Add encoding/decoding logic
  - Add comprehensive tests
  - Remove all `fatalError` TODOs

- [x] Create `datatype/primitives.rs` ✅
  - Extracted `primitive_to_swift()` function with full documentation
  - Extracted `literal_to_swift()` function with full documentation
  - Added 13 comprehensive unit tests
  - Added doc tests for both functions
  - Documented all 15 supported primitive types + 3 unsupported
  - All tests passing (45/45)

#### 2.3 Extract Codable Implementation

- [ ] Create `codable/struct_codable.rs`

  - Extract struct Codable generation
  - Improve init generation
  - Add optional field handling
  - Add unit tests

- [ ] Create `codable/enum_codable.rs`

  - Extract enum Codable generation
  - Separate by enum representation
  - Add comprehensive error handling
  - Add unit tests

- [ ] Create `codable/adjacently_tagged.rs`

  - Extract adjacently tagged enum logic
  - Add tag/content key customization
  - Document tag format
  - Add unit tests

- [ ] Create `codable/coding_keys.rs`
  - Extract CodingKeys generation
  - Add rename rules (snake_case → camelCase)
  - Add custom key handling
  - Add unit tests

#### 2.4 Extract Naming Logic

- [ ] Create `naming/strategies.rs`

  - Move `DuplicateNameStrategy` enum
  - Implement each strategy cleanly
  - Add strategy tests

- [x] Create `naming/case_conversion.rs` ✅

  - Extracted `snake_to_camel()` function
  - Extracted `snake_to_pascal()` function
  - Extracted `to_snake_case()` with acronym handling
  - Added `snake_to_kebab()` function
  - Added `snake_to_screaming_snake()` function
  - Added case detection functions (`is_snake_case`, `is_camel_case`, `is_pascal_case`)
  - Added 22 comprehensive unit tests including roundtrip tests
  - Added 8 doc tests
  - All tests passing (70+ tests total)

- [ ] Create `naming/resolver.rs`
  - Extract name conflict resolution
  - Improve qualified name generation
  - Add module path handling
  - Add unit tests

#### 2.5 Extract Special Type Handling

- [ ] Create `special_types/json_value.rs`

  - Extract JsonValue detection
  - Add JsonValue generation
  - Document custom decoder
  - Add unit tests

- [ ] Create `special_types/duration.rs`

  - Extract Duration handling
  - Add RustDuration type generation
  - Document conversion logic
  - Add unit tests

- [ ] Create `special_types/serde_json.rs`
  - Extract serde_json::Value handling
  - Add serde JSON type detection
  - Add unit tests

#### 2.6 Create Utilities

- [x] Create `utils/formatting.rs` ✅

  - Extracted `indent()` function with 3 levels of indentation support
  - Extracted `format_doc_comment()` for Swift `///` comments
  - Extracted `escape_string()` for string literal escaping
  - Added `join_non_empty()` utility function
  - Added `format_deprecated()` for `@available` attributes
  - Added 19 comprehensive unit tests
  - Added 5 doc tests
  - All tests passing (50+ tests total)

- [ ] Create `utils/validation.rs`
  - Add type validation
  - Add circular reference detection
  - Add invalid name detection
  - Add unit tests

### Phase 3: Code Quality (Week 2)

#### 3.1 Documentation

- [ ] Add module-level documentation to all files
- [ ] Add comprehensive inline comments
- [ ] Document all public functions
- [ ] Add usage examples
- [ ] Create architecture diagram
- [ ] Update README.md with new architecture

#### 3.2 Error Handling

- [ ] Audit all error paths
- [ ] Add contextual error messages
- [ ] Replace panics with proper errors
- [ ] Add error recovery where possible
- [ ] Document error handling strategy

#### 3.3 Testing

- [ ] Ensure all existing tests pass
- [ ] Add unit tests for each module
- [ ] Add integration tests
- [ ] Add edge case tests
- [ ] Add performance benchmarks
- [ ] Achieve 90%+ code coverage

#### 3.4 Code Review

- [ ] Remove all code duplication
- [ ] Simplify complex functions (max 50 lines)
- [ ] Reduce nesting depth (max 3 levels)
- [ ] Add type aliases for complex types
- [ ] Apply Rust idioms consistently
- [ ] Run clippy with strict lints

### Phase 4: Advanced Features (Week 3)

#### 4.1 Tuple Variant Implementation ⚠️ CRITICAL

- [ ] Design tuple variant representation
- [ ] Implement encoding logic
- [ ] Implement decoding logic
- [ ] Add tests for all tuple variant types
- [ ] Remove all `fatalError` TODOs
- [ ] Validate against Spacedrive types

#### 4.2 Performance Optimization

- [ ] Profile code generation
- [ ] Optimize string allocation
- [ ] Cache type lookups
- [ ] Reduce allocations in hot paths
- [ ] Add performance tests

#### 4.3 Extensibility

- [ ] Add plugin architecture for custom types
- [ ] Add configuration options
- [ ] Add hooks for custom generation
- [ ] Document extension points

### Phase 5: Polish & Release (Week 4)

#### 5.1 Final Testing

- [ ] Run full test suite
- [ ] Test with Spacedrive types
- [ ] Test with real-world projects
- [ ] Fix any remaining issues
- [ ] Validate all tests pass

#### 5.2 Documentation

- [ ] Write comprehensive ARCHITECTURE.md
- [ ] Write CONTRIBUTING.md
- [ ] Update CHANGELOG.md
- [ ] Add code examples
- [ ] Create migration guide

#### 5.3 Release Preparation

- [ ] Final code review
- [ ] Update version numbers
- [ ] Tag release
- [ ] Publish to crates.io

---

## Success Criteria

### Must Have ✅

1. ✅ All existing tests passing
2. ⚠️ Zero `TODO` comments in production code
3. ⚠️ Zero `panic!`, `unwrap()`, or `expect()` in production paths
4. ⚠️ Comprehensive error handling
5. ⚠️ Module-level documentation on all files
6. ⚠️ Function documentation on all public APIs
7. ⚠️ Code coverage ≥ 90%
8. ⚠️ Clippy passing with strict lints
9. ⚠️ No files > 500 lines
10. ⚠️ No functions > 50 lines
11. ⚠️ Max nesting depth ≤ 3 levels
12. ⚠️ **Tuple variants fully implemented**

### Nice to Have 🎯

1. Performance benchmarks
2. Architecture diagram
3. Code generation examples
4. Tutorial documentation
5. Video walkthrough
6. Community contributions guide

---

## Technical Debt to Address

### 1. Tuple Variants ⚠️ CRITICAL

**Current**: `fatalError("Tuple variant decoding not implemented")`  
**Target**: Full encoding/decoding support  
**Impact**: Blocks production use of SpacedriveTypes  
**Priority**: P0 - MUST FIX

### 2. Code Organization

**Current**: 1,408-line `primitives.rs`  
**Target**: 10+ modules, each < 200 lines  
**Impact**: Maintainability and extensibility  
**Priority**: P1

### 3. Error Messages

**Current**: Generic error messages  
**Target**: Contextual, actionable error messages  
**Impact**: Developer experience  
**Priority**: P1

### 4. Documentation

**Current**: Minimal inline comments  
**Target**: Comprehensive module and function docs  
**Impact**: Onboarding and maintainability  
**Priority**: P2

### 5. Code Duplication

**Current**: Repeated patterns for enum variants  
**Target**: Extracted helper functions  
**Impact**: Maintainability  
**Priority**: P2

---

## Code Quality Checklist

### Per-Function Review

- [ ] Single responsibility
- [ ] Clear naming
- [ ] Documented behavior
- [ ] Error handling
- [ ] Type safety
- [ ] No magic numbers
- [ ] No code duplication
- [ ] Tested

### Per-Module Review

- [ ] Clear purpose
- [ ] Public API documented
- [ ] Internal functions private
- [ ] Minimal dependencies
- [ ] No circular dependencies
- [ ] Integration tested

### Overall Review

- [ ] Consistent style
- [ ] Idiomatic Rust
- [ ] No unsafe code (unless necessary)
- [ ] No unwrap/expect in production
- [ ] Performance acceptable
- [ ] Memory usage reasonable

---

## Risk Assessment

### High Risk ⚠️

1. **Tuple variants**: Complex to implement correctly
2. **Breaking changes**: May affect existing users
3. **Test coverage**: Must maintain 100% passing tests

### Medium Risk ⚙️

1. **Module boundaries**: May need adjustment during refactor
2. **API changes**: Internal APIs may shift
3. **Performance**: Large refactor may impact performance

### Low Risk ✅

1. **Documentation**: Pure additive
2. **Code organization**: Internal only
3. **Utilities**: New functionality

---

## Rollback Plan

If refactor fails or tests break:

1. Git revert to last known good state
2. Identify breaking change
3. Create minimal fix
4. Re-test
5. Document lesson learned

---

## Communication Plan

### During Refactor

- Daily: Update this document with progress
- Weekly: Summarize completed tasks
- Blockers: Document immediately

### Post Refactor

- CHANGELOG.md: Document all changes
- README.md: Update with new architecture
- GitHub: Create release notes
- Community: Announce improvements

## Notes

### Key Insights from Current Implementation

1. String enums with raw values work perfectly
2. Adjacently tagged enums are well-supported
3. CodingKeys generation is robust
4. Duplicate name handling is comprehensive
5. **Tuple variants are the main gap**

### Design Decisions to Make

1. How to represent tuple variants in Swift?
2. Should we use arrays or separate properties?
3. How to handle tuple variant encoding?
4. What's the JSON format for tuple variants?

### Questions to Answer

1. What serde representation do tuple variants use?
2. How does Spacedrive use tuple variants?
3. Are there edge cases we're missing?
4. What's the performance impact?

---

## Resources

### Documentation

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Swift Codable Guide](https://developer.apple.com/documentation/foundation/archives_and_serialization/encoding_and_decoding_custom_types)
- [Serde Data Model](https://serde.rs/data-model.html)

### Tools

- `cargo fmt`: Code formatting
- `cargo clippy`: Linting
- `cargo test`: Testing
- `cargo doc`: Documentation
- `cargo tarpaulin`: Code coverage

### Examples

- [serde-rs](https://github.com/serde-rs/serde): Rust serialization framework
- [specta](https://github.com/oscartbeaumont/specta): TypeScript type generation
- [rspc](https://github.com/oscartbeaumont/rspc): RPC framework with type generation

---

## Conclusion

This refactor transforms specta-swift from a working prototype into production-ready, open-source grade software. By focusing on:

1. **Modular architecture** - Easy to understand and maintain
2. **Comprehensive testing** - Confidence in changes
3. **Excellent documentation** - Easy to onboard
4. **Tuple variant support** - Complete feature set
5. **Code quality** - Senior engineer standards

We'll create a library that developers love to use and contribute to.

**Let's build something great.** 🚀
