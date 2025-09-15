/// On tests, auto-generate own crates exports.
#[cfg(test)]
#[test]
fn test() {
	use crate::ItemImportExportUpdater;

	// Automatically generate imports and exports.
	let mut updater:ItemImportExportUpdater = ItemImportExportUpdater::new("src/lib.rs");
	updater.generate().unwrap();

	// Try to automatically generate dependency imports in the TOML file.
	const LIB_SOURCE_FILE_ENV_NAME:&str = "SFCM_LIB_SRC";
	if let Ok(source_dir) = std::env::var(LIB_SOURCE_FILE_ENV_NAME) {
		let libraries_storage:LibrariesStorage = LibrariesStorage::from_file(&source_dir);
		let found_lib_names:Vec<&str> = updater.recursive_imports().iter().map(|import| import.identifier.split("::").next().unwrap()).collect::<Vec<&str>>();
		generate_toml_imports("Cargo.toml", &found_lib_names, &libraries_storage).unwrap();
	}
}



// auto-exports
mod item_imports_and_exports;
mod library_imports;
mod data_structs;

pub use item_imports_and_exports::*; // ItemImportExportUpdater, MODULE_IMPORT_TAG, PARSER_EXPORT_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, PARSER_IDENTIFIER_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, imports_exports_parser
pub use library_imports::*; // generate_toml_imports, LibrariesStorage, Library
pub(crate) use data_structs::*; // Import, Export