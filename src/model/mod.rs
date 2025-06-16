pub mod entities;
pub mod diagram;
pub mod registry;

pub use diagram::{EventModelDiagram, DiagramMetadata};
pub use entities::{Wireframe, Command, Event, Projection, Query, Automation};
pub use registry::{EntityRegistry, EntityRef};