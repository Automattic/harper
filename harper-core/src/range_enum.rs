use std::ops::{
    Add, Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

/// A range type that can contain any variant of the standard range.
#[derive(Clone, PartialEq)]
pub(crate) enum RangeEnum<Idx> {
    Range(Range<Idx>),
    RangeFrom(RangeFrom<Idx>),
    RangeFull,
    RangeInclusive(RangeInclusive<Idx>),
    RangeTo(RangeTo<Idx>),
    RangeToInclusive(RangeToInclusive<Idx>),
}

impl<Idx: Copy> RangeEnum<Idx> {
    /// Set the start index for the range.
    pub(crate) fn set_start(&mut self, start: Idx) {
        use self::RangeEnum::*;

        *self = match self {
            Range(range) => Range(start..range.end),
            RangeFrom(_) | RangeFull => RangeFrom(start..),
            RangeInclusive(range_inclusive) => RangeInclusive(start..=*range_inclusive.end()),
            RangeTo(range_to) => Range(start..range_to.end),
            RangeToInclusive(range_to_inclusive) => RangeInclusive(start..=range_to_inclusive.end),
        };
    }

    /// Set the end index for the range.
    pub(crate) fn set_end(&mut self, end: Idx) {
        use self::RangeEnum::*;

        *self = match self {
            Range(range) => Range(range.start..end),
            RangeFrom(range_from) => Range(range_from.start..end),
            RangeFull | RangeTo(_) => RangeTo(..end),
            RangeInclusive(range_inclusive) => RangeInclusive(*range_inclusive.start()..=end),
            RangeToInclusive(_) => RangeToInclusive(..=end),
        };
    }

    /// Offset the range by adding the given signed/unsigned offset.
    ///
    /// This is currently only used as a helper function for [`Self::offset_by`].
    fn offset_by_idx(&mut self, offset: Idx)
    where
        Idx: Add<Output = Idx>,
    {
        use self::RangeEnum::*;

        *self = match self {
            Range(range) => Range(range.start + offset..range.end + offset),
            RangeFrom(range_from) => RangeFrom(range_from.start + offset..),
            RangeFull => RangeFull,
            RangeInclusive(range_inclusive) => {
                RangeInclusive(*range_inclusive.start() + offset..=*range_inclusive.end() + offset)
            }
            RangeTo(range_to) => RangeTo(..range_to.end + offset),
            RangeToInclusive(range_to_inclusive) => {
                RangeToInclusive(..=range_to_inclusive.end + offset)
            }
        }
    }

    /// Offset the range by adding the given signed/unsigned offset.
    ///
    /// `Offset` must be infallibly convertible to `Idx`.
    pub(crate) fn offset_by<Offset>(&mut self, offset: Offset)
    where
        Idx: Add<Output = Idx>,
        Idx: From<Offset>,
    {
        self.offset_by_idx(Idx::from(offset));
    }

    /// Clamp a provided range-like type into the span of this range.
    ///
    /// The range-like type must be convertible to and from [`Range<Idx>`].
    pub(crate) fn clamp<Other>(&self, other: Other) -> Other
    where
        Other: Into<Range<Idx>> + From<Range<Idx>>,
        Idx: Ord,
    {
        let mut other = other.into();

        // Since this only supports (exclusive) range types, we don't actually pay attention to whether the
        // bounds are inclusive or exclusive here. The output will be exclusive either way.
        if let Some(start_bound) = self.start_bound().value() {
            other.start = Idx::max(*start_bound, other.start);
        }

        if let Some(end_bound) = self.end_bound().value() {
            other.end = Idx::min(*end_bound, other.end);
        }

        Other::from(other)
    }
}

trait BoundExt<Idx> {
    /// Get the contained bound, whether inclusive or exclusive.
    ///
    /// If the bound is [`Bound::Unbounded`], returns `None`.
    fn value(&self) -> Option<Idx>;
}

impl<Idx: Copy> BoundExt<Idx> for Bound<Idx> {
    fn value(&self) -> Option<Idx> {
        match self {
            Self::Included(val) | Self::Excluded(val) => Some(*val),
            Self::Unbounded => None,
        }
    }
}

// Might not be the best way to go about doing things, but I'm not sure how else to do it.
// Casting to a `&dyn RangeBounds<Idx>` doesn't seem to be possible since `RangeBounds` isn't
// dyn-compatible.
macro_rules! delegate_to_inner {
    ($fn:ident, $return:ty) => {
        fn $fn(&self) -> $return {
            match self {
                Self::Range(range) => range.$fn(),
                Self::RangeFrom(range_from) => range_from.$fn(),
                Self::RangeFull => RangeFull.$fn(),
                Self::RangeInclusive(range_inclusive) => range_inclusive.$fn(),
                Self::RangeTo(range_to) => range_to.$fn(),
                Self::RangeToInclusive(range_to_inclusive) => range_to_inclusive.$fn(),
            }
        }
    };
}

impl<Idx> RangeBounds<Idx> for RangeEnum<Idx> {
    delegate_to_inner!(start_bound, Bound<&Idx>);
    delegate_to_inner!(end_bound, Bound<&Idx>);
}

// Convert from standard ranges.
impl<Idx> From<Range<Idx>> for RangeEnum<Idx> {
    fn from(value: Range<Idx>) -> Self {
        Self::Range(value)
    }
}
impl<Idx> From<RangeFrom<Idx>> for RangeEnum<Idx> {
    fn from(value: RangeFrom<Idx>) -> Self {
        Self::RangeFrom(value)
    }
}
impl<Idx> From<RangeFull> for RangeEnum<Idx> {
    fn from(_: RangeFull) -> Self {
        Self::RangeFull
    }
}
impl<Idx> From<RangeInclusive<Idx>> for RangeEnum<Idx> {
    fn from(value: RangeInclusive<Idx>) -> Self {
        Self::RangeInclusive(value)
    }
}
impl<Idx> From<RangeTo<Idx>> for RangeEnum<Idx> {
    fn from(value: RangeTo<Idx>) -> Self {
        Self::RangeTo(value)
    }
}
impl<Idx> From<RangeToInclusive<Idx>> for RangeEnum<Idx> {
    fn from(value: RangeToInclusive<Idx>) -> Self {
        Self::RangeToInclusive(value)
    }
}
