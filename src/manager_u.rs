#[test]
fn manage_self() {
	use crate::CrateManager;
	
	CrateManager::new().generate_exports().run().unwrap();
}