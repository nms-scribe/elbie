use crate::errors::ElbieError;
use crate::language::Language;
use crate::phoneme::Phoneme;
use crate::phoneme_table_builder::TableEntry;
use crate::word::Word;
use crate::word_table::WordTable;
use core::fmt::Display;
use core::slice::Iter;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::rc::Rc;

pub(crate) struct AnalysisConfig<'language> {
    language: &'language Language,
    cluster_sets: Vec<&'static str>,
    structure_sets: Vec<&'static str>
}

#[derive(Default)]
struct ClusterFollowers {
    in_cluster: BTreeMap<&'static str, (usize, Self)>,
    beyond_cluster: BTreeMap<&'static str, (usize, BTreeMap<&'static str, usize>)>,
    end_of_word: usize
}

impl ClusterFollowers {
    fn follow_with(&mut self, keys: &mut Iter<'_, &'static str>, next_out_of_cluster: Option<(&'static str, &'static str)>) {
        if let Some(first) = keys.next() {
            let value = self.in_cluster.entry(first).or_default();
            value.0 += 1;
            value.1.follow_with(keys, next_out_of_cluster)
        } else if let Some((cluster_type, set)) = next_out_of_cluster {
            let value = self.beyond_cluster.entry(cluster_type).or_default();
            value.0 += 1;
            *value.1.entry(set).or_insert(0) += 1;
        } else {
            self.end_of_word += 1;
        }
    }

    fn indented_fmt(&self, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { in_cluster,
                   beyond_cluster,
                   end_of_word } = self;

        for (set, (count, followers)) in in_cluster {
            writeln!(f, "{:indent$}[{set}] {count} ->", ' ')?;
            followers.indented_fmt(indent + 4, f)?;
        }

        for (cluster_type, (count, sets)) in beyond_cluster {
            writeln!(f, "{:indent$}NEXT [{cluster_type}] {count} ->", ' ')?;
            let indent = indent + 4;
            for (set, set_count) in sets {
                writeln!(f, "{:indent$}[{set}] {set_count}", ' ')?;
            }
        }

        if end_of_word > &0 {
            writeln!(f, "{:indent$}DONE <end of word> {end_of_word}", ' ')?;
        }

        Ok(())
    }
}

#[derive(Default)]
struct ClusterInfoByStructureClasses {
    #[expect(clippy::type_complexity, reason = "Yes, you're right, but I don't want to deal with this right now.")]
    clusters: BTreeMap<Vec<&'static str>, (usize, BTreeMap<Vec<Rc<Phoneme>>, usize>)>,
    tree: BTreeMap<&'static str, (usize, ClusterFollowers)>
}

impl Display for ClusterInfoByStructureClasses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { clusters,
                   tree } = self;

        writeln!(f, "### Clusters and Counts")?;
        writeln!(f)?;
        for (cluster, (count, info)) in clusters {
            write!(f, "\"{}\" ({count}) including: ", cluster.join(" + "))?;
            for (phonemes, phoneme_counts) in info {
                write!(f, "{} ({phoneme_counts}) ", Word::new(phonemes))?;
            }
            writeln!(f)?;
        }

        writeln!(f)?;
        writeln!(f, "### Tree")?;
        writeln!(f)?;
        writeln!(f, "```")?;
        for (set, (count, followers)) in tree {
            writeln!(f, "[{set}] {count} ->")?;
            followers.indented_fmt(4, f)?;
        }
        writeln!(f, "```")?;

        Ok(())
    }
}

impl ClusterInfoByStructureClasses {
    fn add_cluster(&mut self, phonemes: Vec<Rc<Phoneme>>, next_out_of_cluster: Option<&Rc<Phoneme>>, config: &AnalysisConfig) -> Result<(), ElbieError> {
        let key = phonemes.iter().map(|phoneme| config.find_structural_set_for_phoneme(phoneme)).collect::<Result<Vec<_>, _>>()?;

        // build the tree stuff...
        let mut keys = key.iter();
        if let Some(first) = keys.next() {
            let next_out_of_cluster = next_out_of_cluster.map(|phoneme| {
                                                             let cluster_type = config.find_cluster_set_for_phoneme(phoneme)?;
                                                             let set = config.find_structural_set_for_phoneme(phoneme)?;
                                                             Ok((cluster_type, set))
                                                         })
                                                         .transpose()?;
            let value = self.tree.entry(first).or_default();
            value.0 += 1;
            value.1.follow_with(&mut keys, next_out_of_cluster)
        }

        let info = self.clusters.entry(key).or_default();
        info.0 += 1;
        let clusters = info.1.entry(phonemes).or_default();
        *clusters += 1;

        Ok(())
    }
}

#[derive(Default)]
struct ClusterSetInfo {
    initial: ClusterInfoByStructureClasses,
    medial: ClusterInfoByStructureClasses,
    final_: ClusterInfoByStructureClasses,
    // i.e. how many words have the specified number of clusters. So, if a word has 5 vowel clusters, you might say it has five syllables (assuming one vowel cluster = one syllable)
    counts_per_word: BTreeMap<usize, usize>
}

