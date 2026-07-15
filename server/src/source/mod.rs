mod catalog;
mod list;
mod package;
mod protocol;
mod registry;
mod runtime;
mod service;

pub use catalog::{AvailableSource, SourceCatalogError, SourceCatalogService};
pub use list::{SourceList, SourceListEntry, SourceListError};
pub use package::{
    SourceCapabilities, SourceImport, SourceInfo, SourceInterface, SourceListing, SourceManifest,
    SourcePackage, SourcePackageError,
};
pub use protocol::{
    Chapter, ContentRating, FilterValue, ImageRef, ImageRequest, ImageResponse, Listing,
    ListingKind, Manga, MangaPageResult, MangaStatus, Page, PageContent, PageContext,
    UpdateStrategy, Viewer,
};
pub use registry::{InstalledSource, SourceRegistry, SourceRegistryError};
pub use runtime::{CompiledSource, SourceInstance, SourceRuntimeError};
pub use service::{SourceService, SourceServiceError};
