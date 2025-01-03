use file_ref::FileRef;
use std::error::Error;



const AUTO_EXPORT_TAG:&str = "// auto-export";

const CRATE_ROOT_DETECTOR_FILE_NAME:&str = "Cargo.toml";
const CRATE_SOURCE_DIR:&str = "src";
const LIB_FILE_NAME:&str = "lib.rs";
const MOD_FILE_NAME:&str = "mod.rs";
const DIR_RECURSE_EXCEPTIONS:&[&str] = &["target"];




/// Automatically create exports for all crates in the working dir.
pub fn generate_exports_for_crates_in_working_dir() -> Result<(), Box<dyn Error>> {
	generate_exports_for_crates_in(FileRef::working_dir().path())
}

/// Automatically create exports for all crates in the given directory.
pub fn generate_exports_for_crates_in(root_directory_path:&str) -> Result<(), Box<dyn Error>> {
	for crate_dir in crates_in(root_directory_path) {
		generate_exports_for_crate(crate_dir.path())?;
	}
	Ok(())
}

/// Get all crate dirs in a directory.
fn crates_in(root_directory_path:&str) -> Vec<FileRef> {
	FileRef::new(root_directory_path)
		.absolute()
		.scanner()
		.include_self()
		.include_dirs()
		.recurse_filter(|sub_dir| !DIR_RECURSE_EXCEPTIONS.contains(&sub_dir.name()))
		.filter(|crate_dir| (crate_dir.clone() + "/" + CRATE_ROOT_DETECTOR_FILE_NAME).exists() && (crate_dir.clone() + "/" + CRATE_SOURCE_DIR + "/" + LIB_FILE_NAME).exists())
		.collect()
}

/// Automatically create exports for the crate in the given directory.
pub fn generate_exports_for_crate(crate_path:&str) -> Result<(), Box<dyn Error>> {

	// Validate path is a library crate root dir.
	let crate_dir:FileRef = FileRef::new(crate_path);
	let src_dir:FileRef = crate_dir.clone() + "/" + CRATE_SOURCE_DIR;
	let lib_file:FileRef = src_dir.clone() + "/" + LIB_FILE_NAME;
	if !(crate_dir + "/" + CRATE_ROOT_DETECTOR_FILE_NAME).exists() {
		return Err(format!("Could not generate exports for for crate at '{}'. Crate root path should contain a '{}' file.", crate_path, CRATE_ROOT_DETECTOR_FILE_NAME).into());
	}
	if !src_dir.exists() {
		return Err(format!("Could not generate exports for for crate at '{}'. Crate root does not contains a '{}' dir.", crate_path, CRATE_SOURCE_DIR).into());
	}
	if !lib_file.exists() {
		return Err(format!("Could not generate exports for for crate at '{}'. Crate root does not contains a '{}/{}' file.", crate_path, CRATE_SOURCE_DIR, LIB_FILE_NAME).into());
	}

	// Find all linked lib.rs and mod.rs files.
	let mut mod_files:Vec<FileRef> = vec![lib_file];
	let mut undiscovered_dirs:Vec<FileRef> = src_dir.list_dirs();
	while !undiscovered_dirs.is_empty() {
		let dir:FileRef = undiscovered_dirs.remove(0);
		let mod_file:FileRef = dir.clone() + "/" + MOD_FILE_NAME;
		if mod_file.exists() {
			mod_files.push(mod_file);
			undiscovered_dirs.extend_from_slice(&dir.list_dirs());
		}
	}
	
	// Generate exports for all mod files that contain the auto-export tag.
	for mod_file in mod_files {
		let original_mod_file_code:String = mod_file.read()?;
		if !original_mod_file_code.contains(AUTO_EXPORT_TAG) {
			continue;
		}
		let mod_dir:FileRef = mod_file.parent_dir()?;
		let source_files:Vec<FileRef> = mod_dir.scanner().include_files().filter(|file| file.name() != LIB_FILE_NAME && file.name() != MOD_FILE_NAME).collect();
		let sub_mod_dirs:Vec<FileRef> = mod_dir.scanner().include_dirs().filter(|dir| (dir.clone() + "/" + MOD_FILE_NAME).exists()).collect();

		// Generate and mod-file code.
		let sources:Vec<String> = [source_files, sub_mod_dirs].iter().flatten().map(|file| file.file_name_no_extension().to_owned()).collect();
		let prefix:&str = original_mod_file_code.split(AUTO_EXPORT_TAG).next().unwrap_or("");
		let new_mod_file_contents:String = format!(
			"{}{}\n{}\n{}",
			prefix,
			AUTO_EXPORT_TAG,
			sources.iter().map(|source| format!("mod {source};")).collect::<Vec<String>>().join("\n"),
			sources.iter().map(|source| format!("pub use {source}::*;")).collect::<Vec<String>>().join("\n")
		);

		// If generated code does not match current contents, overwrite.
		if original_mod_file_code != new_mod_file_contents {
			mod_file.write(&new_mod_file_contents)?;
		}
	}

	// Return success.
	Ok(())
}