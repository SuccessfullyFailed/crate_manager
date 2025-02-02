use crate::{ files::CrateDir, lib_export_generator };
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
			println!("{}", crate_dir.path);

			// Crate dir modifications.

			// Mod file modifications.
			for mod_file in &mut crate_dir.mod_files {
				println!("\t{}", mod_file.file.path);
				if self.generate_exports {
					println!("\t\t + auto_exports");
					lib_export_generator::generate_exports_for_mod(mod_file)?;
				}

				// Code file modifications.
			}
		}

		// Store modifications to file.
		for crate_dir in &mut crate_dirs {
			crate_dir.store_changes()?;
		}

		// Return modified files list.
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