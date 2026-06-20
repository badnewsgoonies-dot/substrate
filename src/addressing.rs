//! addressing — facets resolve to exactly one kind, or refuse, or name the
//! discriminator. Never a ranked guess.
//!
//! This is a Rust port of gc-project's `backpack/addressing_substrate.py`
//! (the "AddressedField") onto substrate's own universe. A *request* is a
//! set of facets; resolving intersects the posting lists of ONLY the
//! requested facets, so address cost depends on the request, not on how
//! many kinds exist — you touch the postings, never the field. The only
//! correct outcomes are:
//!
//!   * ADDRESS — exactly one kind carries every requested facet.
//!   * REFUSE  — no kind does (or the request is empty / names an unknown
//!     facet). A refusal is an answer; it is never a nearest guess.
//!   * AMBIGUOUS — several kinds qualify. The verdict then *names the
//!     discriminator* (`add_one_of`): a facet that splits the survivors.
//!     It still refuses to pick one.
//!
//! Similarity ranking, if it existed, would be advisory only — it never
//! produces an ADDRESS. [`AddressedField::prove_flat`] self-verifies the
//! flatness invariant the way gc-project's `prove_instant()` does.
//!
//! The census here is [`crate::kindgen::UNIVERSE_MANIFEST`]: each kind's
//! declared `archetype_conformance` is its facet set. The *mechanism* is
//! universal — only the census varies between fields.

use std::collections::{BTreeMap, BTreeSet};

use crate::kindgen::UNIVERSE_MANIFEST;

/// Why an address could not be issued. A refusal carries enough to explain
/// itself — it is an answer, not a failure to find one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefuseReason {
    /// The request named no facets. There is nothing to address by.
    NoFacets,
    /// The request named a facet that is not in this field's vocabulary.
    UnknownFacet(String),
    /// Every requested facet is known, but no kind carries all of them.
    NoMatch { matched: Vec<&'static str> },
}

/// The three — and only three — outcomes of resolving a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Exactly one kind carries every requested facet.
    Address {
        tag: &'static str,
        /// The facets that did the addressing (audit trail).
        matched: Vec<&'static str>,
    },
    /// No address could be issued. See [`RefuseReason`].
    Refuse(RefuseReason),
    /// Several kinds qualify. `add_one_of` names the discriminators: adding
    /// any one of them to the request narrows the field. Empty `add_one_of`
    /// means the candidates are facet-identical — genuinely indistinguishable
    /// in this census, so addressing them apart is impossible, not merely
    /// unspecified.
    Ambiguous {
        matched: Vec<&'static str>,
        candidates: Vec<&'static str>,
        add_one_of: Vec<&'static str>,
    },
}

/// A facet field: items indexed by the facets they carry, with the inverted
/// (posting-list) index that makes addressing cost query-local.
#[derive(Debug, Clone, Default)]
pub struct AddressedField {
    /// item -> its full facet set.
    items: BTreeMap<&'static str, BTreeSet<&'static str>>,
    /// facet -> posting list of items carrying it (the inverted index).
    postings: BTreeMap<&'static str, BTreeSet<&'static str>>,
}

impl AddressedField {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the field over substrate's universe: every kind in
    /// `UNIVERSE_MANIFEST`, indexed by its declared facets. Because the
    /// manifest is meta-test-enforced bijective with `ALL_KIND_TAGS`, the
    /// field is exactly the census — no kind silently in or out.
    pub fn from_universe() -> Self {
        let mut field = Self::new();
        for entry in UNIVERSE_MANIFEST {
            field.register(entry.tag_name, entry.facets());
        }
        field
    }

