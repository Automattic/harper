 }
 }

 /// Regression test for <https://github.com/Automattic/harper/issues/3362>
 #[test]
 fn athough_suggests_although() {
 assert_suggestion_result(
 "I walked athough the park.",
 SpellCheck::new(FstDictionary::curated(), Dialect::American),
 "I walked although the park.",
 );
 }
}
