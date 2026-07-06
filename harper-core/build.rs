//! Harper Core Build Script
//!
//! This build script is the entry point for Harper's compile-time code generation system.
//! It delegates to build/main.rs which contains the main orchestration logic.
//!
//! ## Build System Structure
//!
//! - `build.rs`: This entry point file with documentation
//! - `build/main.rs`: Main orchestration logic
//! - `build/build_lib/`: Build library modules:
//!   - `language_config.rs`: Language discovery and configuration
//!   - `language_modules.rs`: Language integration code generation
//!   - `weir_rules.rs`: Weir rule processing
//!
//! ## What This Build Script Does
//!
//! 1. **Weir Rule Processing**: Compiles and prepares Weir grammar rules
//! 2. **Language Integration**: Generates code to integrate language modules
//! 3. **Feature-based Compilation**: Conditional compilation for optional languages
//!
//! The generated code enables Harper to support multiple languages while keeping
//! the main codebase clean and maintainable.

include!("build/main.rs");
