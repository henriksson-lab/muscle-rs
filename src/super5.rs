// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Super5 {
    pub min_ea_pass1: f32,
    pub input_seqs: Option<MultiSequence>,
    pub unique_seqs: Option<MultiSequence>,
    pub centroid_seqs: Option<MultiSequence>,
    pub centroid_msa: Option<MultiSequence>,
    pub extended_msa: Option<MultiSequence>,
    pub final_msa: Option<MultiSequence>,
    pub guide_tree_none: Tree,
    pub guide_tree_abc: Tree,
    pub guide_tree_acb: Tree,
    pub guide_tree_bca: Tree,
    pub final_msa_none: MultiSequence,
    pub final_msa_abc: MultiSequence,
    pub final_msa_acb: MultiSequence,
    pub final_msa_bca: MultiSequence,
    pub d: Derep,
    pub u: UClust,
    pub ta: TransAln,
    pub s4: Super4,
    pub is_dupe: Vec<bool>,
    pub is_centroid: Vec<bool>,
    pub is_member: Vec<bool>,
    pub dupe_gs_is: Vec<uint>,
    pub dupe_rep_gs_is: Vec<uint>,
    pub centroid_gs_is: Vec<uint>,
    pub member_gs_is: Vec<uint>,
    pub member_centroid_gs_is: Vec<uint>,
    pub centroid_seqs_seq_index_to_gsi: Vec<uint>,
    pub centroid_msa_seq_index_to_gsi: Vec<uint>,
    pub gsi_to_centroid_seqs_seq_index: Vec<uint>,
    pub gsi_to_centroid_msa_seq_index: Vec<uint>,
    pub gsi_to_member_count: Vec<uint>,
    pub gsi_to_centroid_gsi: Vec<uint>,
    pub centroid_gsi_to_member_gs_is: Vec<Vec<uint>>,
    pub dupe_rep_gsi_to_member_gs_is: Vec<Vec<uint>>,
    pub gsi_to_member_centroid_path: Vec<String>,
} // original: Super5 (muscle/src/super5.h)

/// Convert a byte vector of ASCII characters into a `String`.
pub fn char_vec_to_str(vec: &[byte]) -> String {
    let mut s = String::new();
    for &c in vec {
        s.push(char::from(c));
    }
    s
}

/// Apply CLI overrides for Super5 tunables (pass-1 minimum EA).
#[track_caller]
pub fn super5_set_opts(s5: &mut Super5, super5_minea1: Option<f32>) {
    // C++ super5.h: DEFAULT_MIN_EA_SUPER5_PASS1 = 0.99f
    s5.min_ea_pass1 = super5_minea1.unwrap_or(0.99);
}

/// Reset cached guide trees and final MSAs before a fresh Super5 run.
#[track_caller]
pub fn super5_clear_trees_and_ms_as(s5: &mut Super5) {
    s5.guide_tree_none = Tree::default();
    s5.guide_tree_abc = Tree::default();
    s5.guide_tree_acb = Tree::default();
    s5.guide_tree_bca = Tree::default();

    multi_sequence_clear(&mut s5.final_msa_none);
    multi_sequence_clear(&mut s5.final_msa_abc);
    multi_sequence_clear(&mut s5.final_msa_acb);
    multi_sequence_clear(&mut s5.final_msa_bca);
}

/// Dereplicate input sequences and run UClust to produce cluster
/// centroid sequences plus the supporting index/look-up vectors.
#[track_caller]
pub fn super5_make_centroid_seqs<FDerep, FUClust>(
    s5: &mut Super5,
    input_seqs: &MultiSequence,
    mut run_derep: FDerep,
    mut run_uclust: FUClust,
) where
    FDerep: FnMut(&mut Derep, &MultiSequence) -> MultiSequence,
    FUClust: FnMut(&mut UClust, &MultiSequence, f32) -> MultiSequence,
{
    s5.input_seqs = Some(input_seqs.clone());
    s5.unique_seqs = Some(MultiSequence::default());
    s5.centroid_seqs = Some(MultiSequence::default());
    s5.centroid_msa = Some(MultiSequence::default());

    let unique = run_derep(&mut s5.d, input_seqs);
    s5.unique_seqs = Some(unique);
    super5_set_dupe_vecs(s5);

    let unique_seqs = s5
        .unique_seqs
        .as_ref()
        .expect("Super5::MakeCentroidSeqs missing unique seqs")
        .clone();
    let centroid_seqs = run_uclust(&mut s5.u, &unique_seqs, s5.min_ea_pass1);
    s5.centroid_seqs = Some(centroid_seqs);
    super5_set_centroid_vecs(s5);
    super5_set_centroid_seqs_vecs(s5);
    super5_validate_vecs(s5);
}

