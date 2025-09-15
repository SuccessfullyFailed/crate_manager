pub(super) const AUTO_EXPORTS_TAG:&str = "auto-exports";



// auto-exports
mod import_export_updater;
mod libraries_storage_u;
mod libraries_storage;
mod parser;

pub use import_export_updater::{ ImportExportUpdater };

pub use libraries_storage::{ LibrariesStorage, Library };
pub use parser::{ MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser };