use ariadne::{Color, Label};
use clap::ValueEnum;
use harper_core::{Document, Span, Token, TokenKind};
use strum::IntoEnumIterator;

/// Represents an annotation.
pub(super) struct Annotation {
    /// The range the annotation covers in the source. For instance, this might be a single word.
    span: Span<char>,
    /// The message displayed by the annotation.
    annotation_text: String,
    /// The color of the annotation.
    color: Color,
}
impl Annotation {
    /// Converts the annotation into an [`ariadne::Label`].
    #[must_use]
    pub(super) fn into_label(
        self,
        input_identifier: &str,
    ) -> Label<(&str, std::ops::Range<usize>)> {
        Label::new((input_identifier, self.span.into()))
            .with_message(self.annotation_text)
            .with_color(self.color)
    }

    /// Gets an iterator of annotation `Label` from the given document.
    ///
    /// This is similar to [`self::iter_from_document()`], but this additionally converts
    /// the [`Annotation`] into [`ariadne::Label`] for convenience.
    pub(super) fn iter_labels_from_document<'inpt_id>(
        annotation_type: AnnotationType,
        document: &Document,
        input_identifier: &'inpt_id str,
    ) -> impl Iterator<Item = Label<(&'inpt_id str, std::ops::Range<usize>)>> {
        Self::iter_from_document(annotation_type, document)
            .map(|annotation| annotation.into_label(input_identifier))
    }

    /// Constructs an [`Annotation`] from the given `token` based on the given `annotation_type`.
    ///
    /// This will return `None` when the `token` should not be annotated according to
    /// `annotation_type`.
    #[must_use]
    fn from_token(annotation_type: AnnotationType, token: &Token) -> Option<Self> {
        let span = token.span;
        match annotation_type {
            AnnotationType::Upos => {
                if let TokenKind::Word(Some(metadata)) = &token.kind {
                    // Only annotate words (with dict word metadata) for `AnnotationType::Upos`.
                    let pos_tag = metadata.pos_tag;
                    Some(Self {
                        span,
                        annotation_text: pos_tag.map_or("NONE".to_owned(), |upos| upos.to_string()),
                        color: pos_tag.map_or(Color::Red, get_color_for_enum_variant),
                    })
                } else {
                    // Not a word, or a word with no metadata.
                    None
                }
            }
        }
    }

    /// Gets an iterator of [`Annotation`] for a given document. The annotations will be based on
    /// `annotation_type`.
    fn iter_from_document(
        annotation_type: AnnotationType,
        document: &Document,
    ) -> impl Iterator<Item = Self> {
        document
            .tokens()
            .filter_map(move |token| Self::from_token(annotation_type, token))
    }
}

/// Represents how the tokens should be annotated.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(super) enum AnnotationType {
    /// UPOS (part of speech)
    Upos,
}

/// Gets a random `Color` for an enum variant.
///
/// A given enum variant's color is consistent, meaning it will not change throughout multiple
/// calls of this function or multiple runs of the application.
#[must_use]
fn get_color_for_enum_variant<T: IntoEnumIterator + PartialEq>(variant_to_color: T) -> Color {
    // Using a lower than default `min_brightness` to hopefully create more distinguishable colors.
    let mut color_gen = ariadne::ColorGenerator::from_state([31715, 3528, 21854], 0.2);
    T::iter()
        // Note: `ColorGenerator` does not implement `Iterator`, so we can't just zip it.
        .map(|enum_variant| (enum_variant, color_gen.next()))
        .find(|(enum_variant, _)| *enum_variant == variant_to_color)
        .unwrap()
        .1
}