impl Display for ClusterSetInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { initial,
                   medial,
                   final_,
                   counts_per_word } = self;

        writeln!(f, "## Counts Per Word")?;
        writeln!(f)?;

        let mut total = 0;
        let mut counts_per_word: Vec<_> = counts_per_word.iter()
                                                         .map(|(number, count)| {
                                                             total += count;
                                                             (number, count, total)
                                                         })
                                                         .collect();
        counts_per_word.sort_by_key(|item| item.0);
        let mut prev_total_at_least = total;
        let mut prev_number = 0;
        for (number, count, running) in counts_per_word {
            let total_at_least = (total - running) + count;
            write!(f, "{total_at_least} words have at least {number} instances")?;
            if number > &0 && (total_at_least != prev_total_at_least) {
                #[expect(clippy::integer_division, reason = "I'm displaying a rounded percentage anyway, so integer division is fine.")]
                let percent = (total_at_least * 100) / prev_total_at_least;
                writeln!(f, ", or {percent}% of words with at least {prev_number} instances.")?;
            } else {
                writeln!(f, ", or 100% of words.")?;
            }
            prev_total_at_least = total_at_least;
            prev_number = *number;
        }
        writeln!(f, "total of {total} words analyzed")?;
        writeln!(f)?;

        /*
        6 words have 0 instances -- 100/100
        44 words have 1 instances -- 94/94
        41 words have 2 instances -- 50/50
        7 words have 3 instances -- 9/9
        1 words have 4 instances -- 2/2
        1 words have 7 instances -- 1/1
        total of 100 words analyzed
        */

        writeln!(f, "## Initial Clusters")?;
        writeln!(f)?;
        writeln!(f, "{initial}")?;

        writeln!(f, "## Final Clusters")?;
        writeln!(f)?;
        writeln!(f, "{final_}")?;

        writeln!(f, "## Medial Clusters")?;
        writeln!(f)?;
        writeln!(f, "{medial}")?;

        Ok(())
    }
}

impl ClusterSetInfo {
    fn add_cluster(&mut self, cluster: Vec<Rc<Phoneme>>, initial: bool, final_: bool, next_out_of_cluster: Option<&Rc<Phoneme>>, config: &AnalysisConfig) -> Result<(), ElbieError> {
        if initial {
            if final_ {
                self.final_.add_cluster(cluster.clone(), next_out_of_cluster, config)?;
            }
            self.initial.add_cluster(cluster, next_out_of_cluster, config)
        } else if final_ {
            self.final_.add_cluster(cluster, next_out_of_cluster, config)
        } else {
            self.medial.add_cluster(cluster, next_out_of_cluster, config)
        }
    }

    fn add_cluster_counts_for_one_word(&mut self, count: usize) {
        *self.counts_per_word.entry(count).or_insert(0) += 1;
    }
}

#[derive(Default)]
pub(crate) struct AnalysisResults {
    info_by_set: BTreeMap<&'static str, ClusterSetInfo>
}

impl AnalysisResults {
    fn add_cluster(&mut self, cluster: Vec<Rc<Phoneme>>, set: &'static str, initial: bool, final_: bool, next_out_of_cluster: Option<&Rc<Phoneme>>, config: &AnalysisConfig) -> Result<(), ElbieError> {
        self.info_by_set.entry(set).or_default().add_cluster(cluster, initial, final_, next_out_of_cluster, config)
    }

    /// Add the count of the set per word.
    fn add_cluster_counts_for_one_word(&mut self, counts: BTreeMap<&'static str, usize>) {
        for (set, count) in counts {
            self.info_by_set.entry(set).or_default().add_cluster_counts_for_one_word(count)
        }
    }
}

impl Display for AnalysisResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (set, info) in &self.info_by_set {
            writeln!(f, "# {set}")?;
            writeln!(f)?;
            writeln!(f, "{info}")?;
        }
        Ok(())
    }
}

impl<'language> AnalysisConfig<'language> {
    pub(crate) fn from_language(language: &'language Language) -> Self {
        fn fill_structure_sets(structure_sets: &mut BTreeSet<&str>, table: &TableEntry) {
            // I'm filling a BTreeSet because there's a chance that a row's set is used in multiple tables
            // (i.e. you have rounded diphthongs as well as monopthongs). So I want to guarantee unique sets.
            // This does have the side effect of sorting them alphabetically.
            if let Some(rows) = table.definition().row_sets() {
                #[expect(clippy::iter_over_hash_type, reason = "Order isn't important here.")]
                for row in rows {
                    _ = structure_sets.insert(*row);
                }
            } else {
                _ = structure_sets.insert(table.set());
            }
        }

        let (cluster_sets, structure_sets) = match (language.analysis_cluster_sets(), language.analysis_structure_sets()) {
            (None, None) => {
                let mut cluster_sets = Vec::new();
                let mut structure_sets = BTreeSet::new();
                for table in language.tables() {
                    cluster_sets.push(table.set());
                    fill_structure_sets(&mut structure_sets, table);
                }
                (cluster_sets, structure_sets.into_iter().collect())
            },
            (None, Some(structure_sets)) => {
                let mut cluster_sets = Vec::new();
                for table in language.tables() {
                    cluster_sets.push(table.set());
                }
                (cluster_sets, structure_sets.clone())
            },
            (Some(cluster_sets), None) => {
                let mut structure_sets = BTreeSet::new();
                for table in language.tables() {
                    fill_structure_sets(&mut structure_sets, table);
                }
                (cluster_sets.clone(), structure_sets.into_iter().collect())
            },
            (Some(cluster_sets), Some(structure_sets)) => (cluster_sets.clone(), structure_sets.clone())
        };

        Self { language,
               cluster_sets,
               structure_sets }
    }
}

impl AnalysisConfig<'_> {
    fn validate_set_coverage(from: &Language, phoneme: &Rc<Phoneme>, sets: &[&'static str]) -> Result<(), ElbieError> {
        let mut found_set = None;
        for set in sets {
            if from.inventory().phoneme_is(phoneme, set)? {
                if let Some(previous_set) = found_set {
                    return Err(ElbieError::AnalysisSetIsNonExclusive(phoneme.name, set, previous_set));
                }
                found_set = Some(set);
            }
        }
        if found_set.is_none() {
            return Err(ElbieError::AnalysisSetCoverageIsIncomplete(phoneme.name, sets.join(", ")));
        }
        Ok(())
    }

    pub(crate) fn validate(&self, language: &Language) -> Result<(), ElbieError> {
        // make sure that the cluster_sets, and the structural_sets are exclusive:
        #[expect(clippy::iter_over_hash_type, reason = "Order isn't important here.")]
        for phoneme in language.inventory().phonemes().values() {
            Self::validate_set_coverage(language, phoneme, &self.cluster_sets)?;
            Self::validate_set_coverage(language, phoneme, &self.structure_sets)?;
        }

        Ok(())
    }

    pub(crate) fn analyze(&self, words: &WordTable) -> Result<AnalysisResults, ElbieError> {
        // TODO: What if You want to "separate" clusters? There should be a way to configure, say, not clustering vowels so each vowel is a separate cluster. Making it easier to count syllables in some languages.

        let mut results = AnalysisResults::default();
        for entry in &mut words.entries() {
            let word = entry.word();
            let word = self.language.read_word(word)?;
            let mut phonemes = word.phonemes().iter();
            let mut cluster_phonemes = Vec::new();
            let mut initial = true;
            let mut counts: BTreeMap<&str, usize> = BTreeMap::from_iter(self.cluster_sets.iter().copied().map(|set| (set, 0)));
            if let Some(first) = phonemes.next() {
                let mut set = self.find_cluster_set_for_phoneme(first)?;
                cluster_phonemes.push(first.clone());
                for next in phonemes {
                    let next_set = self.find_cluster_set_for_phoneme(next)?;
                    if set != next_set {
                        let index = counts.get_mut(set).expect("I'm iterating through the same sets the map was generated with, this should work");
                        results.add_cluster(cluster_phonemes, set, initial, false, Some(next), self)?;
                        *index += 1;
                        cluster_phonemes = Vec::new();
                        set = next_set;
                        initial = false;
                    }
                    cluster_phonemes.push(next.clone());
                }

                // close off the last one...
                let index = counts.get_mut(set).expect("I'm iterating through the same sets the map was generated with, this should work");
                *index += 1;
                results.add_cluster(cluster_phonemes, set, initial, true, None, self)?;
            }

            results.add_cluster_counts_for_one_word(counts);
        }

        Ok(results)
    }

    fn find_structural_set_for_phoneme(&self, phoneme: &Rc<Phoneme>) -> Result<&'static str, ElbieError> {
        self.find_set_for_phoneme(phoneme, &self.structure_sets)
    }

    fn find_cluster_set_for_phoneme(&self, phoneme: &Rc<Phoneme>) -> Result<&'static str, ElbieError> {
        self.find_set_for_phoneme(phoneme, &self.cluster_sets)
    }

    fn find_set_for_phoneme(&self, phoneme: &Rc<Phoneme>, sets: &[&'static str]) -> Result<&'static str, ElbieError> {
        for set in sets {
            if self.language.inventory().phoneme_is(phoneme, set)? {
                return Ok(set);
            }
        }
        // TODO: If we're doing this, then why do we need to validate this in the coverage?
        // -- I mean, we do have to validate the non-exclusivity
        Err(ElbieError::AnalysisSetCoverageIsIncomplete(phoneme.name, sets.join(", ")))
    }
}
