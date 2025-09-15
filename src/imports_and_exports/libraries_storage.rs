use file_ref::FileRef;



const GIT_URL_TAG:&str = "github.com";



pub struct LibrariesStorage {
	pub(crate) libraries:Vec<Library>
}
impl LibrariesStorage {

	/// Create a new library storage.
	pub fn new(libraries:Vec<Library>) -> LibrariesStorage {
		let mut storage:LibrariesStorage = LibrariesStorage { libraries };
		storage.combine_libraries();
		storage
	}

	/// Create a library storage from the contents of a file.
	pub fn from_file(file:&str) -> LibrariesStorage {
		let mut libraries:Vec<Library> = Vec::new();
		let contents:String = FileRef::new(file).read().unwrap_or_default();
		for line in contents.split('\n').map(|line| line.trim()) {
			if !line.is_empty() {
				let line_chars:Vec<char> = line.chars().collect();
				let whitespace_start:usize = line_chars.iter().position(|char| char.is_whitespace()).unwrap_or(line_chars.len());
				let whitespace_length:usize = line_chars[whitespace_start..].iter().position(|char| !char.is_whitespace()).unwrap_or(0);
				libraries.push(Library::new(&line[..whitespace_start], &line[whitespace_start + whitespace_length..]));
			}
		}
		LibrariesStorage::new(libraries)
	}

	/// Combine libraries when possible.
	fn combine_libraries(&mut self) {
		let mut source_index:usize = 0;
		while source_index < self.libraries.len() {
			while let Some(target_offset) = self.libraries[source_index + 1..].iter().position(|library| self.libraries[source_index].name == library.name) {
				let target:Library = self.libraries.remove(source_index + 1 + target_offset);
				self.libraries[source_index].combine_with(target);
			}
			source_index += 1;
		}
	}

	/// Find a library by name.
	pub fn find(&self, library_name:&str) -> Option<&Library> {
		self.libraries.iter().find(|library| library.name == library_name)
	}
}



#[derive(Clone)]
pub struct Library {
	pub(crate) name:String,
	pub(crate) versions:Vec<String>,
	pub(crate) git_url:Option<String>,
	pub(crate) local_path:Option<String>
}
impl Library {

	/* CONSTRUCTOR METHODS */

	/// Create a new library. Tries to determine type by source.
	pub fn new(name:&str, source:&str) -> Library {

		// Recurse with trailing version number.
		if let Some(version) = source.split(" ").last().filter(|word| word.split(".").filter(|s| s.chars().all(|char| char.is_numeric())).count() == 3) {
			Library::new(name, source[..source.len() - version.len()].trim()).with_version_number(version)
		}
		
		// Git url.
		else if source.contains(GIT_URL_TAG) {
			Library::git(name, source)
		}
		
		// Local path.
		else if FileRef::new(source).exists() {
			Library::local(name, source)
		}
		
		// Only from name.
		else {
			Library {
				name: name.to_string(),
				versions: Vec::new(),
				git_url: None,
				local_path: None
			}
		}
	}
	
	/// Create a new git-imported library.
	pub fn git(name:&str, git_url:&str) -> Library {
		Library {
			name: name.to_string(),
			versions: Vec::new(),
			git_url: Some(git_url.to_string()),
			local_path: None
		}
	}

	/// Create a new library from a local path.
	pub fn local(name:&str, path:&str) -> Library {
		Library {
			name: name.to_string(),
			versions: Vec::new(),
			git_url: None,
			local_path: Some(path.to_string())
		}
	}

	/// Return self with a specific version number.
	pub fn with_version_number(mut self, version:&str) -> Self {
		self.versions.push(version.to_string());
		self
	}


	/* USAGE METHODS */

	/// Combine this library with another.
	pub fn combine_with(&mut self, other:Library) {
		self.versions.extend(other.versions);
		if let Some(git_url) = other.git_url {
			self.git_url = Some(git_url);
		}
		if let Some(local_path) = other.local_path {
			self.local_path = Some(local_path);
		}
	}

	/// Create a string importing the library, able to be pasted in a Cargo.toml file.
	pub fn as_import_string(&self) -> String {
		const QUOTED:fn(&str) -> String = |source| format!("\"{}\"", source.trim().trim_matches('"'));

		let mut properties:Vec<(&'static str, String)> = Vec::new();
		if let Some(version) = self.versions.last() {
			properties.push(("version", QUOTED(version)));
		}
		if let Some(local_path) = &self.local_path {
			properties.push(("path", QUOTED(local_path)));
		}
		if let Some(git_url) = &self.git_url {
			properties.push(("git", QUOTED(git_url)));
		}
		format!(
			"{}={} {} {}",
			self.name,
			'{',
			properties.iter().map(|(name, value)| format!("{name}={value}")).collect::<Vec<String>>().join(", "),
			'}'
		)
	}
}