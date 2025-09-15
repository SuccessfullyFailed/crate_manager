#[derive(PartialEq, Clone)]
pub(crate) enum PubType { Pub, Super, Crate }
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
pub(crate) struct Import {
	pub(crate) pub_type:Option<PubType>,
	pub(crate) struct_type:String,
	pub(crate) identifier:String
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct Export {
	pub(crate) pub_type:Option<PubType>,
	pub(crate) struct_type:String,
	pub(crate) identifier:String
}