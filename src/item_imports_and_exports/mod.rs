pub(super) const AUTO_EXPORTS_TAG:&str = "auto-exports";



// auto-exports
mod item_import_export_updater;
mod item_import_export_parser;

pub use item_import_export_updater::*; // ItemImportExportUpdater
pub use item_import_export_parser::*; // MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser