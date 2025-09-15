use glyph_kit::{ TextMatchResult, TextMatcher, TextMatcherSet, TextMatcherSource };
use crate::imports_and_exports::AUTO_EXPORTS_TAG;
use cachew::cache;



pub const MODULE_IMPORT_TAG:&str = "import";
pub const PARSER_EXPORT_TAG:&str = "export";
pub const PARSER_PUB_TYPE_TAG:&str = "pub_type";
pub const PARSER_TYPE_TAG:&str = "export_type";
pub const PARSER_IDENTIFIER_TAG:&str = "identifier";
pub const PARSER_AUTO_EXPORTS_TRIGGER_TAG:&str = "auto_exports_trigger";
pub fn imports_exports_parser() -> &'static TextMatcherSet {
	cache!(
		TextMatcherSet,
		{

			// Small matchers.
			let max_optional_whitespace:TextMatcher = TextMatcher::optional_repeat_max(TextMatcher::whitespace());
			let pub_type_matcher:TextMatcher = TextMatcher::named(PARSER_PUB_TYPE_TAG, TextMatcher::new("pub") + TextMatcher::optional(TextMatcher::new("(") + max_optional_whitespace.clone() + (TextMatcher::new("crate") | "super") + max_optional_whitespace.clone() + ")"));
			let optional_pub_type_matcher:TextMatcher = TextMatcher::optional(pub_type_matcher.clone() + TextMatcher::repeat_max(TextMatcher::whitespace()));
			let identifier_matcher:TextMatcher = TextMatcher::named(PARSER_IDENTIFIER_TAG, TextMatcher::repeat_max(TextMatcher::new(|text:&str| {
				if !text.is_empty() {
					let first_char:u8 = text[..1].chars().next().unwrap() as u8;
					if (first_char >= 'A' as u8 && first_char < 'z' as u8) || first_char == '_' as u8 {
						return Some(TextMatchResult::new(1, text));
					}
				}
				None
			})));

			// Full matchers.
			let string_matcher:TextMatcher = TextMatcher::new("\"") + TextMatcher::optional_repeat_max(TextMatcher::new("\\\"") | !TextMatcher::new("\"")) + "\"";
			let comment_matcher:TextMatcher = (TextMatcher::new("//") + TextMatcher::optional_repeat_max(!TextMatcher::new("\n")) + "\n") | (TextMatcher::new("/*") + TextMatcher::optional_repeat_max(!TextMatcher::new("*/")) + "*/");
			
			// Matcher set.
			TextMatcherSet::new().with_matchers(vec![

				/* IMPORTERS */
				(
					MODULE_IMPORT_TAG,
					optional_pub_type_matcher.clone() +
					TextMatcher::named(PARSER_TYPE_TAG, TextMatcher::new("mod")) +
					TextMatcher::repeat_max(TextMatcher::whitespace()) +
					TextMatcher::named(PARSER_IDENTIFIER_TAG, TextMatcher::repeat_max(!(TextMatcher::new(";"))))
				),
				(
					MODULE_IMPORT_TAG,
					optional_pub_type_matcher.clone() +
					TextMatcher::named(PARSER_TYPE_TAG, TextMatcher::new("use")) +
					TextMatcher::repeat_max(TextMatcher::whitespace()) +
					TextMatcher::named(PARSER_IDENTIFIER_TAG, TextMatcher::repeat_max(!(TextMatcher::new(";"))))
				),



				/* EXPORTERS */
				(
					PARSER_EXPORT_TAG,
					pub_type_matcher.clone() +
					TextMatcher::repeat_max(TextMatcher::whitespace()) +
					TextMatcher::named(PARSER_TYPE_TAG, TextMatcher::new("struct") | "trait" | "fn" | "const" | "static") +
					TextMatcher::repeat_max(TextMatcher::whitespace()) +
					identifier_matcher.clone()
				),



				/* AUTO EXPORTS TAG */
				(
					PARSER_AUTO_EXPORTS_TRIGGER_TAG,
					TextMatcher::new("//") + max_optional_whitespace + AUTO_EXPORTS_TAG
				),



				/* MISCELLANEOUS */
				("string", string_matcher.clone()),
				("comment", comment_matcher.clone()),
				(
					"scope",
					TextMatcher::new(
						move |text:&str| {
							if !text.is_empty() && &text[..1] == "{" {
								let mut cursor:usize = 1;
								let mut depth:usize = 1;
								let cursor_max:usize = text.len();
								while depth > 0 && cursor < cursor_max {
									if let Some(result) = string_matcher.clone().match_text(&text[cursor..]) {
										cursor += result.length;
									} else if let Some(result) = comment_matcher.clone().match_text(&text[cursor..]) {
										cursor += result.length;
									} else {
										match &text[cursor..cursor + 1] {
											"{" => depth += 1,
											"}" => depth -= 1,
											_ => {}
										}
										cursor += 1;
									}
								}
								if depth == 0 {
									return Some(TextMatchResult::new(cursor, text))
								}
							}
							None
						}
					)
				)
			])
		}
	)
}