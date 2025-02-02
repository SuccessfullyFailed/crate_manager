use omni_parser::{ NestedCodeParser, NestedSegment, NestedSegmentCode };
use crate::files::{CodeFile, ModFile};
use std::error::Error;



const AUTO_EXPORT_TAG:&str = "// auto-export";



/// Automatically create exports for the crate in the given directory.
pub(crate) fn generate_exports_for_mod(mod_file:&mut ModFile) -> Result<(), Box<dyn Error>> {

	// Parse sub-mods first.
	for sub_mod in &mut mod_file.sub_mod_files {
		generate_exports_for_mod(sub_mod)?;
	}

	// Validate file contains export tag.
	let original_mod_file_code:&str = mod_file.file.get_contents()?;
	if !original_mod_file_code.contains(AUTO_EXPORT_TAG) {
		return Ok(());
	}

	// Create a list of all sources found and another containing all sources that have public exports.
	let mut sources:Vec<(String, bool)> = [
		mod_file.source_files.iter_mut().map(|file| (
			file.path.file_name_no_extension().to_string(),
			file_contains_pub_exports(file).unwrap_or(false
		))).collect::<Vec<(String, bool)>>(),
		mod_file.sub_mod_files.iter_mut().map(|mod_file| (
			mod_file.file.path.parent_dir().unwrap().file_name_no_extension().to_string(),
			file_contains_pub_exports(&mut mod_file.file).unwrap_or(false)
		)).collect::<Vec<(String, bool)>>()
	].iter().flatten().cloned().collect::<Vec<(String, bool)>>();
	sources.sort_by(|a, b| a.0.len().cmp(&b.0.len()));

	// Generate and update mod-file code.
	let prefix:&str = original_mod_file_code.split(AUTO_EXPORT_TAG).next().unwrap_or("");
	let new_mode_code:String =format!(
		"{}{}\n{}\n{}",
		prefix,
		AUTO_EXPORT_TAG,
		sources.iter().map(|source| format!("mod {};", &source.0)).collect::<Vec<String>>().join("\n"),
		sources.iter().filter(|source| source.1).map(|source| format!("pub use {}::*;", &source.0)).collect::<Vec<String>>().join("\n")
	);
	if new_mode_code != original_mod_file_code {
		mod_file.file.mod_contents(move |contents| *contents = new_mode_code.clone())?;
	}

	// Return success.
	Ok(())
}

/// Check if a code snipper contains public exports in the surface level.
fn file_contains_pub_exports(file:&mut CodeFile) -> Result<bool, Box<dyn Error>> {

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
					&("scope", true, "{", "}"),
					&("export", r#"^pub\s(struct|enum|fn|trait|impl|mod|const|static|type|use|crate|macro)\s"#)
				]));
				RUST_CODE_PARSER.as_mut().unwrap()
			}
		}
	};

	// Find any public exports in the non-nested code.
	let parser_result:NestedSegment = rust_code_parser.parse(file.get_contents()?);
	let surface_exports:Vec<(usize, &NestedSegmentCode)> = parser_result.flat_code_filtered(|depth, code| depth == 1 && &code.type_name == "export");
	Ok(!surface_exports.is_empty())
}