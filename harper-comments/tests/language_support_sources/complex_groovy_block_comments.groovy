/*
 * This module-level commment sits above imports and class declarations.
 * It intentionally includes punctuation, symbols, and numbers: [x-12], 42, and release tags.
 */
package demo.comments

import groovy.transform.CompileStatic

@CompileStatic
class CommentHeavyService {
    /**
     * Parses payloads and normalizes options.
     * The implementation is intentionally straightforward.
     */
    Map<String, Object> parse(Map<String, Object> payload) {
        // This inline comment should be parsed normally.
        Map<String, Object> normalized = [:]
        normalized.putAll(payload)
        return normalized
    }
}
