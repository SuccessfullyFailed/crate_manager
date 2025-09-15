use crate::{ MODULE_IMPORT_TAG, PARSER_AUTO_EXPORTS_TRIGGER_TAG, PARSER_EXPORT_TAG, PARSER_IDENTIFIER_TAG, PARSER_PUB_TYPE_TAG, PARSER_TYPE_TAG, imports_and_exports::{ imports_exports_parser, AUTO_EXPORTS_TAG } };
use std::error::Error;
use file_ref::FileRef;



#[derive(PartialEq, Clone)]
enum PubType { Pub, Super, Crate }
impl PubType {
	pub fn from_str(contents:&str) -> PubType {
		if contents.contains("crate") {
			PubType::Crate
		} else if contents.contains("super") {
			PubType::Super
		} else {
			PubType::Pub
		}
	}
	pub fn to_str(&self) -> &str {
		match self {
			PubType::Pub => "pub",
			PubType::Super => "pub(super)",
			PubType::Crate => "pub(crate)",
		}
	}
}

#[derive(Clone)]
#[allow(dead_code)]
struct Import {
	pub_type:Option<PubType>,
	struct_type:String,
	identifier:String
}

#[derive(Clone)]
#[allow(dead_code)]
struct Export {
	pub_type:Option<PubType>,
	struct_type:String,
	identifier:String
}



pub struct ImportExportUpdater {
	file:FileRef,
	is_mod_file:bool,
	parsed:bool,
	imports:Vec<Import>,
	exports:[(PubType, Vec<Export>); 3],
	sub_finders:Vec<ImportExportUpdater>
}
impl ImportExportUpdater {

	/// Create a new exports finder.
	pub fn new(file:FileRef) -> ImportExportUpdater {
		ImportExportUpdater {
			file: file.clone().absolute(),
			is_mod_file: file.name() == "lib.rs" || file.name() == "mod.rs",
			parsed: false,
			imports: Vec::new(),
			exports: [(PubType::Pub, Vec::new()), (PubType::Crate, Vec::new()), (PubType::Super, Vec::new())],
			sub_finders: Vec::new()
		}
	}

	/// Find all imports and exports for this file.
	pub fn generate(&mut self) -> Result<(), Box<dyn Error>> {

		// If already parsed, don't parse again.
		if self.parsed {
			return Ok(());
		}
		self.parsed = true;

		// Read and parse file.
		let mut exports_trigger_location:Option<usize> = None;
		self.imports = Vec::new();
		self.exports.iter_mut().for_each(|(_, list)| *list = Vec::new());
		let file_contents:String = self.file.read()?;
		for (match_cursor, match_result) in imports_exports_parser().find_matches(&file_contents) {

			// Imports and exports.
			if match_result.type_name == MODULE_IMPORT_TAG || match_result.type_name == PARSER_EXPORT_TAG {
				let pub_type:Option<PubType> = match_result.find_child_by_type_path(&[PARSER_PUB_TYPE_TAG]).map(|child| PubType::from_str(&child.contents));
				let struct_type:String = match_result.find_child_by_type_path(&[PARSER_TYPE_TAG]).unwrap().contents.clone();
				let identifier:String = match_result.find_child_by_type_path(&[PARSER_IDENTIFIER_TAG]).unwrap().contents.clone();

				if match_result.type_name == MODULE_IMPORT_TAG {
					self.imports.push(Import { pub_type: pub_type.clone(), struct_type: struct_type.clone(), identifier: identifier.clone() });
				}
				if match_result.type_name == PARSER_EXPORT_TAG {
					if let Some(pub_type) = pub_type {
						if let Some((_, list)) = self.exports.iter_mut().find(|(list_pub_type, _)| *list_pub_type == pub_type) {
							list.push(Export { pub_type: Some(pub_type), struct_type, identifier });
						}
					}
				}
			}

			// Auto-exporting trigger.
			if match_result.type_name == PARSER_AUTO_EXPORTS_TRIGGER_TAG {
				exports_trigger_location = Some(match_cursor);
			}
		}

		// Parse other linked files.
		self.sub_finders = Vec::new();
		for import in self.imports.iter().filter(|import| import.struct_type == "mod") {
			let next_file_refix:FileRef = self.file.parent_dir()? + "/" + &import.identifier;
			for next_file in [next_file_refix.clone() + ".rs", next_file_refix.clone() + "/mod.rs"] {
				if next_file.exists() {
					self.sub_finders.push(ImportExportUpdater::new(next_file));
				}
			}
		}
		if self.is_mod_file {
			for file in self.file.parent_dir()?.scanner().include_files().filter(|file| file.name() != "mod.rs" && file.name() != "lib.rs" && file.extension() == Some("rs")) {
				if self.sub_finders.iter().find(|sub_finder| sub_finder.file == file).is_none() {
					self.sub_finders.push(ImportExportUpdater::new(file));
				}
			}
			for file in self.file.parent_dir()?.list_dirs().into_iter().map(|dir| dir + "/mod.rs").filter(|file| file.exists()) {
				if self.sub_finders.iter().find(|sub_finder| sub_finder.file == file).is_none() {
					self.sub_finders.push(ImportExportUpdater::new(file));
				}
			}
		}
		for sub_finder in &mut self.sub_finders {
			sub_finder.generate()?;
		}

		// Handle auto-exports if tag present.
		if let Some(cursor) = exports_trigger_location {
			self.generate_auto_exports(&file_contents, cursor)?;
		}

		// Return success.
		Ok(())
	}

