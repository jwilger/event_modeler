//! Advanced type safety utilities for compile-time guarantees.
//!
//! This module provides types and patterns that eliminate runtime validation
//! by encoding invariants in the type system. All validation happens at
//! system boundaries (parsing), and the rest of the application works with
//! types that maintain invariants by construction.
//!
//! # Key Concepts
//!
//! - **Phantom Types**: Zero-cost compile-time type markers
//! - **Parse, Don't Validate**: Validation happens once at boundaries
//! - **Make Illegal States Unrepresentable**: Invalid states cannot be constructed

use std::marker::PhantomData;
use std::path::PathBuf;

// Phantom types for file extensions

/// Marker type for Event Model files (.eventmodel extension).
#[derive(Debug, Clone, Copy)]
pub struct EventModelFile;

/// Marker type for Markdown files (.md extension).
#[derive(Debug, Clone, Copy)]
pub struct MarkdownFile;

/// Marker type for any file type (no extension restriction).
#[derive(Debug, Clone, Copy)]
pub struct AnyFile;

// Phantom types for path types

/// Marker type indicating a path points to a directory.
#[derive(Debug, Clone, Copy)]
pub struct Directory;

/// Marker type indicating a path points to a file.
#[derive(Debug, Clone, Copy)]
pub struct File;

/// Marker type indicating a path's existence is not verified.
#[derive(Debug, Clone, Copy)]
pub struct MaybeExists;

/// Marker type indicating a path has been verified to exist.
#[derive(Debug, Clone, Copy)]
pub struct Exists;

// Non-empty collection type

/// A collection that is guaranteed to have at least one element.
///
/// This type makes it impossible to have an empty collection at compile time,
/// eliminating the need for runtime empty checks.
///
/// # Examples
///
/// ```ignore
/// use event_modeler::type_safety::NonEmpty;
///
/// // Create a singleton
/// let single = NonEmpty::singleton("first");
/// assert_eq!(single.len(), 1);
///
/// // Create with multiple elements
/// let multiple = NonEmpty::from_head_and_tail("first", vec!["second", "third"]);
/// assert_eq!(multiple.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct NonEmpty<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmpty<T> {
    /// Creates a `NonEmpty` collection with a single element.
    pub fn singleton(value: T) -> Self {
        Self {
            head: value,
            tail: vec![],
        }
    }

    /// Creates a `NonEmpty` collection from a head element and a tail vector.
    ///
    /// The resulting collection will always have at least one element (the head).
    pub fn from_head_and_tail(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    /// Returns a reference to the first (head) element.
    ///
    /// This is guaranteed to exist and never panics.
    pub fn head(&self) -> &T {
        &self.head
    }

    /// Returns a slice of the tail elements (may be empty).
    pub fn tail(&self) -> &[T] {
        &self.tail
    }

    /// Returns an iterator over all elements in the collection.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }

    /// Returns the number of elements in the collection.
    ///
    /// Always returns at least 1.
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Returns false because NonEmpty collections are never empty.
    pub const fn is_empty(&self) -> bool {
        false
    }
}

// Type-safe path with phantom types

/// A path with compile-time guarantees about its type and properties.
///
/// This type uses phantom types to encode:
/// - File type (EventModelFile, MarkdownFile, AnyFile)
/// - Path type (Directory, File)
/// - Existence (Exists, MaybeExists)
///
/// # Type Parameters
///
/// - `FileType`: The type of file this path points to
/// - `PathType`: Whether this is a file or directory
/// - `ExistenceType`: Whether the path has been verified to exist
#[derive(Debug, Clone)]
pub struct TypedPath<FileType, PathType, ExistenceType> {
    path: PathBuf,
    _file_type: PhantomData<FileType>,
    _path_type: PhantomData<PathType>,
    _existence: PhantomData<ExistenceType>,
}

impl<F, P, E> TypedPath<F, P, E> {
    /// Returns the underlying `PathBuf`.
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.path
    }
}

// Builder for creating typed paths at compile time

/// Builder for parsing and validating paths at system boundaries.
///
/// This is the only way to create `TypedPath` instances, ensuring all
/// validation happens at parse time rather than construction time.
pub struct PathBuilder;

