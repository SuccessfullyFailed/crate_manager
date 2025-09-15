/// On tests, auto-generate own crates exports.
#[cfg(test)]
#[test]
fn test() {
	use crate::ImportExportUpdater;

	let mut updater:ImportExportUpdater = ImportExportUpdater::new("src/lib.rs");
	updater.generate().unwrap();
}



// auto-exports
mod imports_and_exports;

pub use imports_and_exports::{ MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser, ImportExportUpdater, LibrariesStorage, Library };
pub(crate) use imports_and_exports::{ Import, Export };