/// Drive the Super5 pipeline: derep, UClust centroids, Super4 over the
/// centroids, then transit-align members and dupes into the final MSA.
#[track_caller]
pub fn super5_run<FDerep, FUClust, FSuper4>(
    s5: &mut Super5,
    input_seqs: &MultiSequence,
    perm: TREEPERM,
    input_order: bool,
    mut run_derep: FDerep,
    mut run_uclust: FUClust,
    mut run_super4: FSuper4,
) where
    FDerep: FnMut(&mut Derep, &MultiSequence) -> MultiSequence,
    FUClust: FnMut(&mut UClust, &MultiSequence, f32) -> MultiSequence,
    FSuper4: FnMut(&mut Super4, &MultiSequence, TREEPERM),
{
    super5_make_centroid_seqs(
        s5,
        input_seqs,
        |d, input_seqs| run_derep(d, input_seqs),
        |u, unique_seqs, min_ea| run_uclust(u, unique_seqs, min_ea),
    );
    let centroid_seqs = s5
        .centroid_seqs
        .as_ref()
        .expect("Super5::Run missing centroid seqs")
        .clone();
    run_super4(&mut s5.s4, &centroid_seqs, perm);

    if perm != TREEPERM::TP_All {
        s5.centroid_msa = Some(s5.s4.final_msa.clone());
        super5_set_centroid_msa_vecs(s5);
        super5_align_members(s5);
        super5_align_dupes(s5);
        let mut final_msa = s5
            .extended_msa
            .as_ref()
            .expect("Super5::Run missing extended MSA")
            .clone();
        super5_sort_msa(s5, &mut final_msa, input_order);
        s5.final_msa = Some(final_msa);
        return;
    }

    s5.centroid_msa = Some(s5.s4.final_msa_none.clone());
    super5_set_centroid_msa_vecs(s5);
    super5_align_members(s5);
    super5_align_dupes(s5);
    s5.final_msa_none = s5
        .extended_msa
        .as_ref()
        .expect("Super5::Run missing extended MSA for default tree")
        .clone();
    let mut final_msa_none = s5.final_msa_none.clone();
    super5_sort_msa(s5, &mut final_msa_none, input_order);
    s5.final_msa_none = final_msa_none;

    s5.centroid_msa = Some(s5.s4.final_msa_abc.clone());
    super5_set_centroid_msa_vecs(s5);
    super5_align_members(s5);
    super5_align_dupes(s5);
    s5.final_msa_abc = s5
        .extended_msa
        .as_ref()
        .expect("Super5::Run missing extended MSA for ABC tree")
        .clone();
    let mut final_msa_abc = s5.final_msa_abc.clone();
    super5_sort_msa(s5, &mut final_msa_abc, input_order);
    s5.final_msa_abc = final_msa_abc;

    s5.centroid_msa = Some(s5.s4.final_msa_acb.clone());
    super5_set_centroid_msa_vecs(s5);
    super5_align_members(s5);
    super5_align_dupes(s5);
    s5.final_msa_acb = s5
        .extended_msa
        .as_ref()
        .expect("Super5::Run missing extended MSA for ACB tree")
        .clone();
    let mut final_msa_acb = s5.final_msa_acb.clone();
    super5_sort_msa(s5, &mut final_msa_acb, input_order);
    s5.final_msa_acb = final_msa_acb;

    s5.centroid_msa = Some(s5.s4.final_msa_bca.clone());
    super5_set_centroid_msa_vecs(s5);
    super5_align_members(s5);
    super5_align_dupes(s5);
    s5.final_msa_bca = s5
        .extended_msa
        .as_ref()
        .expect("Super5::Run missing extended MSA for BCA tree")
        .clone();
    let mut final_msa_bca = s5.final_msa_bca.clone();
    super5_sort_msa(s5, &mut final_msa_bca, input_order);
    s5.final_msa_bca = final_msa_bca;
}

/// Index the centroid MSA: build `gsi_to_centroid_msa_seq_index` and the
/// reverse map.
#[track_caller]
pub fn super5_set_centroid_msa_vecs(s5: &mut Super5) {
    s5.centroid_msa_seq_index_to_gsi.clear();
    s5.gsi_to_centroid_msa_seq_index.clear();

    let centroid_seq_count = s5.centroid_gs_is.len() as uint;
    let centroid_msa = s5
        .centroid_msa
        .as_ref()
        .expect("Super5::SetCentroidMSAVecs null centroid MSA");
    let centroid_msa_seq_count = centroid_msa.seqs.len() as uint;
    assert_eq!(centroid_seq_count, centroid_msa_seq_count);

    let global_seq_count = get_global_ms_seq_count();
    s5.gsi_to_centroid_msa_seq_index
        .resize(global_seq_count as usize, uint::MAX);
    s5.centroid_msa_seq_index_to_gsi.clear();

    for centroid_msa_seq_index in 0..centroid_seq_count {
        let seq = &centroid_msa.seqs[centroid_msa_seq_index as usize];
        let gsi = get_gsi_by_label(&seq.label);
        assert!(gsi < global_seq_count);
        if s5.gsi_to_centroid_msa_seq_index[gsi as usize] != uint::MAX {
            panic!(
                "Super5::SetCentroidMSAVecs() GSI={} found twice ({},{})",
                gsi, s5.gsi_to_centroid_msa_seq_index[gsi as usize], centroid_msa_seq_index
            );
        }
        s5.gsi_to_centroid_msa_seq_index[gsi as usize] = centroid_msa_seq_index;
        s5.centroid_msa_seq_index_to_gsi.push(gsi);
    }
}

/// Index the centroid sequences set: build `gsi_to_centroid_seqs_seq_index`
/// and the reverse map.
#[track_caller]
pub fn super5_set_centroid_seqs_vecs(s5: &mut Super5) {
    s5.centroid_seqs_seq_index_to_gsi.clear();
    s5.gsi_to_centroid_seqs_seq_index.clear();

    let centroid_seq_count = s5.centroid_gs_is.len() as uint;
    let centroid_seqs = s5
        .centroid_seqs
        .as_ref()
        .expect("Super5::SetCentroidSeqsVecs null centroid seqs");
    let centroid_seq_seq_count = centroid_seqs.seqs.len() as uint;
    assert_eq!(centroid_seq_count, centroid_seq_seq_count);

    let global_seq_count = get_global_ms_seq_count();
    s5.gsi_to_centroid_seqs_seq_index
        .resize(global_seq_count as usize, uint::MAX);
    s5.centroid_seqs_seq_index_to_gsi.clear();

    for centroid_seq_seq_index in 0..centroid_seq_count {
        let seq = &centroid_seqs.seqs[centroid_seq_seq_index as usize];
        let gsi = get_gsi_by_label(&seq.label);
        assert!(gsi < global_seq_count);
        assert_eq!(s5.gsi_to_centroid_seqs_seq_index[gsi as usize], uint::MAX);
        s5.gsi_to_centroid_seqs_seq_index[gsi as usize] = centroid_seq_seq_index;
        s5.centroid_seqs_seq_index_to_gsi.push(gsi);
    }
}

/// Populate the duplicate-handling vectors from the Derep results.
#[track_caller]
pub fn super5_set_dupe_vecs(s5: &mut Super5) {
    let input_seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::SetDupeVecs null input seqs")
        .seqs
        .len() as uint;
    s5.dupe_gs_is.clear();
    s5.dupe_rep_gs_is.clear();
    s5.is_dupe.clear();
    s5.dupe_rep_gsi_to_member_gs_is.clear();
    s5.dupe_rep_gsi_to_member_gs_is
        .resize(input_seq_count as usize, Vec::new());

    let (dupe_gs_is, dupe_rep_gs_is) = derep_get_dupe_gs_is(&s5.d);
    s5.dupe_gs_is = dupe_gs_is;
    s5.dupe_rep_gs_is = dupe_rep_gs_is;

    s5.is_dupe.resize(input_seq_count as usize, false);
    let dupe_count = s5.dupe_gs_is.len();
    for i in 0..dupe_count {
        let gsi = s5.dupe_gs_is[i];
        let dupe_rep_gsi = s5.dupe_rep_gs_is[i];
        assert!(gsi < input_seq_count);
        assert!(!s5.is_dupe[gsi as usize]);
        s5.is_dupe[gsi as usize] = true;
        s5.dupe_rep_gsi_to_member_gs_is[dupe_rep_gsi as usize].push(gsi);
    }
}

/// Populate the centroid/member vectors and per-GSI flags from UClust
/// results.
#[track_caller]
pub fn super5_set_centroid_vecs(s5: &mut Super5) {
    let input_seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::SetCentroidVecs null input seqs")
        .seqs
        .len() as uint;

    s5.centroid_gs_is.clear();
    s5.member_gs_is.clear();
    s5.member_centroid_gs_is.clear();

    s5.gsi_to_centroid_gsi.clear();
    s5.gsi_to_centroid_gsi
        .resize(input_seq_count as usize, uint::MAX);

    s5.centroid_gsi_to_member_gs_is.clear();
    s5.centroid_gsi_to_member_gs_is
        .resize(input_seq_count as usize, Vec::new());

    let (centroid_gs_is, member_gs_is, member_centroid_gs_is, gsi_to_member_centroid_path) =
        u_clust_get_gs_is(&s5.u);
    s5.centroid_gs_is = centroid_gs_is;
    s5.member_gs_is = member_gs_is;
    s5.member_centroid_gs_is = member_centroid_gs_is;
    s5.gsi_to_member_centroid_path = gsi_to_member_centroid_path;

    s5.is_centroid.clear();
    s5.is_member.clear();

    s5.is_centroid.resize(input_seq_count as usize, false);
    s5.is_member.resize(input_seq_count as usize, false);

    let gsi_count = get_global_ms_seq_count();
    s5.gsi_to_member_count.resize(gsi_count as usize, 0);

    let centroid_count = s5.centroid_gs_is.len();
    for i in 0..centroid_count {
        let centroid_gsi = s5.centroid_gs_is[i];
        assert!(centroid_gsi < input_seq_count);
        assert!(!s5.is_dupe[centroid_gsi as usize]);
        assert!(!s5.is_centroid[centroid_gsi as usize]);
        s5.is_centroid[centroid_gsi as usize] = true;
    }

    let member_count = s5.member_gs_is.len();
    assert_eq!(s5.member_centroid_gs_is.len(), member_count);
    for i in 0..member_count {
        let member_gsi = s5.member_gs_is[i];
        let member_centroid_gsi = s5.member_centroid_gs_is[i];

        assert!(member_gsi < gsi_count);
        assert!(member_centroid_gsi < gsi_count);
        assert!(s5.is_centroid[member_centroid_gsi as usize]);

        let is_dupe = s5.is_dupe[member_gsi as usize];
        let is_member = s5.is_member[member_gsi as usize];
        let is_centroid = s5.is_centroid[member_gsi as usize];
        if is_dupe || is_member || is_centroid {
            panic!(
                "Super5::SetCentroidVecs(), MemberGSI={} dupe={} mem={} cent={}",
                member_gsi,
                if is_dupe { 'T' } else { 'F' },
                if is_member { 'T' } else { 'F' },
                if is_centroid { 'T' } else { 'F' }
            );
        }

        assert!(!is_dupe);
        assert!(!is_member);
        assert!(!is_centroid);

        s5.is_member[member_gsi as usize] = true;
        s5.gsi_to_centroid_gsi[member_gsi as usize] = member_centroid_gsi;
        s5.centroid_gsi_to_member_gs_is[member_centroid_gsi as usize].push(member_gsi);
        s5.gsi_to_member_count[member_centroid_gsi as usize] += 1;
    }
}

/// Dump a per-GSI table of cluster categories, indices and label chains.
#[track_caller]
pub fn super5_log_clusters(s5: &Super5) -> String {
    let input_seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::LogClusters null input seqs")
        .seqs
        .len();
    assert_eq!(s5.is_dupe.len(), input_seq_count);
    assert_eq!(s5.is_centroid.len(), input_seq_count);
    assert_eq!(s5.is_member.len(), input_seq_count);
    assert_eq!(s5.gsi_to_centroid_seqs_seq_index.len(), input_seq_count);
    assert_eq!(s5.gsi_to_centroid_msa_seq_index.len(), input_seq_count);
    assert_eq!(s5.gsi_to_member_count.len(), input_seq_count);

    let mut out = String::new();
    out.push_str(&format!("{:>5.5}", "GSI"));
    out.push_str(&format!("  {:>3.3}", "Cat"));
    out.push_str(&format!("  {:>5.5}", "CSSI"));
    out.push_str(&format!("  {:>5.5}", "CMSI"));
    out.push_str(&format!("  {:>5.5}", "MbCt"));
    out.push_str(&format!("  {:>5.5}", "Cent"));
    out.push('\n');

    for gsi in 0..input_seq_count {
        let label = get_label_by_gsi(gsi as uint);
        let dupe = s5.is_dupe[gsi];
        let centroid = s5.is_centroid[gsi];
        let member = s5.is_member[gsi];
        let cat = if dupe {
            "Dup"
        } else if centroid {
            "Cnt"
        } else if member {
            "Mem"
        } else {
            panic!("Super5::LogClusters invalid category");
        };

        let centroid_seqs_seq_index = s5.gsi_to_centroid_seqs_seq_index[gsi];
        let centroid_msa_seq_index = s5.gsi_to_centroid_msa_seq_index[gsi];
        let member_count = s5.gsi_to_member_count[gsi];
        let centroid_gsi = s5.gsi_to_centroid_gsi[gsi];
        let centroid_label = if centroid_gsi != uint::MAX {
            get_label_by_gsi(centroid_gsi)
        } else {
            String::new()
        };

        out.push_str(&format!("{gsi:5}"));
        out.push_str(&format!("  {cat:3.3}"));

        if centroid_seqs_seq_index == uint::MAX {
            out.push_str(&format!("  {:>5.5}", "*"));
        } else {
            out.push_str(&format!("  {centroid_seqs_seq_index:5}"));
        }

        if centroid_msa_seq_index == uint::MAX {
            out.push_str(&format!("  {:>5.5}", "*"));
        } else {
            out.push_str(&format!("  {centroid_msa_seq_index:5}"));
        }

        out.push_str(&format!("  {member_count:5}"));

        if centroid_gsi == uint::MAX {
            out.push_str(&format!("  {:>5.5}", "*"));
        } else {
            out.push_str(&format!("  {centroid_gsi:5}"));
        }

        out.push_str(&format!("  >{label}"));
        if !centroid_label.is_empty() {
            out.push_str(&format!("  >> {centroid_label}"));
        }

        let members = &s5.centroid_gsi_to_member_gs_is[gsi];
        let m = members.len();
        for member in members.iter().take(std::cmp::min(m, 4)) {
            out.push_str(&format!(" <{member}>"));
        }
        if m > 4 {
            out.push_str(" ...");
        }

        let dupe_members = &s5.dupe_rep_gsi_to_member_gs_is[gsi];
        let dm = dupe_members.len();
        for dupe_member in dupe_members.iter().take(std::cmp::min(dm, 4)) {
            out.push_str(&format!(" ={dupe_member}"));
        }
        if dm > 4 {
            out.push_str(" ...");
        }

        out.push('\n');
    }
    out
}

/// Assert every input sequence is classified as exactly one of
/// duplicate / centroid / member.
#[track_caller]
pub fn super5_validate_vecs(s5: &Super5) {
    let input_seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::ValidateVecs null input seqs")
        .seqs
        .len();
    assert_eq!(s5.is_dupe.len(), input_seq_count);
    assert_eq!(s5.is_centroid.len(), input_seq_count);
    assert_eq!(s5.is_member.len(), input_seq_count);

    for i in 0..input_seq_count {
        let dupe = s5.is_dupe[i];
        let centroid = s5.is_centroid[i];
        let member = s5.is_member[i];
        let sum = dupe as i32 + centroid as i32 + member as i32;
        if sum != 1 {
            panic!(
                "Input seq {} dupe {}, centroid {} member {}",
                i,
                if dupe { 'T' } else { 'F' },
                if centroid { 'T' } else { 'F' },
                if member { 'T' } else { 'F' }
            );
        }
    }
}

/// Project every cluster member into the centroid MSA via transitive
/// alignment to produce the extended MSA.
#[track_caller]
pub fn super5_align_members(s5: &mut Super5) {
    let member_count = s5.member_gs_is.len() as uint;
    let gsi_count = get_gsi_count();
    assert_eq!(s5.member_centroid_gs_is.len() as uint, member_count);
    let centroid_msa = s5
        .centroid_msa
        .as_ref()
        .expect("Super5::AlignMembers null centroid MSA")
        .clone();
    let centroid_count = centroid_msa.seqs.len() as uint;
    let _centroid_msa_col_count = multi_sequence_get_col_count(&centroid_msa);

    let mut member_index_to_centroid_index = Vec::new();
    assert_eq!(s5.member_centroid_gs_is.len() as uint, member_count);
    assert_eq!(s5.gsi_to_centroid_msa_seq_index.len() as uint, gsi_count);
    assert_eq!(s5.gsi_to_member_centroid_path.len() as uint, gsi_count);

    let mut member_seqs = MultiSequence::default();
    let mut member_paths = Vec::new();
    for member_index in 0..member_count {
        let member_gsi = s5.member_gs_is[member_index as usize];
        let member_seq = get_global_input_seq_by_index(member_gsi);
        member_seqs.seqs.push(member_seq);
        member_seqs.owners.push(false);

        let centroid_gsi = s5.member_centroid_gs_is[member_index as usize];
        assert!(centroid_gsi < gsi_count);
        let centroid_msa_seq_index = s5.gsi_to_centroid_msa_seq_index[centroid_gsi as usize];
        assert!(centroid_msa_seq_index < centroid_count);
        member_index_to_centroid_index.push(centroid_msa_seq_index);

        let path = &s5.gsi_to_member_centroid_path[member_gsi as usize];
        assert!(!path.is_empty());
        member_paths.push(path.clone());
    }

    trans_aln_init(
        &mut s5.ta,
        &centroid_msa,
        &member_seqs,
        &member_index_to_centroid_index,
        &member_paths,
    );
    trans_aln_make_extended_msa(&mut s5.ta);
    let extended_msa = s5
        .ta
        .extended_msa
        .as_ref()
        .expect("Super5::AlignMembers missing extended MSA")
        .clone();
    s5.extended_msa = Some(extended_msa);
    assert_seqs_eq_input(
        "super5.cpp",
        386,
        s5.extended_msa
            .as_ref()
            .expect("Super5::AlignMembers missing extended MSA"),
    );
}

/// Reinsert duplicate sequences into the extended MSA by cloning their
/// representative's aligned row.
#[track_caller]
pub fn super5_align_dupes(s5: &mut Super5) -> String {
    let dupe_count = s5.dupe_gs_is.len() as uint;
    assert_eq!(s5.dupe_rep_gs_is.len() as uint, dupe_count);
    if dupe_count == 0 {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!("Inserting {dupe_count} dupes..."));
    let gsi_count = get_gsi_count();
    let mut gsi_to_extended_seq_index = vec![uint::MAX; gsi_count as usize];
    let extended_msa = s5
        .extended_msa
        .as_mut()
        .expect("Super5::AlignDupes null extended MSA");
    let extended_seq_count = extended_msa.seqs.len() as uint;
    for extended_seq_index in 0..extended_seq_count {
        let seq = &extended_msa.seqs[extended_seq_index as usize];
        let gsi = get_gsi_by_label(&seq.label);
        assert!(gsi < gsi_count);
        assert_eq!(gsi_to_extended_seq_index[gsi as usize], uint::MAX);
        gsi_to_extended_seq_index[gsi as usize] = extended_seq_index;
    }

    for i in 0..dupe_count {
        let dupe_gsi = s5.dupe_gs_is[i as usize];
        let rep_gsi = s5.dupe_rep_gs_is[i as usize];
        assert!(rep_gsi < gsi_count);
        let rep_extended_seq_index = gsi_to_extended_seq_index[rep_gsi as usize];
        assert!(rep_extended_seq_index < extended_seq_count);
        let rep = &extended_msa.seqs[rep_extended_seq_index as usize];
        let mut aligned_dupe = sequence_clone(rep);
        aligned_dupe.label = get_label_by_gsi(dupe_gsi);
        extended_msa.seqs.push(aligned_dupe);
        extended_msa.owners.push(true);
    }
    out.push_str(" done.\n");
    assert_seqs_eq_input("super5.cpp", 425, extended_msa);
    out
}

/// Build a label-to-row-index map for an MSA; panics on duplicates.
#[track_caller]
pub fn super5_get_label_to_aln_seq_index(
    _s5: &Super5,
    aln: &MultiSequence,
) -> std::collections::BTreeMap<String, uint> {
    let mut label_to_aln_seq_index = std::collections::BTreeMap::new();
    let seq_count = aln.seqs.len() as uint;
    for i in 0..seq_count {
        let label = aln.seqs[i as usize].label.clone();
        if label_to_aln_seq_index.contains_key(&label) {
            panic!("Duplicate label >{label}");
        }
        label_to_aln_seq_index.insert(label, i);
    }
    label_to_aln_seq_index
}

/// Sort the MSA rows by input order or guide-tree order.
#[track_caller]
pub fn super5_sort_msa(s5: &Super5, aln: &mut MultiSequence, input_order: bool) {
    if input_order {
        super5_sort_msa_by_input_order(s5, aln);
    } else {
        super5_sort_msa_by_guide_tree(s5, aln);
    }
}

/// Look up the input label for a global sequence index.
#[track_caller]
pub fn super5_get_label(s5: &Super5, gsi: uint) -> &str {
    s5.input_seqs
        .as_ref()
        .expect("Super5::GetLabel null input seqs")
        .seqs[gsi as usize]
        .label
        .as_str()
}

/// Append the label for `gsi` followed by its duplicate members.
#[track_caller]
pub fn super5_append_labels(s5: &Super5, gsi: uint, labels: &mut Vec<String>) {
    let label = super5_get_label(s5, gsi);
    labels.push(label.to_string());
    assert!((gsi as usize) < s5.dupe_rep_gsi_to_member_gs_is.len());
    let dupe_gs_is = &s5.dupe_rep_gsi_to_member_gs_is[gsi as usize];
    let d = dupe_gs_is.len();
    for dupe_gsi in dupe_gs_is.iter().take(d) {
        let label = super5_get_label(s5, *dupe_gsi);
        labels.push(label.to_string());
    }
}

/// Append the centroid's labels followed by every member's labels.
#[track_caller]
pub fn super5_append_labels_from_centroid(
    s5: &Super5,
    centroid_index: uint,
    labels: &mut Vec<String>,
) {
    assert!((centroid_index as usize) < s5.centroid_gs_is.len());
    let centroid_gsi = s5.centroid_gs_is[centroid_index as usize];
    super5_append_labels(s5, centroid_gsi, labels);
    assert!((centroid_gsi as usize) < s5.centroid_gsi_to_member_gs_is.len());
    let member_gs_is = &s5.centroid_gsi_to_member_gs_is[centroid_gsi as usize];
    let m = member_gs_is.len();
    for member_gsi in member_gs_is.iter().take(m) {
        super5_append_labels(s5, *member_gsi, labels);
    }
}

/// Enumerate all input labels in centroid (guide-tree) traversal order.
#[track_caller]
pub fn super5_get_labels_in_guide_tree_order(s5: &Super5) -> Vec<String> {
    let mut labels = Vec::new();
    let centroid_count = s5.centroid_gs_is.len() as uint;
    for centroid_index in 0..centroid_count {
        super5_append_labels_from_centroid(s5, centroid_index, &mut labels);
    }

    let seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::GetLabelsInGuideTreeOrder null input seqs")
        .seqs
        .len();
    assert_eq!(labels.len(), seq_count);
    labels
}

/// Reorder MSA rows so they follow centroid (guide-tree) traversal order.
#[track_caller]
pub fn super5_sort_msa_by_guide_tree(s5: &Super5, aln: &mut MultiSequence) {
    let seq_count = s5
        .input_seqs
        .as_ref()
        .expect("Super5::SortMSA_ByGuideTree null input seqs")
        .seqs
        .len() as uint;

    let label_to_msa_seq_index = super5_get_label_to_aln_seq_index(s5, aln);

    let labels = super5_get_labels_in_guide_tree_order(s5);
    assert_eq!(labels.len() as uint, seq_count);

    let mut sorted_seqs = Vec::new();
    for i in 0..seq_count {
        let label = &labels[i as usize];
        let msa_seq_index = *label_to_msa_seq_index
            .get(label)
            .unwrap_or_else(|| panic!("Super5::SortMSA_ByGuideTree(), missing >{label}"));
        let seq = aln.seqs[msa_seq_index as usize].clone();
        sorted_seqs.push(seq);
    }

    aln.seqs[..(seq_count as usize)].clone_from_slice(&sorted_seqs[..(seq_count as usize)]);
}

/// Reorder MSA rows so they match the original input sequence order.
#[track_caller]
pub fn super5_sort_msa_by_input_order(s5: &Super5, aln: &mut MultiSequence) {
    let input_seqs = s5
        .input_seqs
        .as_ref()
        .expect("Super5::SortMSA_ByInputOrder null input seqs");
    let seq_count = input_seqs.seqs.len() as uint;
    let label_to_msa_seq_index = super5_get_label_to_aln_seq_index(s5, aln);

    let mut sorted_seqs = Vec::new();
    for i in 0..seq_count {
        let label = &input_seqs.seqs[i as usize].label;
        let msa_seq_index = *label_to_msa_seq_index
            .get(label)
            .unwrap_or_else(|| panic!("Super5::SortMSA_ByInputOrder(), missing >{label}"));
        let seq = aln.seqs[msa_seq_index as usize].clone();
        sorted_seqs.push(seq);
    }

    aln.seqs[..(seq_count as usize)].clone_from_slice(&sorted_seqs[..(seq_count as usize)]);
}

