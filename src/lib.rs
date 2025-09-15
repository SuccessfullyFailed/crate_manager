/// On tests, auto-generate own crates exports.
#[cfg(test)]
#[test]
fn test() {
	crate::ImportExportUpdater::new(file_ref::FileRef::working_dir() + "/src/lib.rs").generate().unwrap();
}



// auto-exports
mod imports_and_exports;

pub use imports_and_exports::{ ImportExportUpdater, LibrariesStorage, Library, MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser };