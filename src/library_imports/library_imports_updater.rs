use crate::{ LibrariesStorage, Library };
use std::error::Error;
use file_ref::FileRef;



/// Try to find all missing imports and insert them in the toml file.
pub fn generate_toml_imports(toml_file:&str, found_lib_names:&[&str], available_libraries:&LibrariesStorage) -> Result<(), Box<dyn Error>> {
	const INVALID_LIB_NAMES:&[&str] = &["std", "crate", "super"];
	
	// Read toml file.
	let toml_file:FileRef = FileRef::new(toml_file);
	let toml_contents:String = toml_file.read()?;
	let mut toml_category_index:usize = 0;
	let mut toml:Vec<(String, Vec<(String, String)>)> = vec![(String::new(), Vec::new())];
	for line in toml_contents.split('\n').map(|line| line.trim()) {

		// Empty line.
		if line.is_empty() {
			continue;
		}
		
		// Category change.
		else if line.starts_with('[') && line.ends_with(']') {
			let toml_category:&str = line[1..line.len() - 1].trim();
			toml_category_index = match toml.iter().position(|(name, _)| name == toml_category) {
				Some(index) => index,
				None => {
					toml.push((toml_category.to_string(), Vec::new()));
					toml.len() - 1
				}
			};
		}
		
		// Value.
		else if line.contains('=') {
			let name:&str = line.split('=').next().unwrap();
			let value:&str = line[name.len() + 1..].trim();
			toml[toml_category_index].1.push((name.trim().to_string(), value.to_string()));
		}
	}

	// Find missing imports.
	if let Some((_, package_data)) = toml.iter().find(|(name, _)| name == "package") {
		if let Some((_, own_crate_name)) = package_data.iter().find(|(name, _)| name == "name") {
			let existing_imports:Vec<String> = match toml.iter().find(|(name, _)| name == "dependencies") {
				Some((_, dependencies_ini)) => dependencies_ini.iter().map(|(name, _)| name.to_string()).collect(),
				None => Vec::new()
			};
			let missing_imports:Vec<&&str> = found_lib_names.iter().filter(|name| *name != &own_crate_name && !INVALID_LIB_NAMES.contains(name) && !existing_imports.iter().any(|existing| existing == *name)).collect();

			// Try to add missing imports.
			let missing_libraries:Vec<&Library> = missing_imports.iter().map(|name| available_libraries.find(name)).flatten().collect();
			if !missing_libraries.is_empty() {
				let dependencies_ini:&mut Vec<(String, String)> = match toml.iter_mut().find(|(name, _)| name == "dependencies") {
					Some((_, dependencies_ini)) => dependencies_ini,
					None => {
						toml.push(("dependencies".to_string(), Vec::new()));
						&mut toml.last_mut().unwrap().1
					}
				};
				dependencies_ini.extend(missing_libraries.iter().map(|library| (library.name.clone(), library.as_import_value())));
				

				// Write new file.
				let new_file_contents:String = toml.iter().filter(|(_, values)| !values.is_empty()).map(|(category_name, category_data)| 
					format!("[{category_name}]\n{}",
						category_data.iter().map(|(name, value)| format!("{name}={value}")).collect::<Vec<String>>().join("\n")
					)
				).collect::<Vec<String>>().join("\n\n");
				toml_file.write(new_file_contents).unwrap();
			}
		}
	}

	// Return success.
	Ok(())
}