/// CLI entry point for `super5`: load input, run the pipeline (optionally
/// over all tree permutations), and write the resulting MSA file(s).
#[track_caller]
pub fn cmd_super5<FRunSuper5>(
    input_file_name: &str,
    output_pattern: &str,
    force_nucleo: Option<bool>,
    tree_perm: Option<TREEPERM>,
    perturb_seed: Option<uint>,
    super5_minea1: Option<f32>,
    input_order: bool,
    mega: bool,
    diversified: bool,
    replicates: bool,
    stratified: bool,
    mut run_super5: FRunSuper5,
) -> (Super5, Vec<String>, String)
where
    FRunSuper5: FnMut(&mut Super5, &MultiSequence, TREEPERM, bool) -> String,
{
    if mega {
        die("-super5 does not support -mega, use -super7");
    }

    // Match C++ super5.cpp:560 `LoadInput(InputSeqs)`, which auto-detects
    // `.mega` extension. super5 doesn't actually use mega profiles but the
    // file load path needs to handle the extension so the later error
    // (Mega::GetProfileByLabel) parallels C++.
    let input_seqs = load_input(input_file_name, mega);

    if output_pattern.is_empty() {
        die("Must set -output");
    }

    let input_seq_count = get_global_ms_seq_count();
    let _ = input_seq_count;

    let nucleo = force_nucleo.unwrap_or_else(|| multi_sequence_guess_is_nucleo(&input_seqs));
    set_alpha_l209(if nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    set_perturb_seed(perturb_seed);
    init_probcons();

    if diversified {
        die("-diversified not supported");
    }
    if replicates {
        die("-replicates not supported");
    }
    if stratified {
        die("-stratified not supported");
    }

    let perm = tree_perm.unwrap_or(TREEPERM::TP_None);
    if perm == TREEPERM::TP_All && !output_pattern.contains('@') {
        die("Must be '@' in output filename with -perm all");
    }

    let mut s5 = Super5::default();
    super5_set_opts(&mut s5, super5_minea1);
    let log = run_super5(&mut s5, &input_seqs, perm, input_order);
    let seed = perturb_seed.unwrap_or(0);
    let mut output_file_names = Vec::new();
    if perm == TREEPERM::TP_All {
        let file_name_none = make_replicate_file_name(output_pattern, TREEPERM::TP_None, seed);
        let file_name_abc = make_replicate_file_name(output_pattern, TREEPERM::TP_ABC, seed);
        let file_name_acb = make_replicate_file_name(output_pattern, TREEPERM::TP_ACB, seed);
        let file_name_bca = make_replicate_file_name(output_pattern, TREEPERM::TP_BCA, seed);
        multi_sequence_write_mfa(&s5.final_msa_none, &file_name_none);
        multi_sequence_write_mfa(&s5.final_msa_abc, &file_name_abc);
        multi_sequence_write_mfa(&s5.final_msa_acb, &file_name_acb);
        multi_sequence_write_mfa(&s5.final_msa_bca, &file_name_bca);
        output_file_names.push(file_name_none);
        output_file_names.push(file_name_abc);
        output_file_names.push(file_name_acb);
        output_file_names.push(file_name_bca);
    } else {
        let output_file_name = if output_pattern.contains('@') {
            make_replicate_file_name(output_pattern, perm, seed)
        } else {
            output_pattern.to_string()
        };
        let final_msa = s5.final_msa.as_ref().expect("cmd_super5 missing final MSA");
        multi_sequence_write_mfa(final_msa, &output_file_name);
        output_file_names.push(output_file_name);
    }
    if let Some(final_msa) = s5.final_msa.as_mut() {
        multi_sequence_clear(final_msa);
    }
    s5.final_msa = None;
    multi_sequence_clear(&mut s5.final_msa_none);
    multi_sequence_clear(&mut s5.final_msa_abc);
    multi_sequence_clear(&mut s5.final_msa_acb);
    multi_sequence_clear(&mut s5.final_msa_bca);
    (s5, output_file_names, log)
}
