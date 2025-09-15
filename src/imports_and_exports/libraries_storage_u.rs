#[cfg(test)]
mod tests {
	use crate::{LibrariesStorage, Library};


	fn test_libs() -> Vec<Library> {
		vec![
			Library::local("local_lib", "C:/lib"),
			Library::git("git_lib", "https://github.com/git_lib"),
			Library::new("local_lib_auto", "Cargo.toml"),
			Library::new("git_lib_auto", "https://github.com/git_lib_auto"),
			Library::new("local_lib_auto_v", "Cargo.toml 1.23.456"),
			Library::new("git_lib_auto_v", "https://github.com/git_lib_auto 1.23.456"),
			Library::new("nonexistent_lib", ""),
			Library::new("nonexistent_lib_v", "1.23.456"),
			Library::new("combining_lib", "https://github.com/git_lib 1.2.3"),
			Library::new("combining_lib", "Cargo.toml 4.5.6"),
		]
	}
	
	
	#[test]
	fn test_library_creation() {
		let libs:Vec<Library> = test_libs();

		assert_eq!(libs[0].name, "local_lib");
		assert_eq!(libs[0].local_path, Some("C:/lib".to_string()));
		assert_eq!(libs[0].git_url, None);
		assert_eq!(libs[0].versions, Vec::<String>::new());

		assert_eq!(libs[1].name, "git_lib");
		assert_eq!(libs[1].local_path, None);
		assert_eq!(libs[1].git_url, Some("https://github.com/git_lib".to_string()));
		assert_eq!(libs[1].versions, Vec::<String>::new());

		assert_eq!(libs[2].name, "local_lib_auto");
		assert_eq!(libs[2].local_path, Some("Cargo.toml".to_string()));
		assert_eq!(libs[2].git_url, None);
		assert_eq!(libs[2].versions, Vec::<String>::new());

		assert_eq!(libs[3].name, "git_lib_auto");
		assert_eq!(libs[3].local_path, None);
		assert_eq!(libs[3].git_url, Some("https://github.com/git_lib_auto".to_string()));
		assert_eq!(libs[3].versions, Vec::<String>::new());

		assert_eq!(libs[4].name, "local_lib_auto_v");
		assert_eq!(libs[4].local_path, Some("Cargo.toml".to_string()));
		assert_eq!(libs[4].git_url, None);
		assert_eq!(libs[4].versions, vec!["1.23.456".to_string()]);

		assert_eq!(libs[5].name, "git_lib_auto_v");
		assert_eq!(libs[5].local_path, None);
		assert_eq!(libs[5].git_url, Some("https://github.com/git_lib_auto".to_string()));
		assert_eq!(libs[5].versions, vec!["1.23.456".to_string()]);

		assert_eq!(libs[6].name, "nonexistent_lib");
		assert_eq!(libs[6].local_path, None);
		assert_eq!(libs[6].git_url, None);
		assert_eq!(libs[6].versions, Vec::<String>::new());

		assert_eq!(libs[7].name, "nonexistent_lib_v");
		assert_eq!(libs[7].local_path, None);
		assert_eq!(libs[7].git_url, None);
		assert_eq!(libs[7].versions, vec!["1.23.456".to_string()]);
	}

	#[test]
	fn test_library_storage() {
		let storage:LibrariesStorage = LibrariesStorage::new(test_libs());

		// Assert two libraries have been combined.
		assert_eq!(storage.libraries.len(), 9);
		assert_eq!(storage.libraries[8].name, "combining_lib");
		assert_eq!(storage.libraries[8].local_path, Some("Cargo.toml".to_string()));
		assert_eq!(storage.libraries[8].git_url, Some("https://github.com/git_lib".to_string()));
		assert_eq!(storage.libraries[8].versions, vec!["1.2.3".to_string(), "4.5.6".to_string()]);

		// Find library by name.
		assert_eq!(storage.find("git_lib_auto").unwrap().git_url, Some("https://github.com/git_lib_auto".to_string()));
	}
}