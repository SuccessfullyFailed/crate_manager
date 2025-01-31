#[test]
fn test_create_own_exports() {
	crate::generate_exports_for_crates_in_working_dir().unwrap();
}