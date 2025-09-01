pub const AUTO_EXPORTS_TAG:&str = "auto-exports";



// auto-exports
mod parser;
mod imports_exports_finder;

pub use parser::{ MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser };
pub use imports_exports_finder::{ ExportsFinder };