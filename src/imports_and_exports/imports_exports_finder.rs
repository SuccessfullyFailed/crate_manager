use std::error::Error;
use file_ref::FileRef;

use crate::{ imports_and_exports::{imports_exports_parser, AUTO_EXPORTS_TAG}, MODULE_IMPORT_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, PARSER_EXPORT_TAG, PARSER_IDENTIFIER_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG};



struct Import {
	pub_type:Option<String>,
	struct_type:String,
	identifier:String
}
struct Export {
	pub_type:Option<String>,
	struct_type:String,
	identifier:String
}



pub struct ExportsFinder {
	file:FileRef,
	parsed:bool,
	exports_trigger_location:Option<usize>,
	imports:Vec<Import>,
	exports:Vec<Export>,
	sub_finders:Vec<ExportsFinder>
}



impl ExportsFinder {

	/// Create a new exports finder.
	pub fn new(file:FileRef) -> ExportsFinder {
		ExportsFinder {
			file: file.absolute(),
			parsed: false,
			exports_trigger_location: None,
			imports: Vec::new(),
			exports: Vec::new(),
			sub_finders: Vec::new()
		}
	}

	/// Find all exports for this file.
	pub fn find_all(&mut self) -> Result<(), Box<dyn Error>> {

		// If already parsed, don't parse again.
		if self.parsed {
			return Ok(());
		}
		self.parsed = true;

		// Read and parse file.
		self.exports_trigger_location = None;
		self.imports = Vec::new();
		self.exports = Vec::new();
		let file_contents:String = self.file.read()?;
		for (match_cursor, match_result) in imports_exports_parser().find_matches(&file_contents) {

			// Imports and exports.
			if match_result.type_name == MODULE_IMPORT_TAG || match_result.type_name == PARSER_EXPORT_TAG {
				let pub_type:Option<String> = match_result.find_child_by_type_path(&[PARSER_PUB_TYPE_TAG]).map(|child| child.contents.clone());
				let struct_type:String = match_result.find_child_by_type_path(&[PARSER_TYPE_TAG]).unwrap().contents.clone();
				let identifier:String = match_result.find_child_by_type_path(&[PARSER_IDENTIFIER_TAG]).unwrap().contents.clone();

				if match_result.type_name == MODULE_IMPORT_TAG {
					self.imports.push(Import { pub_type: pub_type.clone(), struct_type: struct_type.clone(), identifier: identifier.clone() });
				}
				if match_result.type_name == PARSER_EXPORT_TAG {
					self.exports.push(Export { pub_type, struct_type, identifier });
				}
			}

			// Auto-exporting trigger.
			if match_result.type_name == PARSER_AUTO_EXPORTS_TRIGGER_TAG {
				self.exports_trigger_location = Some(match_cursor);
			}
		}

		// Parse other linked files.
		self.sub_finders = Vec::new();
		for import in self.imports.iter().filter(|import| import.struct_type == "mod") {
			let next_file_refix:FileRef = self.file.parent_dir()? + "/" + &import.identifier;
			for next_file in [next_file_refix.clone() + ".rs", next_file_refix.clone() + "/mod.rs"] {
				if next_file.exists() {
					self.sub_finders.push(ExportsFinder::new(next_file));
				}
			}
		}
		if self.file.name() == "lib.rs" || self.file.name() == "mod.rs" {
			for file in self.file.parent_dir()?.scanner().include_files().filter(|file| file.name() != "mod.rs" && file.name() != "lib.rs" && file.extension() == Some("rs")) {
				if self.sub_finders.iter().find(|sub_finder| sub_finder.file == file).is_none() {
					self.sub_finders.push(ExportsFinder::new(file));
				}
			}
			for file in self.file.parent_dir()?.list_dirs().into_iter().map(|dir| dir + "/mod.rs").filter(|file| file.exists()) {
				if self.sub_finders.iter().find(|sub_finder| sub_finder.file == file).is_none() {
					self.sub_finders.push(ExportsFinder::new(file));
				}
			}
		}
		for sub_finder in &mut self.sub_finders {
			sub_finder.find_all()?;
		}

		// If auto-exports trigger found, generate exports.
		if let Some(cursor) = self.exports_trigger_location {
			let mut generated_exports:Vec<(String, Vec<String>)> = Vec::new();
			for sub_finder in &self.sub_finders {
				let file_name:&str = sub_finder.file.file_name_no_extension();
				generated_exports.push((
					if file_name == "mod" {
						sub_finder.file.parent_dir()?.file_name_no_extension().to_string()
					} else {
						sub_finder.file.file_name_no_extension().to_string()
					},
					sub_finder.recursive_exports().into_iter().map(|export| export.identifier.clone()).collect::<Vec<String>>()
				));
			}
			generated_exports.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
			let new_contents:String = format!(
				"{}// {}\n{}\n\n{}",
				file_contents[..cursor].to_string(),
				AUTO_EXPORTS_TAG,
				generated_exports.iter().map(|(mod_name, _item_names)| format!("mod {mod_name};")).collect::<Vec<String>>().join("\n"),
				generated_exports.iter().map(|(mod_name, item_names)| format!("pub use {}::{} {} {};", mod_name, '{', item_names.join(", "), '}')).collect::<Vec<String>>().join("\n"),
			);
			if new_contents != file_contents {
				self.file.write(new_contents)?;
			}
		}

		// Return success.
		Ok(())
	}

	/// Get all exports of this finder and all sub-finders.
	fn recursive_exports(&self) -> Vec<&Export> {
		[
			self.exports.iter().collect::<Vec<&Export>>(),
			self.sub_finders.iter().map(|finder| finder.recursive_exports()).flatten().collect::<Vec<&Export>>()
		].into_iter().flatten().collect()
	}
}