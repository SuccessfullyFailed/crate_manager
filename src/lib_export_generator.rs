use omni_parser::NestedSegment;
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
	let mod_file_contents:&NestedSegment = mod_file.file.contents()?;
	let mod_file_contents_flat:Vec<(usize, &NestedSegment)> = mod_file_contents.flat();
	let export_tag_index:Option<usize> = mod_file_contents_flat.iter().position(|(_, segment)| segment.type_name().contains("comment") && segment.to_string().contains(AUTO_EXPORT_TAG));
	if export_tag_index.is_none() {
		return Ok(());
	}
	let export_tag_index:usize = export_tag_index.unwrap();

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

	// Generate new export segments.
	let current_exports:Vec<NestedSegment> = if export_tag_index == mod_file_contents_flat.len() { Vec::new() } else { mod_file_contents_flat[export_tag_index + 1..].iter().map(|(_, segment)| (*segment).clone()).collect::<Vec<NestedSegment>>() };
	let new_exports:Vec<NestedSegment> = [
		sources.iter().map(|source| NestedSegment::new_code("mod_import", &format!("mod {};", &source.0), Vec::new(), "")).collect::<Vec<NestedSegment>>(),
		sources.iter().filter(|source| source.1).map(|source| NestedSegment::new_code("export", &format!("pub use {}::*;", &source.0), Vec::new(), "")).collect::<Vec<NestedSegment>>()
	].iter().flatten().cloned().collect();

	// Replace and write new segments.
	if new_exports != current_exports {
		let mut mod_file_contents:Vec<(usize, NestedSegment)> = mod_file.file.contents_mut()?.flat().iter().map(|(depth, segment)| (*depth, (*segment).clone())).collect::<Vec<(usize, NestedSegment)>>();
		mod_file_contents.drain(export_tag_index + 1..);
		mod_file_contents.extend(new_exports.iter().map(|export| (0, export.clone())).collect::<Vec<(usize, NestedSegment)>>());
		*mod_file.file.contents_mut()? = NestedSegment::from_flat(mod_file_contents).unwrap();
	}

	// Return success.
	Ok(())
}

/// Check if a code snipper contains public exports in the surface level.
fn file_contains_pub_exports(file:&mut CodeFile) -> Result<bool, Box<dyn Error>> {
	Ok(!file.contents()?.flat_code_filtered(|depth, code| depth == 1 && &code.type_name == "export").is_empty())
}