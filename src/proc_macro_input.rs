use crate::input::{Input, InputMatch};
use crate::proc_macro::Pat;
use crate::proc_macro::{FlatTokenPat, Span, TokenStream};
use flat_token::{flatten, FlatToken};
use std::ops::Deref;
use std::ops::Range;

impl Input for TokenStream {
    type Container = Vec<FlatToken>;
    type Slice = [FlatToken];
    type SourceInfo = Range<Span>;
    type SourceInfoPoint = Span;
    fn to_container(self) -> Self::Container {
        let mut out = vec![];
        flatten(self, &mut out);
        out
    }
    fn slice<'b>(input: &'b Self::Container, range: Range<usize>) -> &'b Self::Slice {
        &input[range]
    }

    fn source_info(input: &Self::Container, range: Range<usize>) -> Self::SourceInfo {
        // FIXME(eddyb) should be joining up spans, but the API
        // for that is still "semver-exempt" in `proc-macro2`.
        Self::source_info_point(input, range.start)..Self::source_info_point(input, range.end)
    }

    fn source_info_point(input: &Self::Container, index: usize) -> Self::SourceInfoPoint {
        // Try to get as much information as possible.
        let (before, after) = input.split_at(index);
        if let Some(first) = after.first() {
            first.span()
        } else if let Some(last) = before.last() {
            // Not correct but we're at the end of the input anyway.
            last.span()
        } else {
            // HACK(eddyb) last resort, make a span up
            // (a better option should exist)
            Span::call_site()
        }
    }

    fn len(input: &Self::Container) -> usize {
        input.len()
    }
}

// FIXME(eddyb) can't use `Pats: AsRef<[FlatTokenPat<S>]` as it doesn't constrain `S`.
impl<S: AsRef<str>, Pats: Deref<Target = [FlatTokenPat<S>]>> InputMatch<Pat<Pats>>
    for [FlatToken]
{
    fn match_left(&self, pat: &Pat<Pats>) -> Option<usize> {
        self.match_left(&*pat.0)
    }
    fn match_right(&self, pat: &Pat<Pats>) -> Option<usize> {
        self.match_right(&*pat.0)
    }
}

impl<S: AsRef<str>> InputMatch<[FlatTokenPat<S>]> for [FlatToken] {
    fn match_left(&self, pat: &[FlatTokenPat<S>]) -> Option<usize> {
        if self
            .iter()
            .zip(pat)
            .take_while(|(t, p)| p.matches(t))
            .count()
            == pat.len()
        {
            Some(pat.len())
        } else {
            None
        }
    }
    fn match_right(&self, pat: &[FlatTokenPat<S>]) -> Option<usize> {
        if self
            .iter()
            .zip(pat)
            .rev()
            .take_while(|(t, p)| p.matches(t))
            .count()
            == pat.len()
        {
            Some(pat.len())
        } else {
            None
        }
    }
}