impl PathBuilder {
    /// Parses a path as an Event Model file.
    ///
    /// # Requirements
    ///
    /// - Must have `.eventmodel` extension
    /// - Must exist on the filesystem
    /// - Must be a file (not a directory)
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InvalidEventModelFile` if requirements are not met.
    pub fn parse_event_model_file(
        path: PathBuf,
    ) -> Result<TypedPath<EventModelFile, File, Exists>, ParseError> {
        // This validation happens once at system boundary
        if path.extension().is_some_and(|ext| ext == "eventmodel")
            && path.exists()
            && path.is_file()
        {
            Ok(TypedPath {
                path,
                _file_type: PhantomData,
                _path_type: PhantomData,
                _existence: PhantomData,
            })
        } else {
            Err(ParseError::InvalidEventModelFile)
        }
    }

    /// Parses a path as a Markdown file.
    ///
    /// # Requirements
    ///
    /// - Must have `.md` extension
    ///
    /// Note: Existence is not verified, allowing for documentation links
    /// that may not exist yet.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InvalidMarkdownFile` if the extension is not `.md`.
    pub fn parse_markdown_file(
        path: PathBuf,
    ) -> Result<TypedPath<MarkdownFile, File, MaybeExists>, ParseError> {
        if path.extension().is_some_and(|ext| ext == "md") {
            Ok(TypedPath {
                path,
                _file_type: PhantomData,
                _path_type: PhantomData,
                _existence: PhantomData,
            })
        } else {
            Err(ParseError::InvalidMarkdownFile)
        }
    }

    /// Parses a path as an existing directory.
    ///
    /// # Requirements
    ///
    /// - Must exist on the filesystem
    /// - Must be a directory (not a file)
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InvalidDirectory` if requirements are not met.
    pub fn parse_directory(
        path: PathBuf,
    ) -> Result<TypedPath<AnyFile, Directory, Exists>, ParseError> {
        if path.exists() && path.is_dir() {
            Ok(TypedPath {
                path,
                _file_type: PhantomData,
                _path_type: PhantomData,
                _existence: PhantomData,
            })
        } else {
            Err(ParseError::InvalidDirectory)
        }
    }

    /// Parses a path as an output directory.
    ///
    /// # Requirements
    ///
    /// - Parent directory must exist (or path has no parent)
    ///
    /// This allows for directories that don't exist yet but can be created.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InvalidOutputDirectory` if parent doesn't exist.
    pub fn parse_output_directory(
        path: PathBuf,
    ) -> Result<TypedPath<AnyFile, Directory, MaybeExists>, ParseError> {
        if path.parent().is_none_or(|p| p.exists()) {
            Ok(TypedPath {
                path,
                _file_type: PhantomData,
                _path_type: PhantomData,
                _existence: PhantomData,
            })
        } else {
            Err(ParseError::InvalidOutputDirectory)
        }
    }
}

/// Errors that can occur during parsing at system boundaries.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The path is not a valid Event Model file.
    #[error("Invalid event model file: must have .eventmodel extension and exist")]
    InvalidEventModelFile,

    /// The path is not a valid Markdown file.
    #[error("Invalid markdown file: must have .md extension")]
    InvalidMarkdownFile,

    /// The path is not a valid directory.
    #[error("Invalid directory: must exist and be a directory")]
    InvalidDirectory,

    /// The output directory path is invalid.
    #[error("Invalid output directory: parent must exist")]
    InvalidOutputDirectory,

    /// String cannot be empty.
    #[error("String cannot be empty")]
    EmptyString,

    /// Invalid identifier format.
    #[error("Invalid identifier format")]
    InvalidIdentifier,

    /// Event name must start with uppercase letter.
    #[error("Event name must start with uppercase letter")]
    InvalidEventName,

    /// Value must be greater than zero.
    #[error("Value must be greater than zero")]
    NotPositive,

    /// Port must be between 1 and 65535.
    #[error("Port must be between 1 and 65535")]
    InvalidPort,

    /// Float must be finite.
    #[error("Float must be finite")]
    NotFinite,

    /// Percentage must be between 0 and 100.
    #[error("Percentage must be between 0 and 100")]
    InvalidPercentage,
}

// Compile-time proof types

/// A compile-time proof of some property.
///
/// This type has no runtime cost and is used to track properties
/// at compile time using phantom types.
pub struct Proof<T>(PhantomData<T>);

