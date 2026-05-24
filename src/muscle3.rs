// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Muscle3 {
    pub ap_addr: Option<usize>,
    pub ap: Option<M3AlnParams>,
    pub input_seqs_addr: Option<usize>,
    pub input_seqs: Option<MultiSequence>,
    pub dist_mx: Vec<Vec<f32>>,
    pub k66: KmerDist66,
    pub k33: KmerDist33,
    pub labels: Vec<String>,
    pub guide_tree: Tree,
    pub cw: ClustalWeights,
    pub input_seq_weights: Vec<f32>,
    pub pp3: PProg3,
    pub u5: UPGMA5,
    pub final_msa_addr: Option<usize>,
    pub final_msa: Option<MultiSequence>,
} // original: Muscle3 (muscle/src/muscle3.h)

/// Run the full Muscle3 pipeline: k-mer distance, UPGMA tree, weights, progressive alignment, tree iters.
#[track_caller]
pub fn muscle3_run<FRunUpgma, FRunPProg3>(
    m3: &mut Muscle3,
    ap: &M3AlnParams,
    input_seqs: &MultiSequence,
    mut run_upgma: FRunUpgma,
    mut run_pp3: FRunPProg3,
) -> MultiSequence
where
    FRunUpgma: FnMut(&mut UPGMA5, &str, &mut Tree),
    FRunPProg3: FnMut(&mut PProg3, &MultiSequence, &[f32], &Tree) -> MultiSequence,
{
    m3.ap_addr = Some(ap as *const M3AlnParams as usize);
    m3.ap = Some(ap.clone());
    assert!(m3.ap.as_ref().unwrap().ready);

    m3.input_seqs_addr = Some(input_seqs as *const MultiSequence as usize);
    m3.input_seqs = Some(input_seqs.clone());
    let seq_count = input_seqs.seqs.len() as uint;

    let kd = m3.ap.as_ref().unwrap().kmer_dist.clone();
    if kd == "66" {
        m3.dist_mx = kmer_dist66_get_dist_mx(input_seqs);
    } else if kd == "33" {
        m3.dist_mx = kmer_dist33_get_dist_mx(input_seqs);
    } else {
        die(&format!("Muscle3::Run, m_AP->m_KmerDist={kd}"));
    }

    m3.labels.clear();
    for i in 0..seq_count {
        m3.labels.push(input_seqs.seqs[i as usize].label.clone());
    }

    upgma5_init(&mut m3.u5, &m3.labels, &m3.dist_mx);
    let linkage = m3.ap.as_ref().unwrap().linkage.clone();
    run_upgma(&mut m3.u5, &linkage, &mut m3.guide_tree);

    m3.input_seq_weights = clustal_weights_run(&mut m3.cw, input_seqs, &m3.guide_tree);
    m3.pp3.ap = m3.ap.clone();
    let msa = run_pp3(
        &mut m3.pp3,
        input_seqs,
        &m3.input_seq_weights,
        &m3.guide_tree,
    );
    m3.pp3.msa = msa;
    assert!(multi_sequence_is_aligned(&m3.pp3.msa));

    let tree_iters = m3.ap.as_ref().unwrap().tree_iters;
    for _tree_iter in 0..tree_iters {
        let mut msa_input_order = MultiSequence {
            seqs: vec![Sequence::default(); seq_count as usize],
            owners: vec![false; seq_count as usize],
            ..MultiSequence::default()
        };
        for k in 0..seq_count {
            let seq = &m3.pp3.msa.seqs[k as usize];
            let seq_index = get_gsi_by_label(&seq.label);
            assert!(seq_index < seq_count);
            msa_input_order.seqs[seq_index as usize] = seq.clone();
        }

        m3.dist_mx = get_kimura_dist_mx(&msa_input_order);
        {
            let ap_mut = m3.ap.as_mut().unwrap();
            m3_aln_params_perturb_dist_mx(ap_mut, &mut m3.dist_mx);
        }

        upgma5_init(&mut m3.u5, &m3.labels, &m3.dist_mx);
        let linkage = m3.ap.as_ref().unwrap().linkage.clone();
        run_upgma(&mut m3.u5, &linkage, &mut m3.guide_tree);

        m3.input_seq_weights = clustal_weights_run(&mut m3.cw, input_seqs, &m3.guide_tree);
        m3.pp3.ap = m3.ap.clone();
        let msa = run_pp3(
            &mut m3.pp3,
            input_seqs,
            &m3.input_seq_weights,
            &m3.guide_tree,
        );
        m3.pp3.msa = msa;
        assert!(multi_sequence_is_aligned(&m3.pp3.msa));
    }
    m3.final_msa_addr = Some(&m3.pp3.msa as *const MultiSequence as usize);
    m3.final_msa = Some(m3.pp3.msa.clone());
    m3.final_msa.as_ref().unwrap().clone()
}

