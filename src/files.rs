use std::error::Error;
use cachew::cache;
use file_ref::FileRef;
use omni_parser::{NestedCodeParser, NestedSegment};



const CRATE_ROOT_DETECTOR_FILE_NAME:&str = "Cargo.toml";
const CRATE_SOURCE_DIR:&str = "src";
const MOD_FILE_NAME:&str = "mod.rs";
const LIB_FILE_NAME:&str = "lib.rs";
const DIR_RECURSE_EXCEPTIONS:&[&str] = &["target", ".git"];



pub(crate) struct CrateDir {
	pub path:FileRef,
	pub mod_files:Vec<ModFile>
}
impl CrateDir {

	/// List all crate dirs in the given dir.
	pub fn list_in(dir:FileRef) -> Vec<CrateDir> {
		dir
			.absolute()
			.scanner()
			.include_self()
			.include_dirs()
			.recurse_filter(|sub_dir| !DIR_RECURSE_EXCEPTIONS.contains(&sub_dir.name()))
			.filter(|crate_dir| (crate_dir.clone() + "/" + CRATE_ROOT_DETECTOR_FILE_NAME).exists() && (crate_dir.clone() + "/" + CRATE_SOURCE_DIR + "/" + LIB_FILE_NAME).exists())
			.map(|dir| CrateDir::new(dir))
			.collect()
	}

	/// Create a new CrateDir.
	pub fn new(dir:FileRef) -> CrateDir {

		// Find mod-files in dir.
		let mut mod_files:Vec<FileRef> = vec![dir.clone() + "/" + CRATE_SOURCE_DIR + "/" + LIB_FILE_NAME];
		let mut undiscovered_dirs:Vec<FileRef> = dir.list_dirs();
		while !undiscovered_dirs.is_empty() {
			let dir:FileRef = undiscovered_dirs.remove(0);
			let mod_file:FileRef = dir.clone() + "/" + MOD_FILE_NAME;
			if mod_file.exists() {
				mod_files.push(mod_file);
				undiscovered_dirs.extend_from_slice(&dir.list_dirs());
			}
		}
		mod_files.sort_by(|a, b| b.len().cmp(&a.len()));

		// Return struct.
		CrateDir {
			path: dir.clone(),
			mod_files: mod_files.iter().map(|file| ModFile::new(file.clone())).flatten().collect()
		}
	}

	/// Save changes to the structure.
	pub fn store_changes(&mut self) -> Result<(), Box<dyn Error>> {
		for mod_file in &mut self.mod_files {
			mod_file.store_changes()?;
		}
		Ok(())
	}
}

pub(crate) struct ModFile {
	pub file:CodeFile,
	pub source_files:Vec<CodeFile>,
	pub sub_mod_files:Vec<ModFile>
}
impl ModFile {

	/// Create a new ModFile.
	fn new(file:FileRef) -> Result<ModFile, Box<dyn Error>> {
		let mod_dir:FileRef = file.parent_dir()?;
		Ok(ModFile {
			file: CodeFile::new(file.clone()),
			source_files: mod_dir.scanner().include_files().filter(|file| file.name() != LIB_FILE_NAME && file.name() != MOD_FILE_NAME).map(|file| CodeFile::new(file)).collect(),
			sub_mod_files: mod_dir.scanner().include_dirs().map(|dir| (dir.clone(), dir.clone() + "/" + MOD_FILE_NAME)).filter(|(_, mod_file)| mod_file.exists()).map(|(_, file)| ModFile::new(file)).flatten().collect()
		})
	}

	/// Save changes to the structure.
	pub fn store_changes(&mut self) -> Result<(), Box<dyn Error>> {
		self.file.store_changes()?;
		for mod_file in &mut self.source_files {
			mod_file.store_changes()?;
		}
		for mod_file in &mut self.sub_mod_files {
			mod_file.store_changes()?;
		}
		Ok(())
	}
}

pub(crate) struct CodeFile {
	pub path:FileRef,
	read_contents:bool,
	contents:String,
	parsed_contents:NestedSegment,
	gotten_mutable:bool,
	stored_changes:bool
}
impl CodeFile {

	/// Create a new struct by path.
	fn new(path:FileRef) -> CodeFile {
		CodeFile {
			path,
			read_contents: false,
			contents: String::new(),
			parsed_contents: NestedSegment::new_code(omni_parser::ROOT_NAME, "", Vec::new(), ""),
			gotten_mutable: false,
			stored_changes: false
		}
	}

	/// Read file contents if not read before.
	fn read_contents(&mut self) -> Result<(), Box<dyn Error>> {
		if !self.read_contents {
			self.contents = self.path.read()?;
			self.parsed_contents = Self::contents_parser().parse(&self.contents);
			self.read_contents = true;
		}
		Ok(())
	}

	/// Get the contents of the file.
	pub fn contents(&mut self) -> Result<&NestedSegment, Box<dyn Error>> {
		self.read_contents()?;
		Ok(self.contents_mut()? as &NestedSegment)
	}

	/// Get the mutable contents of the file.
	pub fn contents_mut(&mut self) -> Result<&mut NestedSegment, Box<dyn Error>> {
		self.read_contents()?;
		self.gotten_mutable = true;
		Ok(&mut self.parsed_contents)
	}

	/// Get the contents parser.
	fn contents_parser() -> &'static NestedCodeParser {
		cache!(
			NestedCodeParser,
			NestedCodeParser::new(vec![
				&("doc-comment", false, "///", "\n"),
				&("single-line-comment", false, "//", "\n"),
				&("multi-line-comment", false, "/*", "*/"),
				&("quote", false, "\"", Some("\\"), "\"", Some("\\")),
				&("scope", true, "{", "}"),
				&("export", r#"^pub\s(struct|enum|fn|trait|impl|mod|const|static|type|use|crate|macro)\s"#)
			])
		)
	}

	/// Save changes to the structure.
	pub fn store_changes(&mut self) -> Result<(), Box<dyn Error>> {
		if self.gotten_mutable {
			let new_contents:String = self.parsed_contents.to_string();
			if new_contents != self.contents {
				self.path.write(&self.contents.to_string())?;
			}
			self.stored_changes = true;
		}
		Ok(())
	}
}
impl Drop for CodeFile {
	fn drop(&mut self) {
		if self.gotten_mutable && !self.stored_changes {
			eprintln!("Dropped modified file '{}' before storing changes.", self.path);
		}
	}
}