impl<T> Default for Proof<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Proof<T> {
    /// Creates a new proof.
    ///
    /// This is a const function, allowing proofs to be created at compile time.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

// Entity existence proofs

/// Compile-time proof that an entity exists in a registry.
pub struct EntityExists<Id>(PhantomData<Id>);

/// Compile-time proof that an entity has been added to a registry.
pub struct EntityAdded<Id>(PhantomData<Id>);

// Compile-time safe string types

/// A compile-time guaranteed non-empty string.
///
/// This type can only be created through parsing at system boundaries,
/// eliminating runtime validation throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Parses a string, ensuring it's not empty.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(s: String) -> Result<Self, ParseError> {
        if s.is_empty() {
            Err(ParseError::EmptyString)
        } else {
            Ok(Self(s))
        }
    }

    /// Returns the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// A compile-time guaranteed identifier string.
///
/// Valid identifiers match the pattern `[a-zA-Z_][a-zA-Z0-9_]*`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// Parses a string as an identifier.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(s: String) -> Result<Self, ParseError> {
        if s.is_empty() {
            return Err(ParseError::EmptyString);
        }

        let chars: Vec<char> = s.chars().collect();
        if !chars[0].is_ascii_alphabetic() && chars[0] != '_' {
            return Err(ParseError::InvalidIdentifier);
        }

        for &ch in &chars[1..] {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(ParseError::InvalidIdentifier);
            }
        }

        Ok(Self(s))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A compile-time guaranteed event name (PascalCase).
///
/// Event names must start with an uppercase letter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventName(String);

impl EventName {
    /// Parses a string as an event name.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(s: String) -> Result<Self, ParseError> {
        if s.is_empty() {
            return Err(ParseError::EmptyString);
        }

        let first_char = s.chars().next().unwrap();
        if !first_char.is_ascii_uppercase() {
            return Err(ParseError::InvalidEventName);
        }

        Ok(Self(s))
    }

    /// Returns the event name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Compile-time safe numeric types

/// A non-negative integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonNegativeInt(u32);

impl NonNegativeInt {
    /// Creates a non-negative integer.
    ///
    /// Since u32 is always non-negative, this is infallible.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the inner value.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A positive (greater than zero) integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveInt(u32);

impl PositiveInt {
    /// Parses a u32 as a positive integer.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: u32) -> Result<Self, ParseError> {
        if value == 0 {
            Err(ParseError::NotPositive)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the inner value.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A port number (1-65535).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Port(u16);

impl Port {
    /// Parses a u16 as a valid port number.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: u16) -> Result<Self, ParseError> {
        if value == 0 {
            Err(ParseError::InvalidPort)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the port number.
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// A finite floating-point number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteFloat(f32);

impl FiniteFloat {
    /// Parses a f32 as a finite float.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: f32) -> Result<Self, ParseError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ParseError::NotFinite)
        }
    }

    /// Returns the inner value.
    pub const fn value(self) -> f32 {
        self.0
    }
}

/// A percentage value (0.0 to 100.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f32);

impl Percentage {
    /// Parses a f32 as a percentage.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: f32) -> Result<Self, ParseError> {
        if (0.0..=100.0).contains(&value) && value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ParseError::InvalidPercentage)
        }
    }

    /// Returns the percentage value.
    pub const fn value(self) -> f32 {
        self.0
    }
}

/// A positive (greater than zero) floating-point number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PositiveFloat(f32);

impl PositiveFloat {
    /// Parses a f32 as a positive float.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: f32) -> Result<Self, ParseError> {
        if value > 0.0 && value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ParseError::NotPositive)
        }
    }

    /// Returns the inner value.
    pub const fn value(self) -> f32 {
        self.0
    }
}

/// A non-negative (zero or greater) floating-point number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NonNegativeFloat(f32);

impl NonNegativeFloat {
    /// Parses a f32 as a non-negative float.
    ///
    /// This should only be called at system boundaries.
    pub fn parse(value: f32) -> Result<Self, ParseError> {
        if value >= 0.0 && value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ParseError::NotPositive)
        }
    }

    /// Returns the inner value.
    pub const fn value(self) -> f32 {
        self.0
    }
}
