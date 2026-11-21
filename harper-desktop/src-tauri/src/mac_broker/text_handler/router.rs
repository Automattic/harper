use super::*;
use std::collections::HashMap;

/// Routes by the captured editor's bundle ID. Only a missing registration selects
/// the fallback; a handler error must never trigger a second edit.
pub struct RoutingTextHandler {
    handlers: HashMap<&'static str, Box<dyn TextHandler>>,
    fallback: Box<dyn TextHandler>,
}
impl Default for RoutingTextHandler {
    fn default() -> Self {
        let mut handlers: HashMap<_, Box<dyn TextHandler>> = HashMap::new();
        handlers.insert("com.hnc.Discord", Box::new(DiscordTextHandler));
        Self {
            handlers,
            fallback: Box::new(DefaultTextHandler),
        }
    }
}
impl RoutingTextHandler {
    fn handler(&self, id: &str) -> &dyn TextHandler {
        self.handlers.get(id).unwrap_or(&self.fallback).as_ref()
    }
}
impl TextHandler for RoutingTextHandler {
    fn read_text(&self, target: &TextTarget) -> TextResult<String> {
        self.handler(&target.bundle_id).read_text(target)
    }
    fn write_text(
        &self,
        target: &TextTarget,
        snapshot: &str,
        span: Span<char>,
        suggestion: &Suggestion,
    ) -> TextResult<()> {
        self.handler(&target.bundle_id)
            .write_text(target, snapshot, span, suggestion)
    }
}
