use super::super::accessibility_text::cf_type_to_string;
use super::{TextHandler, TextResult, TextTarget};
use accessibility::attribute::AXUIElementAttributes;
use core_foundation::{base::TCFType, string::CFString};
use harper_core::{Span, linting::Suggestion};

/// Reads and replaces AXValue for editors without a registered handler.
pub struct DefaultTextHandler;
impl TextHandler for DefaultTextHandler {
    fn read_text(&self, target: &TextTarget) -> TextResult<String> {
        cf_type_to_string(&target.element.value()?).ok_or_else(|| "editor value is not text".into())
    }
    fn write_text(
        &self,
        target: &TextTarget,
        snapshot: &str,
        span: Span<char>,
        suggestion: &Suggestion,
    ) -> TextResult<()> {
        let mut chars = snapshot.chars().collect();
        suggestion.apply(span, &mut chars);
        let updated = chars.into_iter().collect::<String>();
        target
            .element
            .set_value(CFString::new(&updated).as_CFType())?;
        Ok(())
    }
}
