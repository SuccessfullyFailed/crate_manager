use crate::{ files::{CodeFile, CrateDir, ModFile}, lib_export_generator };
use std::error::Error;
use file_ref::FileRef;



pub struct CrateManager {
	generate_exports:bool,
	ran:bool
}
impl CrateManager {

	/* CONSTRUCTOR METHODS */

	/// Get the default settings.
	pub fn new() -> CrateManager {
		CrateManager {
			generate_exports: false,
			ran: false
		}
	}

	/// Return self with an exports generator.
	pub fn generate_exports(mut self) -> Self {
		self.generate_exports = true;
		self
	}



	/* USAGE METHODS */

	/// Run all processes according to settings. Returns the amount of files modified.
	pub fn run(mut self) -> Result<(), Box<dyn Error>> {
		self.ran = true;

		// Run modifications.
		let mut crate_dirs:Vec<CrateDir> = CrateDir::list_in(FileRef::working_dir());
		println!("[CrateManager]");
		for crate_dir in &mut crate_dirs {
			self.process_crate_dir(crate_dir)?;
		}

		// Store modifications to file.
		for crate_dir in &mut crate_dirs {
			crate_dir.store_changes()?;
		}

		// Return modified files list.
		Ok(())
	}

	/// Process a CrateDir.
	fn process_crate_dir(&mut self, crate_dir:&mut CrateDir) -> Result<(), Box<dyn Error>> {
		println!("{}", crate_dir.path);
		for mod_file in &mut crate_dir.mod_files {
			self.process_mod_file(mod_file)?;
		}
		Ok(())
	}

	/// Process a ModFile.
	fn process_mod_file(&mut self, mod_file:&mut ModFile) -> Result<(), Box<dyn Error>> {
		for sub_mod_file in &mut mod_file.sub_mod_files {
			self.process_mod_file(sub_mod_file)?;
		}
		for code_file in &mut mod_file.source_files {
			self.process_code_file(code_file)?;
		}
		println!("\t{}", mod_file.file.path);
		if self.generate_exports {
			println!("\t\t + auto_exports");
			lib_export_generator::generate_exports_for_mod(mod_file)?;
		}

		Ok(())
	}

	/// Process a code file.
	fn process_code_file(&mut self, _code_file:&mut CodeFile) -> Result<(), Box<dyn Error>> {
		Ok(())
	}
}
impl Drop for CrateManager {
	fn drop(&mut self) {
		if !self.ran {
			eprintln!("CrateManager object was dropped without running. Please call the 'run' method to execute all set modifications.");
		}
	}
}