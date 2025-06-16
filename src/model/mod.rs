pub mod entities;
pub mod diagram;

pub use diagram::{EventModelDiagram, DiagramMetadata};
pub use entities::{Wireframe, Command, Event, Projection, Query, Automation};