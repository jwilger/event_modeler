use std::marker::PhantomData;
use std::path::PathBuf;

// Phantom types for file extensions
#[derive(Debug, Clone, Copy)]
pub struct EventModelFile;
#[derive(Debug, Clone, Copy)]
pub struct MarkdownFile;
#[derive(Debug, Clone, Copy)]
pub struct AnyFile;

// Phantom types for path types
#[derive(Debug, Clone, Copy)]
pub struct Directory;
#[derive(Debug, Clone, Copy)]
pub struct File;
#[derive(Debug, Clone, Copy)]
pub struct MaybeExists;
#[derive(Debug, Clone, Copy)]
pub struct Exists;

// Non-empty collection type
#[derive(Debug, Clone)]
pub struct NonEmpty<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmpty<T> {
    pub fn singleton(value: T) -> Self {
        Self {
            head: value,
            tail: vec![],
        }
    }
    
    pub fn from_head_and_tail(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }
    
    pub fn head(&self) -> &T {
        &self.head
    }
    
    pub fn tail(&self) -> &[T] {
        &self.tail
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
    
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }
}

// Type-safe path with phantom types
#[derive(Debug, Clone)]
pub struct TypedPath<FileType, PathType, ExistenceType> {
    path: PathBuf,
    _file_type: PhantomData<FileType>,
    _path_type: PhantomData<PathType>,
    _existence: PhantomData<ExistenceType>,
}

impl<F, P, E> TypedPath<F, P, E> {
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.path
    }
}

// Builder for creating typed paths at compile time
pub struct PathBuilder;

impl PathBuilder {
    pub fn parse_event_model_file(path: PathBuf) -> Result<TypedPath<EventModelFile, File, Exists>, ParseError> {
        // This validation happens once at system boundary
        if path.extension().map_or(false, |ext| ext == "eventmodel") && path.exists() && path.is_file() {
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
    
    pub fn parse_markdown_file(path: PathBuf) -> Result<TypedPath<MarkdownFile, File, MaybeExists>, ParseError> {
        if path.extension().map_or(false, |ext| ext == "md") {
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
    
    pub fn parse_directory(path: PathBuf) -> Result<TypedPath<AnyFile, Directory, Exists>, ParseError> {
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
    
    pub fn parse_output_directory(path: PathBuf) -> Result<TypedPath<AnyFile, Directory, MaybeExists>, ParseError> {
        if path.parent().map_or(true, |p| p.exists()) {
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

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid event model file: must have .eventmodel extension and exist")]
    InvalidEventModelFile,
    
    #[error("Invalid markdown file: must have .md extension")]
    InvalidMarkdownFile,
    
    #[error("Invalid directory: must exist and be a directory")]
    InvalidDirectory,
    
    #[error("Invalid output directory: parent must exist")]
    InvalidOutputDirectory,
}

// Compile-time proof types
pub struct Proof<T>(PhantomData<T>);

impl<T> Proof<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

// Entity existence proofs
pub struct EntityExists<Id>(PhantomData<Id>);
pub struct EntityAdded<Id>(PhantomData<Id>);