	/// Get all exports of this finder and all sub-finders.
	fn recursive_exports(&self) -> Vec<&(PubType, Vec<Export>)> {
		[
			self.exports.iter().collect::<Vec<&(PubType, Vec<Export>)>>(),
			self.sub_finders.iter().map(|finder| finder.recursive_exports()).flatten().collect::<Vec<&(PubType, Vec<Export>)>>()
		].into_iter().flatten().collect()
	}



	/// Generate auto-exports for this file. Does nothing if the file does not contain the auto-exports tag.
	fn generate_auto_exports(&self, file_contents:&str, exports_trigger_location:usize) -> Result<(), Box<dyn Error>> {

		// Collect exports by mod_name, then pub type, then items.
		let mut item_exports:Vec<(String, [(PubType, Vec<Export>); 3])> = Vec::new();
		for sub_finder in &self.sub_finders {
			let file_name:&str = sub_finder.file.file_name_no_extension();
			let mod_name:String = if file_name == "mod" || file_name == "lib" { sub_finder.file.parent_dir()?.file_name_no_extension().to_string() } else { sub_finder.file.file_name_no_extension().to_string() };
			let list_index:usize = match item_exports.iter().position(|(list_mod_name, _)| list_mod_name == &mod_name) {
				Some(index) => index,
				None => {
					item_exports.push((mod_name.clone(), [(PubType::Pub, Vec::new()), (PubType::Crate, Vec::new()), (PubType::Super, Vec::new())]));
					item_exports.len() - 1
				}
			};
			for (export_set_pub_type, export_set_items) in sub_finder.recursive_exports() {
				if let Some((_, list)) = item_exports[list_index].1.iter_mut().find(|(list_pub_type, _)| list_pub_type == export_set_pub_type) {
					list.extend(export_set_items.clone());
				}
			}
		}

		// Sort items by length of name.
		item_exports.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

		// Generate and store new contents.
		let new_contents:String = format!(
			"{}// {}\n{}\n\n{}",
			&file_contents[..exports_trigger_location],
			AUTO_EXPORTS_TAG,
			item_exports
				.iter()
				.map(|(mod_name, _)| format!("mod {mod_name};"))
				.collect::<Vec<String>>()
				.join("\n"),
			item_exports
				.iter()
				.map(|(mod_name, pub_typed_items)|
					pub_typed_items
						.iter()
						.filter(|(pub_type, items)| !items.is_empty() && !(pub_type == &PubType::Super && self.is_mod_file))
						.map(|(pub_type, items)| format!("{} use {}::{} {} {};", pub_type.to_str(), mod_name, '{', items.into_iter().map(|export| export.identifier.clone()).collect::<Vec<String>>().join(", "), '}'))
						.collect::<Vec<String>>()
						.join("\n")
				)
				.collect::<Vec<String>>()
				.join("\n"),
		);
		if new_contents != file_contents {
			self.file.write(new_contents)?;
		}

		Ok(())
	}
}