/// Write the final Muscle3 MSA to a FASTA file (no-op if `file_name` is empty).
#[track_caller]
pub fn muscle3_write_msa(m3: &Muscle3, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    assert!(m3.final_msa.is_some());
    multi_sequence_write_mfa(m3.final_msa.as_ref().unwrap(), file_name);
}

/// Run Muscle3 in random-order mode: iteratively add a random sequence to the growing profile.
#[track_caller]
pub fn muscle3_run_ro<FNwSmall3, FAlignTwoProfsGivenPath>(
    m3: &mut Muscle3,
    _ap: &M3AlnParams,
    input_seqs: &MultiSequence,
    mut nw_small3: FNwSmall3,
    mut align_two_profs_given_path: FAlignTwoProfsGivenPath,
) -> MultiSequence
where
    FNwSmall3: FnMut(&mut CacheMem3, &Profile3, &Profile3) -> String,
    FAlignTwoProfsGivenPath:
        FnMut(&Profile3, f32, &Profile3, f32, &[[f32; 20]; 20], f32, &str, &mut Profile3),
{
    assert!(m3.ap.is_some());
    assert!(m3.ap.as_ref().unwrap().ready);
    m3.input_seqs_addr = Some(input_seqs as *const MultiSequence as usize);
    m3.input_seqs = Some(input_seqs.clone());
    let seq_count = input_seqs.seqs.len() as uint;
    let mut order = Vec::new();
    for i in 0..seq_count {
        order.push(i);
    }
    shuffle(&mut order);

    let weights = vec![1.0_f32];

    let seq_index0 = order[0];
    let seq0 = input_seqs.seqs[seq_index0 as usize].clone();
    let mut accumulated_msa = MultiSequence::default();
    accumulated_msa.seqs.push(seq0);
    accumulated_msa.owners.push(false);
    let mut accumulated_prof = Profile3::default();
    {
        let ap = m3.ap.as_ref().unwrap();
        profile3_from_msa(
            &mut accumulated_prof,
            &accumulated_msa,
            &ap.subst_mx_letter,
            ap.gap_open,
            &weights,
        );
    }

    let mut cm = CacheMem3::default();
    for k in 1..seq_count {
        let seq_indexk = order[k as usize];
        let seqk = input_seqs.seqs[seq_indexk as usize].clone();
        let mut msa_k = MultiSequence::default();
        msa_k.seqs.push(seqk);
        msa_k.owners.push(false);
        let mut prof_k = Profile3::default();
        {
            let ap = m3.ap.as_ref().unwrap();
            profile3_from_msa(
                &mut prof_k,
                &msa_k,
                &ap.subst_mx_letter,
                ap.gap_open,
                &weights,
            );
        }

        let path = nw_small3(&mut cm, &prof_k, &accumulated_prof);

        let relative_weight_one_seq = 1.0_f32 / k as f32;
        let relative_weight_accumulated_seqs = k as f32;
        let sum = relative_weight_one_seq + relative_weight_accumulated_seqs;

        let w1 = relative_weight_one_seq / sum;
        let wn = relative_weight_accumulated_seqs / sum;

        let mut combined_prof = Profile3::default();
        {
            let ap = m3.ap.as_ref().unwrap();
            align_two_profs_given_path(
                &prof_k,
                w1,
                &accumulated_prof,
                wn,
                &ap.subst_mx_letter,
                ap.gap_open,
                &path,
                &mut combined_prof,
            );
        }
        accumulated_prof = combined_prof;

        let mut combined_msa = MultiSequence::default();
        align_two_ms_as_given_path(&msa_k, &accumulated_msa, &path, &mut combined_msa);
        accumulated_msa = combined_msa;
    }
    m3.final_msa = Some(accumulated_msa);
    m3.final_msa_addr = Some(m3.final_msa.as_ref().unwrap() as *const MultiSequence as usize);
    m3.final_msa.as_ref().unwrap().clone()
}