    /// Add one item and its facets, updating both indexes. An item with no
    /// facets is recorded (it is part of the census) but appears in no
    /// posting list, so it can never be addressed — only refused around.
    pub fn register(&mut self, item: &'static str, facets: &[&'static str]) {
        let set: BTreeSet<&'static str> = facets.iter().copied().collect();
        for &f in &set {
            self.postings.entry(f).or_default().insert(item);
        }
        self.items.insert(item, set);
    }

    /// Number of items in the census.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The field's facet vocabulary, sorted.
    pub fn vocabulary(&self) -> Vec<&'static str> {
        self.postings.keys().copied().collect()
    }

    /// Resolve a request into the one verdict it earns.
    pub fn resolve(&self, request: &[&str]) -> Verdict {
        self.resolve_counting(request).0
    }

    /// Resolve, and also report how many posting entries were touched. The
    /// count is the sum of the requested facets' posting-list lengths (plus
    /// the candidates' facet sets when disambiguating). It depends on the
    /// request, never on items outside those postings — that is flatness,
    /// made observable.
    pub fn resolve_counting(&self, request: &[&str]) -> (Verdict, usize) {
        // Normalize the request to the field's own &'static facet names so
        // verdicts borrow from the vocabulary, and reject anything unknown.
        let mut req: BTreeSet<&'static str> = BTreeSet::new();
        for &f in request {
            match self.canonical(f) {
                Some(canon) => {
                    req.insert(canon);
                }
                None => {
                    return (
                        Verdict::Refuse(RefuseReason::UnknownFacet(f.to_string())),
                        0,
                    );
                }
            }
        }
        if req.is_empty() {
            return (Verdict::Refuse(RefuseReason::NoFacets), 0);
        }

        // Intersect the requested facets' postings, smallest first so the
        // candidate set only ever shrinks. We touch exactly these postings.
        let mut lists: Vec<&BTreeSet<&'static str>> =
            req.iter().map(|f| &self.postings[f]).collect();
        lists.sort_by_key(|s| s.len());
        let mut cost: usize = lists.iter().map(|s| s.len()).sum();

        let mut candidates: BTreeSet<&'static str> = lists[0].iter().copied().collect();
        for list in &lists[1..] {
            candidates = candidates.intersection(list).copied().collect();
            if candidates.is_empty() {
                break;
            }
        }

        let matched: Vec<&'static str> = req.iter().copied().collect();
        match candidates.len() {
            0 => (Verdict::Refuse(RefuseReason::NoMatch { matched }), cost),
            1 => {
                let tag = *candidates.iter().next().unwrap();
                (Verdict::Address { tag, matched }, cost)
            }
            _ => {
                // Name the discriminator: (union - inter) - req over the
                // candidates' full facet sets — the facets that some but not
                // all survivors carry, minus what was already asked.
                let cand_sets: Vec<&BTreeSet<&'static str>> =
                    candidates.iter().map(|c| &self.items[c]).collect();
                cost += cand_sets.iter().map(|s| s.len()).sum::<usize>();

                let mut inter = cand_sets[0].clone();
                let mut union = cand_sets[0].clone();
                for s in &cand_sets[1..] {
                    inter = inter.intersection(s).copied().collect();
                    union = union.union(s).copied().collect();
                }
                let add_one_of: Vec<&'static str> = union
                    .difference(&inter)
                    .copied()
                    .filter(|f| !req.contains(f))
                    .collect();

                (
                    Verdict::Ambiguous {
                        matched,
                        candidates: candidates.into_iter().collect(),
                        add_one_of,
                    },
                    cost,
                )
            }
        }
    }

    /// Map a caller-supplied facet name to the field's own `&'static` copy,
    /// or `None` if it is not in the vocabulary.
    fn canonical(&self, facet: &str) -> Option<&'static str> {
        self.postings.keys().copied().find(|&k| k == facet)
    }

    /// Self-verify flatness: address cost is query-local, independent of how
    /// many irrelevant items the field holds. Resolving any facet must do the
    /// same work whether or not the field is padded with a thousand items
    /// that lack it — a search's cost grows with the field, an address's does
    /// not. Mirrors gc-project's `prove_instant()`.
    pub fn prove_flat(&self) -> bool {
        let mut padded = self.clone();
        for i in 0..1000 {
            // Decoys carry a facet that exists in no real query below, so
            // they never enter any candidate set.
            let name: &'static str = Box::leak(format!("__decoy_{i}").into_boxed_str());
            padded.register(name, &["__decoy_facet"]);
        }
        self.postings
            .keys()
            .all(|&facet| self.resolve_counting(&[facet]).1 == padded.resolve_counting(&[facet]).1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::ALL_KIND_TAGS;

    fn field() -> AddressedField {
        AddressedField::from_universe()
    }

    #[test]
    fn field_is_the_whole_census() {
        // Every kind tag is an item; nothing extra.
        let f = field();
        assert_eq!(f.len(), ALL_KIND_TAGS.len());
        for tag in ALL_KIND_TAGS {
            assert!(
                f.items.contains_key(tag.name()),
                "{} missing from the addressing field",
                tag.name()
            );
        }
    }

    #[test]
    fn address_a_unique_facet_combination() {
        // Only CoffeeMaker is both Cyclable and Powered.
        let f = field();
        assert_eq!(
            f.resolve(&["Cyclable", "Powered"]),
            Verdict::Address {
                tag: "CoffeeMaker",
                matched: vec!["Cyclable", "Powered"],
            }
        );
    }

    #[test]
    fn refuse_empty_request() {
        assert_eq!(field().resolve(&[]), Verdict::Refuse(RefuseReason::NoFacets));
    }

    #[test]
    fn refuse_unknown_facet() {
        assert_eq!(
            field().resolve(&["Teleporter"]),
            Verdict::Refuse(RefuseReason::UnknownFacet("Teleporter".into()))
        );
    }

    #[test]
    fn refuse_when_no_kind_carries_all() {
        // Known facets, but nothing is both a faucet and powered.
        let f = field();
        assert_eq!(
            f.resolve(&["SwitchableFaucet", "Powered"]),
            Verdict::Refuse(RefuseReason::NoMatch {
                matched: vec!["Powered", "SwitchableFaucet"],
            })
        );
    }

    #[test]
    fn ambiguous_names_a_discriminator() {
        // Container is carried by Fridge, Nightstand, Dresser.
        let f = field();
        match f.resolve(&["Container"]) {
            Verdict::Ambiguous { candidates, add_one_of, .. } => {
                assert_eq!(candidates, vec!["Dresser", "Fridge", "Nightstand"]);
                // Adding any of Fridge's extra facets reaches an address.
                assert!(add_one_of.contains(&"Powered"));
                assert!(add_one_of.contains(&"Openable"));
                assert!(add_one_of.contains(&"TimeDecayMinutes"));
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn discriminator_then_address_round_trip() {
        // Take the ambiguous verdict's advice and it resolves.
        let f = field();
        if let Verdict::Ambiguous { add_one_of, .. } = f.resolve(&["Container"]) {
            let pick = add_one_of[0];
            match f.resolve(&["Container", pick]) {
                Verdict::Address { tag, .. } => assert_eq!(tag, "Fridge"),
                other => panic!("expected Address after adding {pick}, got {other:?}"),
            }
        } else {
            panic!("expected Container to be ambiguous");
        }
    }

    #[test]
    fn facet_identical_kinds_have_no_discriminator() {
        // Sink, Shower, BathSink are all exactly {SwitchableFaucet}: the
        // field admits it cannot tell them apart rather than guessing.
        let f = field();
        match f.resolve(&["SwitchableFaucet"]) {
            Verdict::Ambiguous { candidates, add_one_of, .. } => {
                assert_eq!(candidates, vec!["BathSink", "Shower", "Sink"]);
                assert!(add_one_of.is_empty(), "expected no discriminator, got {add_one_of:?}");
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn never_mis_addresses_a_kinds_own_facets() {
        // Resolving any kind's own facet set can only ADDRESS that same
        // kind (it is always among the candidates), never a different one.
        let f = field();
        for entry in crate::kindgen::UNIVERSE_MANIFEST {
            let facets = entry.facets();
            if let Verdict::Address { tag, .. } = f.resolve(facets) {
                assert_eq!(
                    tag, entry.tag_name,
                    "{} addressed to {} by its own facets",
                    entry.tag_name, tag
                );
            }
        }
    }

    #[test]
    fn uniquely_shaped_kinds_address_by_their_own_facets() {
        // These kinds have a facet set no other kind's set is a superset of,
        // so their own description addresses them exactly.
        let f = field();
        for tag in ["Bed", "CoffeeMaker", "Fridge", "Stove", "Bathtub", "Lamp"] {
            let entry = crate::kindgen::manifest_entry_for(tag).unwrap();
            match f.resolve(entry.facets()) {
                Verdict::Address { tag: hit, .. } => assert_eq!(hit, tag),
                other => panic!("{tag} should self-address, got {other:?}"),
            }
        }
    }

    #[test]
    fn flatness_is_self_verified() {
        assert!(field().prove_flat());
    }

    #[test]
    fn address_cost_is_independent_of_field_size() {
        // Padding the field with kinds that lack the queried facet does not
        // change the work a query does.
        let base = field();
        let mut padded = field();
        for i in 0..500 {
            let name: &'static str = Box::leak(format!("decoy{i}").into_boxed_str());
            padded.register(name, &["DecorativeOnly"]);
        }
        assert_eq!(
            base.resolve_counting(&["Cyclable", "Powered"]).1,
            padded.resolve_counting(&["Cyclable", "Powered"]).1
        );
    }
}
