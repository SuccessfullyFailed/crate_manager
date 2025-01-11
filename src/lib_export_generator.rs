use std::error::Error;
use file_ref::FileRef;



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
	mod_files.sort_by(|a, b| b.len().cmp(&a.len()));
	
	// Generate exports for all mod files that contain the auto-export tag.
	for mod_file in mod_files {
		let original_mod_file_code:String = mod_file.read()?;
		if !original_mod_file_code.contains(AUTO_EXPORT_TAG) {
			continue;
		}
		let mod_dir:FileRef = mod_file.parent_dir()?;
		let source_files:Vec<FileRef> = mod_dir.scanner().include_files().filter(|file| file.name() != LIB_FILE_NAME && file.name() != MOD_FILE_NAME).collect();
		let sub_mod_files:Vec<(FileRef, FileRef)> = mod_dir.scanner().include_dirs().map(|dir| (dir.clone(), dir.clone() + "/" + MOD_FILE_NAME)).filter(|(_, mod_file)| mod_file.exists()).collect();
		let source_dirs:Vec<FileRef> = sub_mod_files.iter().map(|(dir, _)| dir.clone()).collect::<Vec<FileRef>>();

		// Create a list of all sources found and another containing all sources that have public exports.
		let all_source_names:Vec<String> = [source_files.clone(), source_dirs.clone()].iter().flatten().map(|file| file.file_name_no_extension().to_owned()).collect();
		let source_files_with_exports:Vec<&FileRef> = source_files.iter().filter(|&file| file_contains_pub_exports(file).unwrap_or(false)).collect::<Vec<&FileRef>>();
		let source_dirs_with_exports:Vec<&FileRef> = sub_mod_files.iter().filter(|&(_, mod_file)| file_contains_pub_exports(mod_file).unwrap_or(false)).map(|(dir, _)| dir).collect::<Vec<&FileRef>>();
		let sources_with_exports:Vec<String> = [source_files_with_exports, source_dirs_with_exports].iter().flatten().map(|file| file.file_name_no_extension().to_owned()).collect();

		// Generate and mod-file code.
		let prefix:&str = original_mod_file_code.split(AUTO_EXPORT_TAG).next().unwrap_or("");
		let new_mod_file_contents:String = format!(
			"{}{}\n{}\n{}",
			prefix,
			AUTO_EXPORT_TAG,
			all_source_names.iter().map(|source| format!("mod {source};")).collect::<Vec<String>>().join("\n"),
			sources_with_exports.iter().map(|source| format!("pub use {source}::*;")).collect::<Vec<String>>().join("\n")
		);

		// If generated code does not match current contents, overwrite.
		if original_mod_file_code != new_mod_file_contents {
			mod_file.write_await(&new_mod_file_contents)?;
		}
	}

	// Return success.
	Ok(())
}

/// Check if a code snipper contains public exports in the surface level.
fn file_contains_pub_exports(file:&FileRef) -> Result<bool, Box<dyn Error>> {
	use omni_parser::{ NestedCodeParser, NestedCode };
	use regex::Regex;

	// Keep a static version of the export finding regex.
	static mut EXPORT_REGEX:Option<Regex> = None;
	let export_regex:&Regex = unsafe {
		match EXPORT_REGEX.as_mut() {
			Some(regex) => regex,
			None => {
				EXPORT_REGEX = Some(Regex::new(r#"(^|\s)pub\s(struct|enum|fn|trait|impl|mod|const|static|type|use|crate|macro)\s"#).unwrap());
				EXPORT_REGEX.as_ref().unwrap()
			}
		}
	};

	// Keep a static version of a rust code parser.
	static mut RUST_CODE_PARSER:Option<NestedCodeParser> = None;
	let rust_code_parser:&NestedCodeParser = unsafe {
		match RUST_CODE_PARSER.as_mut() {
			Some(parser) => parser,
			None => {
				RUST_CODE_PARSER = Some(NestedCodeParser::new(vec![
					&("doc-comment", false, "///", "\n"),
					&("single-line-comment", false, "//", "\n"),
					&("multi-line-comment", false, "/*", "*/"),
					&("quote", false, "\"", Some("\\"), "\"", Some("\\")),
					&("scope", true, "{", "}")
				]));
				RUST_CODE_PARSER.as_mut().unwrap()
			}
		}
	};

	// Find any public exports in the non-nested code.
	let result:NestedCode = rust_code_parser.parse(&file.read()?)?;
	let surface_code:String = result.contents().iter().filter(|segment| !segment.matched()).map(|segment| segment.contents_joined()).collect::<Vec<String>>().join("");
	Ok(export_regex.is_match(&surface_code))
}