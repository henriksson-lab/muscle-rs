// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(clap::Parser, Debug, Clone, Default)]
#[command(
    name = "muscle",
    version,
    disable_help_flag = false,
    disable_version_flag = false,
    trailing_var_arg = false
)]
pub struct MuscleCli {
    #[arg(long)]
    pub align: Option<String>,
    #[arg(long)]
    pub upgma5: Option<String>,
    #[arg(long)]
    pub msastats: Option<String>,
    #[arg(long)]
    pub pprog: Option<String>,
    #[arg(long)]
    pub pprog2: Option<String>,
    #[arg(long)]
    pub pprogt: Option<String>,
    #[arg(long = "strip_gappy", alias = "strip-gappy")]
    pub strip_gappy: Option<String>,
    #[arg(long = "strip_gappy_cols", alias = "strip-gappy-cols")]
    pub strip_gappy_cols: Option<String>,
    #[arg(long = "strip_gappy_rows", alias = "strip-gappy-rows")]
    pub strip_gappy_rows: Option<String>,
    #[arg(long = "guide_tree_join_order", alias = "guide-tree-join-order")]
    pub guide_tree_join_order: Option<String>,
    #[arg(long)]
    pub eadistmx: Option<String>,
    #[arg(long = "eadistmx_msas", alias = "eadistmx-msas")]
    pub eadistmx_msas: Option<String>,
    #[arg(long = "tree_subset_nodes", alias = "tree-subset-nodes")]
    pub tree_subset_nodes: Option<String>,
    #[arg(long)]
    pub derep: Option<String>,
    #[arg(long)]
    pub consseq: Option<String>,
    #[arg(long)]
    pub super4: Option<String>,
    #[arg(long)]
    pub usorter: Option<String>,
    #[arg(long = "permute_tree", alias = "permute-tree")]
    pub permute_tree: Option<String>,
    #[arg(long = "divide_tree", alias = "divide-tree")]
    pub divide_tree: Option<String>,
    #[arg(long)]
    pub fa2efa: Option<String>,
    #[arg(long)]
    pub qscore: Option<String>,
    #[arg(long = "qscore_oldcode", alias = "qscore-oldcode")]
    pub qscore_oldcode: Option<String>,
    #[arg(long)]
    pub qscore2: Option<String>,
    #[arg(long)]
    pub qscoredir: Option<String>,
    #[arg(long)]
    pub eacluster: Option<String>,
    #[arg(long)]
    pub uclust: Option<String>,
    #[arg(long)]
    pub super5: Option<String>,
    #[arg(long)]
    pub super6: Option<String>,
    #[arg(long)]
    pub transaln: Option<String>,
    #[arg(long)]
    pub hmmdump: Option<String>,
    #[arg(long)]
    pub perturbhmm: Option<String>,
    #[arg(long)]
    pub resample: Option<String>,
    #[arg(long)]
    pub disperse: Option<String>,
    #[arg(long)]
    pub efastats: Option<String>,
    #[arg(long = "colscore_efa", alias = "colscore-efa")]
    pub colscore_efa: Option<String>,
    #[arg(long = "qscore_efa", alias = "qscore-efa")]
    pub qscore_efa: Option<String>,
    #[arg(long = "efa_bestconf", alias = "efa-bestconf")]
    pub efa_bestconf: Option<String>,
    #[arg(long = "efa_bestcols", alias = "efa-bestcols")]
    pub efa_bestcols: Option<String>,
    #[arg(long)]
    pub trimtoref: Option<String>,
    #[arg(long = "trimtoref_efa", alias = "trimtoref-efa")]
    pub trimtoref_efa: Option<String>,
    #[arg(long = "efa_explode", alias = "efa-explode")]
    pub efa_explode: Option<String>,
    #[arg(long)]
    pub relabel: Option<String>,
    #[arg(long)]
    pub addconfseq: Option<String>,
    #[arg(long)]
    pub labels2randomchaintree: Option<String>,
    #[arg(long)]
    pub maxcc: Option<String>,
    #[arg(long)]
    pub letterconf: Option<String>,
    #[arg(long = "letterconf_html", alias = "letterconf-html")]
    pub letterconf_html: Option<String>,
    #[arg(long = "masm_stats", alias = "masm-stats")]
    pub masm_stats: Option<String>,
    #[arg(long = "make_a2m", alias = "make-a2m")]
    pub make_a2m: Option<String>,
    #[arg(long = "make_a2m_refseq", alias = "make-a2m-refseq")]
    pub make_a2m_refseq: Option<String>,
    #[arg(long)]
    pub eesort: Option<String>,
    #[arg(long = "strip_anchors", alias = "strip-anchors")]
    pub strip_anchors: Option<String>,
    #[arg(long)]
    pub profalign: Option<String>,
    #[arg(long)]
    pub profseq: Option<String>,
    #[arg(long)]
    pub protdists: Option<String>,
    #[arg(long)]
    pub uclustpd: Option<String>,
    #[arg(long)]
    pub uclustpd2: Option<String>,
    #[arg(long)]
    pub searchpd: Option<String>,
    #[arg(long = "build_guide_tree", alias = "build-guide-tree")]
    pub build_guide_tree: Option<String>,
    #[arg(long = "test_malloc", alias = "test-malloc")]
    pub test_malloc: Option<String>,
    #[arg(long = "upgma5_msa", alias = "upgma5-msa")]
    pub upgma5_msa: Option<String>,
    #[arg(long = "pprog_tree", alias = "pprog-tree")]
    pub pprog_tree: Option<String>,
    #[arg(long)]
    pub test: Option<String>,
    #[arg(long = "build_prof3", alias = "build-prof3")]
    pub build_prof3: Option<String>,
    #[arg(long)]
    pub profprof3: Option<String>,
    #[arg(long)]
    pub muscle3: Option<String>,
    #[arg(long = "make_substmx", alias = "make-substmx")]
    pub make_substmx: Option<String>,
    #[arg(long)]
    pub bench: Option<String>,
    #[arg(long = "bench_blosums", alias = "bench-blosums")]
    pub bench_blosums: Option<String>,
    #[arg(long)]
    pub sweep: Option<String>,
    #[arg(long)]
    pub spatter: Option<String>,
    #[arg(long)]
    pub msaselfscore3: Option<String>,
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long)]
    pub m3ensemble: Option<String>,
    #[arg(long)]
    pub m3select: Option<String>,
    #[arg(long)]
    pub m3refine: Option<String>,
    #[arg(long)]
    pub addletterconfseq: Option<String>,
    #[arg(long = "core_blocks", alias = "core-blocks")]
    pub core_blocks: Option<String>,
    #[arg(long = "cmp_msa", alias = "cmp-msa")]
    pub cmp_msa: Option<String>,
    #[arg(long = "cmp_ref_msas", alias = "cmp-ref-msas")]
    pub cmp_ref_msas: Option<String>,
    #[arg(long = "squeeze_inserts", alias = "squeeze-inserts")]
    pub squeeze_inserts: Option<String>,
    #[arg(long = "newbench_selectpfams", alias = "newbench-selectpfams")]
    pub newbench_selectpfams: Option<String>,
    #[arg(long = "newbench_pfamgroups", alias = "newbench-pfamgroups")]
    pub newbench_pfamgroups: Option<String>,
    #[arg(long = "mustang_core", alias = "mustang-core")]
    pub mustang_core: Option<String>,
    #[arg(long = "test_mega", alias = "test-mega")]
    pub test_mega: Option<String>,
    #[arg(long)]
    pub shrub: Option<String>,
    #[arg(long)]
    pub super7: Option<String>,
    #[arg(long)]
    pub swdistmx: Option<String>,
    #[arg(long)]
    pub transalnref: Option<String>,
    #[arg(long = "mega_msas", alias = "mega-msas")]
    pub mega_msas: Option<String>,
    #[arg(long = "masm_train", alias = "masm-train")]
    pub masm_train: Option<String>,
    #[arg(long)]
    pub sw: Option<String>,
    #[arg(long)]
    pub swmasm: Option<String>,
    #[arg(long = "swmasm_seq", alias = "swmasm-seq")]
    pub swmasm_seq: Option<String>,
    #[arg(long)]
    pub mega2: Option<String>,
    #[arg(long = "test_sw_aa", alias = "test-sw-aa")]
    pub test_sw_aa: Option<String>,
    #[arg(long = "test_sw_mm", alias = "test-sw-mm")]
    pub test_sw_mm: Option<String>,
    #[arg(long)]
    pub swtest: Option<String>,
    #[arg(long)]
    pub swtestmm: Option<String>,
    #[arg(long)]
    pub swsimple2: Option<String>,

    #[arg(long)]
    pub output: Option<String>,
    #[arg(long)]
    pub input: Option<String>,
    #[arg(long)]
    pub input2: Option<String>,
    #[arg(long)]
    pub indir: Option<String>,
    #[arg(long)]
    pub outdir: Option<String>,
    #[arg(long)]
    pub db: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long = "ref")]
    pub ref_: Option<String>,
    #[arg(long = "max_gap_fract", alias = "max-gap-fract")]
    pub max_gap_fract: Option<f64>,
    #[arg(long = "max_gap_fract_row", alias = "max-gap-fract-row")]
    pub max_gap_fract_row: Option<f64>,
    #[arg(long)]
    pub minconf: Option<f64>,
    #[arg(long)]
    pub maxcols: Option<uint>,
    #[arg(long)]
    pub minea: Option<f32>,
    #[arg(long)]
    pub maxpd: Option<f64>,
    #[arg(long)]
    pub paircount: Option<uint>,
    #[arg(long)]
    pub threads: Option<uint>,
    #[arg(long)]
    pub linkage: Option<String>,
    #[arg(long)]
    pub tsvout: Option<String>,
    #[arg(long)]
    pub centroids: Option<String>,
    #[arg(long)]
    pub joins: Option<String>,
    #[arg(long)]
    pub guidetreein: Option<String>,
    #[arg(long)]
    pub guidetreeout: Option<String>,
    #[arg(long)]
    pub label: Option<String>,
    #[arg(long)]
    pub label1: Option<String>,
    #[arg(long)]
    pub label2: Option<String>,
    #[arg(long)]
    pub labels2: Option<String>,
    #[arg(long)]
    pub prefix: Option<String>,
    #[arg(long)]
    pub suffix: Option<String>,
    #[arg(long)]
    pub nodes: Option<String>,
    #[arg(long)]
    pub html: Option<String>,
    #[arg(long)]
    pub jalview: Option<String>,
    #[arg(long)]
    pub subtreeout: Option<String>,
    #[arg(long)]
    pub supertreeout: Option<String>,
    #[arg(long = "min_core_block_cols", alias = "min-core-block-cols")]
    pub min_core_block_cols: Option<uint>,
    #[arg(long = "min_core_block_seqs", alias = "min-core-block-seqs")]
    pub min_core_block_seqs: Option<uint>,
    #[arg(long)]
    pub minpctid: Option<uint>,
    #[arg(long)]
    pub maxpctid: Option<uint>,
    #[arg(long)]
    pub replicates: Option<uint>,
    #[arg(long)]
    pub n: Option<uint>,
    #[arg(long)]
    pub gridspec: Option<String>,
    #[arg(long)]
    pub refdir: Option<String>,
    #[arg(long)]
    pub fev: Option<String>,
    #[arg(long)]
    pub blosumpct: Option<uint>,
    #[arg(long)]
    pub blosumparamset: Option<uint>,
    #[arg(long)]
    pub substmx: Option<String>,
    #[arg(long)]
    pub kmerdist: Option<String>,
    #[arg(long)]
    pub treeiters: Option<uint>,
    #[arg(long = "warmup_pct", alias = "warmup-pct")]
    pub warmup_pct: Option<uint>,
    #[arg(long)]
    pub maxiters: Option<uint>,
    #[arg(long)]
    pub maxfailiters: Option<uint>,
    #[arg(long)]
    pub triesperiter: Option<uint>,
    #[arg(long)]
    pub shrink: Option<f32>,
    #[arg(long)]
    pub spatterspec: Option<String>,
    #[arg(long)]
    pub output1: Option<String>,
    #[arg(long)]
    pub output2: Option<String>,
    #[arg(long)]
    pub output3: Option<String>,
    #[arg(long)]
    pub output4: Option<String>,
    #[arg(long)]
    pub testdir: Option<String>,
    #[arg(long)]
    pub gapopen: Option<f32>,
    #[arg(long)]
    pub center: Option<f32>,
    #[arg(long)]
    pub gapext: Option<f32>,
    #[arg(long)]
    pub termgapopen: Option<f32>,
    #[arg(long)]
    pub termgapext: Option<f32>,
    #[arg(long = "s_is", alias = "s-is")]
    pub s_is: Option<f32>,
    #[arg(long = "s_il", alias = "s-il")]
    pub s_il: Option<f32>,
    #[arg(long = "m_is", alias = "m-is")]
    pub m_is: Option<f32>,
    #[arg(long = "m_il", alias = "m-il")]
    pub m_il: Option<f32>,
    #[arg(long = "is_is", alias = "is-is")]
    pub is_is: Option<f32>,
    #[arg(long = "il_il", alias = "il-il")]
    pub il_il: Option<f32>,
    #[arg(long)]
    pub perturb: Option<uint>,
    /// C++ `opt(randseed)`: seeds the MWC RNG used by `randu32` (uclustpd,
    /// get_pairs, etc.). Does not affect libc rand() used by MPC refinement.
    #[arg(long)]
    pub randseed: Option<uint>,
    /// C++ `opt(hmmin)`: path to a HMM params file written by `-hmmdump`,
    /// loaded in place of the built-in defaults (setprobconsparams.cpp:13-19).
    #[arg(long)]
    pub hmmin: Option<String>,
    /// C++ `opt(hmmout)`: write the (possibly perturbed) HMM params after
    /// `init_probcons` and before any alignment (setprobconsparams.cpp:37-38).
    #[arg(long)]
    pub hmmout: Option<String>,
    /// C++ `opt(log)`: path to a text file that captures `Log(...)` output
    /// (main.cpp:34 calls `SetLogFileName(opt(log))`).
    #[arg(long)]
    pub log: Option<String>,
    /// C++ `opt(tree_order)`: select MSA output ordering by guide-tree traversal
    /// (mpcflat.cpp:341). This is already the default; we just need to accept
    /// the flag without erroring, mirroring C++ option handling.
    #[arg(long = "tree_order", alias = "tree-order")]
    pub tree_order: bool,
    /// C++ `opt(randomchaintree)`: MPCFlat builds a random caterpillar guide
    /// tree (randomchaintree.cpp) instead of UPGMA on EA distances. Consulted
    /// at mpcflat.cpp:185.
    #[arg(long = "randomchaintree", alias = "randomchaintree")]
    pub randomchaintree: bool,
    #[arg(long)]
    pub consiters: Option<uint>,
    #[arg(long)]
    pub refineiters: Option<uint>,
    #[arg(long)]
    pub minsuper: Option<uint>,
    #[arg(long)]
    pub perm: Option<String>,
    #[arg(long = "distmxin", alias = "distmx-in")]
    pub distmxin: Option<String>,
    #[arg(long = "shrub_size", alias = "shrub-size")]
    pub shrub_size: Option<uint>,
    #[arg(long = "super5_minea1", alias = "super5-minea1")]
    pub super5_minea1: Option<f32>,
    #[arg(long = "super4_minea1", alias = "super4-minea1")]
    pub super4_minea1: Option<f32>,
    #[arg(long = "super4_minea2", alias = "super4-minea2")]
    pub super4_minea2: Option<f32>,
    #[arg(long = "super6_maxpd1", alias = "super6-maxpd1")]
    pub super6_maxpd1: Option<f64>,
    #[arg(long)]
    pub nt: bool,
    #[arg(long)]
    pub amino: bool,
    #[arg(long)]
    pub bysequence: bool,
    #[arg(long)]
    pub confseq1: bool,
    #[arg(long)]
    pub right: bool,
    #[arg(long)]
    pub stratified: bool,
    #[arg(long)]
    pub diversified: bool,
    #[arg(long = "input_order", alias = "input-order")]
    pub input_order: bool,
    #[arg(long = "muscle3_randomorder", alias = "muscle3-randomorder")]
    pub muscle3_randomorder: bool,
    #[arg(long)]
    pub mega: bool,
    #[arg(long)]
    pub scaledist: bool,
    #[arg(long)]
    pub eadist: bool,
    #[arg(long)]
    pub reseek: bool,
    #[arg(long)]
    pub quiet: bool,
    /// `-fa2efa` option: replace each MSA name with its file's basename.
    #[arg(long)]
    pub basename: bool,
    /// `-fa2efa` option: append `.N` to each MSA name based on input order.
    #[arg(long)]
    pub intsuffix: bool,
}

/// Format a value with the same convention C++ uses for `%.3g` — 3 significant
/// digits, drop trailing zeros and a trailing decimal point. Used to match the
/// C++ `ProgressLog("%s Q=%.3g, TC=%.3g\n", ...)` output verbatim.
fn format_percent_g3(v: f64) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    let s = format!("{v:.3}");
    // Drop trailing zeros after the decimal, then a trailing decimal point.
    if !s.contains('.') {
        return s;
    }
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// CLI entry point: parses MUSCLE-style command-line arguments and dispatches to the chosen subcommand.
#[track_caller]
pub fn main() {
    let mut normalized_args = Vec::new();
    for (i, arg) in std::env::args().enumerate() {
        if i > 0
            && arg.starts_with('-')
            && !arg.starts_with("--")
            && arg.len() > 2
            && arg.as_bytes()[1].is_ascii_alphabetic()
        {
            normalized_args.push(format!("-{arg}"));
        } else {
            normalized_args.push(arg);
        }
    }
    let cli = match <MuscleCli as clap::Parser>::try_parse_from(normalized_args) {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };
    if !cli.quiet {
        eprint!("{}", print_banner());
    }
    // C++ `optset_perturb && opt(perturb)` is a process-global flag consulted
    // by setprobconsparams.cpp:InitProbcons. Publish it once at startup so
    // every command path picks up the same perturbation as C++.
    set_perturb_seed(cli.perturb);
    set_guidetreeout_path(cli.guidetreeout.clone());
    set_hmmin_path(cli.hmmin.clone());
    set_hmmout_path(cli.hmmout.clone());
    set_randomchaintree_enabled(cli.randomchaintree);
    if let Some(path) = cli.log.as_deref() {
        // Mirror C++ main.cpp:34 `SetLogFileName(opt(log))`.
        set_log_file_name(path);
    }
    if let Some(seed) = cli.randseed {
        // Mirror C++ myutils.cpp:2244-2245 InitRand: when -randseed is set,
        // seed the MWC generator before any randu32 call.
        reset_rand(seed);
    }
    let cmd_count = [
        cli.align.is_some(),
        cli.upgma5.is_some(),
        cli.msastats.is_some(),
        cli.pprog.is_some(),
        cli.pprog2.is_some(),
        cli.pprogt.is_some(),
        cli.strip_gappy.is_some(),
        cli.strip_gappy_cols.is_some(),
        cli.strip_gappy_rows.is_some(),
        cli.guide_tree_join_order.is_some(),
        cli.eadistmx.is_some(),
        cli.eadistmx_msas.is_some(),
        cli.tree_subset_nodes.is_some(),
        cli.consseq.is_some(),
        cli.super4.is_some(),
        cli.usorter.is_some(),
        cli.permute_tree.is_some(),
        cli.divide_tree.is_some(),
        cli.qscore.is_some(),
        cli.qscore_oldcode.is_some(),
        cli.qscore2.is_some(),
        cli.qscoredir.is_some(),
        cli.eacluster.is_some(),
        cli.derep.is_some(),
        cli.uclust.is_some(),
        cli.super5.is_some(),
        cli.super6.is_some(),
        cli.transaln.is_some(),
        cli.hmmdump.is_some(),
        cli.perturbhmm.is_some(),
        cli.resample.is_some(),
        cli.disperse.is_some(),
        cli.efastats.is_some(),
        cli.fa2efa.is_some(),
        cli.colscore_efa.is_some(),
        cli.qscore_efa.is_some(),
        cli.efa_bestconf.is_some(),
        cli.efa_bestcols.is_some(),
        cli.trimtoref.is_some(),
        cli.trimtoref_efa.is_some(),
        cli.efa_explode.is_some(),
        cli.relabel.is_some(),
        cli.addconfseq.is_some(),
        cli.labels2randomchaintree.is_some(),
        cli.maxcc.is_some(),
        cli.letterconf.is_some(),
        cli.letterconf_html.is_some(),
        cli.make_a2m.is_some(),
        cli.make_a2m_refseq.is_some(),
        cli.eesort.is_some(),
        cli.strip_anchors.is_some(),
        cli.profalign.is_some(),
        cli.profseq.is_some(),
        cli.protdists.is_some(),
        cli.uclustpd.is_some(),
        cli.uclustpd2.is_some(),
        cli.searchpd.is_some(),
        cli.build_guide_tree.is_some(),
        cli.test_malloc.is_some(),
        cli.upgma5_msa.is_some(),
        cli.pprog_tree.is_some(),
        cli.test.is_some(),
        cli.build_prof3.is_some(),
        cli.profprof3.is_some(),
        cli.muscle3.is_some(),
        cli.make_substmx.is_some(),
        cli.bench.is_some(),
        cli.bench_blosums.is_some(),
        cli.sweep.is_some(),
        cli.spatter.is_some(),
        cli.msaselfscore3.is_some(),
        cli.batch.is_some(),
        cli.m3ensemble.is_some(),
        cli.m3select.is_some(),
        cli.m3refine.is_some(),
        cli.addletterconfseq.is_some(),
        cli.cmp_msa.is_some(),
        cli.cmp_ref_msas.is_some(),
        cli.squeeze_inserts.is_some(),
        cli.newbench_selectpfams.is_some(),
        cli.newbench_pfamgroups.is_some(),
        cli.mustang_core.is_some(),
        cli.test_mega.is_some(),
        cli.shrub.is_some(),
        cli.super7.is_some(),
        cli.swdistmx.is_some(),
        cli.transalnref.is_some(),
        cli.mega_msas.is_some(),
        cli.masm_train.is_some(),
        cli.sw.is_some(),
        cli.swmasm.is_some(),
        cli.swmasm_seq.is_some(),
        cli.masm_stats.is_some(),
        cli.mega2.is_some(),
        cli.test_sw_aa.is_some(),
        cli.test_sw_mm.is_some(),
        cli.swtest.is_some(),
        cli.swtestmm.is_some(),
        cli.swsimple2.is_some(),
        cli.core_blocks.is_some(),
    ]
    .iter()
    .filter(|set| **set)
    .count();
    if cmd_count > 1 {
        die("More than one command specified");
    }
    let out = if let Some(input_file_name) = &cli.msastats {
        cmd_msastats(input_file_name, cli.max_gap_fract)
    } else if let Some(input_file_name) = &cli.derep {
        cmd_derep(input_file_name, cli.output.as_deref().unwrap_or(""));
        String::new()
    } else if let Some(input_file_name) = &cli.consseq {
        cmd_consseq(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.label.as_deref(),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.fa2efa {
        cmd_fa2efa(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.basename,
            cli.intsuffix,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.qscore {
        let (q, tc) = cmd_qscore(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.bysequence,
        );
        format!(
            "{input_file_name} Q={}, TC={}\n",
            format_percent_g3(q),
            format_percent_g3(tc)
        )
    } else if let Some(input_file_name) = &cli.qscore_oldcode {
        let (_q, _tc, log) = cmd_qscore_oldcode(input_file_name, cli.ref_.as_deref().unwrap_or(""));
        log
    } else if let Some(input_file_name) = &cli.qscore2 {
        cmd_qscore2(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(1.0),
        )
    } else if let Some(input_file_name) = &cli.qscoredir {
        // C++ qscoredir writes only to `-output` (CreateStdioFile("") returns
        // NULL so a missing -output silently drops the data). We match that
        // when -output is set, and otherwise fall back to stdout — a strict
        // improvement on the C++ no-op behaviour.
        let result = cmd_qscoredir(
            input_file_name,
            cli.testdir.as_deref().unwrap_or(""),
            cli.refdir.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
        );
        if cli.output.is_some() {
            String::new()
        } else {
            result
        }
    } else if let Some(input_file_name) = &cli.qscore_efa {
        cmd_qscore_efa(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(1.0),
        )
    } else if let Some(input_file_name) = &cli.efastats {
        cmd_efastats(
            input_file_name,
            cli.max_gap_fract.unwrap_or(0.5),
            cli.ref_.as_deref(),
        )
    } else if let Some(input_file_name) = &cli.disperse {
        cmd_disperse(input_file_name, cli.max_gap_fract.unwrap_or(0.5))
    } else if let Some(input_file_name) = &cli.efa_bestconf {
        let (out, _best_total, _best_median) =
            cmd_efa_bestconf(input_file_name, cli.output.as_deref().unwrap_or(""));
        out
    } else if let Some(input_file_name) = &cli.efa_bestcols {
        let _msa = cmd_efa_bestcols(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.minconf.unwrap_or(1.0),
            cli.max_gap_fract.unwrap_or(0.5),
            cli.maxcols.unwrap_or(uint::MAX),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.efa_explode {
        let _file_names = cmd_efa_explode(
            input_file_name,
            cli.prefix.as_deref(),
            cli.suffix.as_deref(),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.addconfseq {
        cmd_addconfseq(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.ref_.as_deref(),
            cli.label.as_deref(),
            cli.confseq1,
        )
    } else if let Some(input_file_name) = &cli.addletterconfseq {
        cmd_addletterconfseq(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(1.0),
        )
    } else if let Some(input_file_name) = &cli.letterconf {
        let _msa = cmd_letterconf(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.html.as_deref().unwrap_or(""),
            cli.jalview.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(1.0),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.letterconf_html {
        cmd_letterconf_html(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.masm_train {
        cmd_masm_train(
            input_file_name,
            cli.input.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.label.as_deref(),
            |mega_file_name| mega_from_file(mega_file_name),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.masm_stats {
        cmd_masm_stats(input_file_name)
    } else if let Some(input_file_name) = &cli.make_a2m {
        cmd_make_a2m(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
            true,
        )
    } else if let Some(input_file_name) = &cli.make_a2m_refseq {
        cmd_make_a2m_refseq(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.label.as_deref(),
            true,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.core_blocks {
        cmd_core_blocks(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.min_core_block_cols.unwrap_or(2),
            cli.min_core_block_seqs.unwrap_or(2),
        )
    } else if let Some(input_file_name) = &cli.make_substmx {
        cmd_make_substmx(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.label.as_deref(),
            cli.minpctid,
            cli.maxpctid,
        )
    } else if let Some(input_file_name) = &cli.squeeze_inserts {
        let _out_msa = cmd_squeeze_inserts(input_file_name, cli.output.as_deref().unwrap_or(""));
        String::new()
    } else if let Some(input_file_name) = &cli.colscore_efa {
        // C++ colscore_efa writes only to `-output`; suppress stdout when it
        // is set so we don't double-emit the same content.
        let result = cmd_colscore_efa(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
        );
        if cli.output.is_some() {
            String::new()
        } else {
            result
        }
    } else if let Some(input_file_name) = &cli.cmp_ref_msas {
        let (_qs, log) = cmd_cmp_ref_msas(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(1.0),
        );
        let mut out = log.lines().next().unwrap_or("").to_string();
        if !out.is_empty() {
            out.push('\n');
        }
        out
    } else if let Some(input_file_name) = &cli.cmp_msa {
        cmd_cmp_msa(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.eadistmx {
        if cli.output.as_deref().unwrap_or("").is_empty() {
            die("Must set -output");
        }
        let _dist_mx = cmd_eadistmx(
            input_file_name,
            cli.output.as_deref().unwrap(),
            |label1, label2, path| {
                let (ea, aln_path) = align_pair_flat(label1, label2);
                *path = aln_path;
                (ea, None)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.eadistmx_msas {
        let _pp = cmd_eadistmx_msas(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.paircount,
            |label, msa1, msa2, pair_count, path| {
                p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.align {
        let perm = cli.perm.as_deref().map(str_to_treeperm);
        let (_mpc, _files, log) = cmd_align(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.minsuper,
            cli.consiters,
            cli.refineiters,
            cli.stratified,
            cli.diversified,
            cli.replicates,
            cli.perturb,
            perm,
            |hp| {
                hmm_params_cmd_line_update(
                    hp, cli.s_is, cli.s_il, cli.m_is, cli.m_il, cli.is_is, cli.il_il,
                )
            },
            |mpc, input_seqs| {
                mpc_flat_run(
                    mpc,
                    input_seqs,
                    cli.input_order,
                    |mpc| mpc_flat_calc_posteriors(mpc),
                    |mpc| {
                        mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                            upgma5_run_l75(u, "biased", guide_tree)
                        })
                    },
                    |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                    |mpc| {
                        mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                            mpc_flat_align_alns(mpc, msa1, msa2).0
                        })
                    },
                    |mpc| {
                        mpc_flat_refine(
                            mpc,
                            || libc_rand(),
                            |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
                        )
                    },
                );
                mpc.msa
                    .as_ref()
                    .expect("cmd_align missing final MSA")
                    .clone()
            },
            |input_seq_count| {
                let (_s5, files, log) = cmd_super5(
                    input_file_name,
                    cli.output.as_deref().unwrap_or(""),
                    None,
                    perm,
                    cli.perturb,
                    cli.super5_minea1,
                    cli.input_order,
                    cli.mega,
                    cli.diversified,
                    cli.replicates.is_some(),
                    cli.stratified,
                    |s5, input_seqs, perm, input_order| {
                        super5_run(
                            s5,
                            input_seqs,
                            perm,
                            input_order,
                            |d, input_seqs| {
                                derep_run(d, input_seqs, true);
                                let mut unique = MultiSequence::default();
                                derep_get_unique_seqs(d, &mut unique);
                                unique
                            },
                            |u, unique_seqs, min_ea| {
                                let _log = u_clust_run(u, unique_seqs, min_ea, |label1, label2| {
                                    align_pair_flat(label1, label2)
                                });
                                let mut centroids = MultiSequence::default();
                                u_clust_get_centroid_seqs(u, &mut centroids);
                                centroids
                            },
                            |s4, input_seqs, perm| {
                                let _log = super4_run(
                                    s4,
                                    input_seqs,
                                    perm,
                                    |s4| {
                                        super4_set_opts(
                                            s4,
                                            cli.paircount,
                                            None,
                                            None,
                                            cli.consiters,
                                            cli.refineiters,
                                        )
                                    },
                                    |ec, big_mfa, min_ea| {
                                        ea_cluster_run(ec, big_mfa, min_ea, |label1, label2| {
                                            ea_cluster_align_seq_pair(label1, label2)
                                        });
                                        ec.cluster_mfas.clone()
                                    },
                                    |mpc, cluster_mfa| {
                                        mpc_flat_run(
                                            mpc,
                                            cluster_mfa,
                                            cli.input_order,
                                            |mpc| mpc_flat_calc_posteriors(mpc),
                                            |mpc| {
                                                mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                                                    upgma5_run_l75(u, "biased", guide_tree)
                                                })
                                            },
                                            |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                                            |mpc| {
                                                mpc_flat_progressive_align(
                                                    mpc,
                                                    |mpc, msa1, msa2| {
                                                        mpc_flat_align_alns(mpc, msa1, msa2).0
                                                    },
                                                )
                                            },
                                            |mpc| {
                                                mpc_flat_refine(
                                                    mpc,
                                                    || libc_rand(),
                                                    |mpc, msa1, msa2| {
                                                        mpc_flat_align_alns(mpc, msa1, msa2).0
                                                    },
                                                )
                                            },
                                        );
                                        mpc.msa
                                            .as_ref()
                                            .expect("super5 minsuper missing cluster MSA")
                                            .clone()
                                    },
                                    |consensus_seqs| {
                                        calc_ea_dist_mx(
                                            consensus_seqs,
                                            None,
                                            |label1, label2, path| {
                                                let (ea, aln_path) =
                                                    align_pair_flat(label1, label2);
                                                *path = aln_path;
                                                (ea, None)
                                            },
                                        )
                                        .0
                                    },
                                    |guide_tree, labels, dist_mx| {
                                        let mut u = UPGMA5::default();
                                        upgma5_init(&mut u, labels, dist_mx);
                                        upgma5_run_l75(&mut u, "biased", guide_tree);
                                    },
                                    |none, abc, acb, bca, labels_a, labels_b, labels_c| {
                                        permute_tree(
                                            none, abc, acb, bca, labels_a, labels_b, labels_c,
                                        )
                                    },
                                    |pp, tree| {
                                        p_prog_run_guide_tree(pp, tree, |pp, index1, index2| {
                                            p_prog_align_and_join(
                                                pp,
                                                index1,
                                                index2,
                                                |label, msa1, msa2, pair_count, path| {
                                                    p_prog_align_ms_as_flat(
                                                        label, msa1, msa2, pair_count, path,
                                                    )
                                                },
                                            )
                                        });
                                        p_prog_get_final_msa(pp).clone()
                                    },
                                );
                            },
                        );
                        String::new()
                    },
                );
                let _ = input_seq_count;
                (files, log)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.profalign {
        let (_hp, _msa) = cmd_profalign(
            input_file_name,
            cli.input2.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.perturb,
            |hp| {
                hmm_params_cmd_line_update(
                    hp, cli.s_is, cli.s_il, cli.m_is, cli.m_il, cli.is_is, cli.il_il,
                )
            },
            |mpc, pair_index| mpc_flat_calc_posterior(mpc, pair_index),
            |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.profseq {
        let (_hp, paths) = cmd_profseq(
            input_file_name,
            cli.input2.as_deref().unwrap_or(""),
            cli.perturb,
            |hp| {
                hmm_params_cmd_line_update(
                    hp, cli.s_is, cli.s_il, cli.m_is, cli.m_il, cli.is_is, cli.il_il,
                )
            },
            |mpc, pair_index| mpc_flat_calc_posterior(mpc, pair_index),
        );
        if paths.is_empty() {
            String::new()
        } else {
            format!("{}\n", paths.join("\n"))
        }
    } else if let Some(input_file_name) = &cli.build_prof3 {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            None,
            cli.perturb,
            cli.linkage.as_deref(),
            None,
            cli.treeiters,
        );
        let (_prof, log, _tsv) = cmd_build_prof3(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            &ap.subst_mx_letter,
            ap.gap_open,
        );
        log
    } else if let Some(input_file_name) = &cli.profprof3 {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            None,
            cli.perturb,
            cli.linkage.as_deref(),
            None,
            cli.treeiters,
        );
        let (_msa, _prof1, _prof2, _prof12_msa, _prof12_path, _diff_count, log) = cmd_profprof3(
            input_file_name,
            cli.input2.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.output1.as_deref().unwrap_or(""),
            cli.output2.as_deref().unwrap_or(""),
            cli.output3.as_deref().unwrap_or(""),
            cli.output4.as_deref().unwrap_or(""),
            &ap,
            |cm, prof1, prof2| nw_small3(cm, prof1, prof2),
            |prof1, w1, prof2, w2, subst_mx_letter, gap_open, path| {
                align_two_profs_given_path(prof1, w1, prof2, w2, subst_mx_letter, gap_open, path)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.protdists {
        cmd_protdists(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            |seqi, li, seqj, lj| {
                let mut mem = XDPMem::default();
                let mut pi = PathInfo::default();
                viterbi_fast_mem(&mut mem, seqi, li, seqj, lj, &mut pi);
                pi
            },
            |row_x, row_y, col_count| {
                get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
            },
        )
    } else if let Some(input_file_name) = &cli.searchpd {
        if cli.output.is_some() {
            die("Use -tsvout not -output");
        }
        cmd_searchpd(
            input_file_name,
            cli.db.as_deref().unwrap_or_else(|| {
                die("Must set -db");
            }),
            cli.maxpd.unwrap_or_else(|| {
                die("Must set -maxpd");
            }),
            cli.tsvout.as_deref().unwrap_or(""),
            |seqi, li, seqj, lj| {
                let mut mem = XDPMem::default();
                let mut pi = PathInfo::default();
                viterbi_fast_mem(&mut mem, seqi, li, seqj, lj, &mut pi);
                pi
            },
            |row_x, row_y, col_count| {
                get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
            },
        )
    } else if let Some(input_file_name) = &cli.eacluster {
        let (_ec, _files) = cmd_eacluster(
            input_file_name,
            cli.minea.unwrap_or(0.9),
            cli.output.as_deref().unwrap_or("cluster%.afa"),
            |label1, label2| ea_cluster_align_seq_pair(label1, label2),
        );
        String::new()
    } else if let Some(query_file_name) = &cli.eesort {
        let (_eas, _order, _tsv, _fa) = cmd_eesort(
            query_file_name,
            cli.db.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.tsvout.as_deref().unwrap_or(""),
            |label1, label2, path| {
                let (ea, aln_path) = align_pair_flat(label1, label2);
                *path = aln_path;
                f64::from(ea)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.maxcc {
        if cli.output.as_deref().unwrap_or("").is_empty() {
            die("Must set -output");
        }
        let (_best_msa, _best_msa_index, _avg_conf) =
            cmd_maxcc(input_file_name, cli.output.as_deref().unwrap());
        String::new()
    } else if let Some(input_file_name) = &cli.mustang_core {
        cmd_mustang_core(input_file_name, cli.output.as_deref().unwrap_or(""))
    } else if let Some(input_file_name) = &cli.labels2randomchaintree {
        let _tree =
            cmd_labels2randomchaintree(input_file_name, cli.output.as_deref().unwrap_or(""));
        String::new()
    } else if let Some(input_file_name) = &cli.relabel {
        cmd_relabel(
            input_file_name,
            cli.labels2.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
        )
    } else if let Some(input_file_name) = &cli.resample {
        if cli.output.as_deref().unwrap_or("").is_empty() {
            die("Must set -output");
        }
        let _reps = cmd_resample(
            input_file_name,
            cli.output.as_deref().unwrap(),
            cli.max_gap_fract.unwrap_or(0.5),
            cli.minconf.unwrap_or(0.5),
            cli.replicates.unwrap_or(100),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.perturbhmm {
        assert!(!cli.nt || !cli.amino);
        let iters = str_to_uint_l1278(input_file_name, false);
        cmd_perturbhmm(iters, cli.nt)
    } else if let Some(input_file_name) = &cli.sw {
        cmd_sw(input_file_name)
    } else if let Some(input_file_name) = &cli.swmasm {
        cmd_swmasm(
            input_file_name,
            cli.query.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            |mega_file_name| mega_from_file(mega_file_name),
        )
    } else if let Some(input_file_name) = &cli.swmasm_seq {
        cmd_swmasm_seq(
            input_file_name,
            cli.input.as_deref().unwrap_or(""),
            cli.input2.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            |mega_file_name| mega_from_file(mega_file_name),
        )
    } else if let Some(input_file_name) = &cli.newbench_pfamgroups {
        let out_base = cli.output.as_deref().unwrap_or(".");
        let out_fa = format!("{out_base}/fa");
        let out_tsv = format!("{out_base}/tsv");
        cmd_newbench_pfamgroups(
            input_file_name,
            cli.input.as_deref().unwrap_or(""),
            &out_fa,
            &out_tsv,
            10,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.newbench_selectpfams {
        cmd_newbench_selectpfams(input_file_name, cli.output.as_deref().unwrap_or(""))
    } else if let Some(input_file_name) = &cli.msaselfscore3 {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            cli.blosumparamset,
            cli.perturb,
            cli.linkage.as_deref(),
            cli.kmerdist.as_deref(),
            cli.treeiters,
        );
        let (_prof, _self_score, log) =
            cmd_msaselfscore3(input_file_name, &ap.subst_mx_letter, ap.gap_open);
        log
    } else if let Some(input_file_name) = &cli.muscle3 {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            cli.blosumparamset,
            cli.perturb,
            cli.linkage.as_deref(),
            cli.kmerdist.as_deref(),
            cli.treeiters,
        );
        let (_msa, _tree) = cmd_muscle3(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.guidetreeout.as_deref(),
            cli.muscle3_randomorder,
            &ap,
            |ap, input_seqs| {
                set_global_input_ms(input_seqs);
                let mut m3 = Muscle3::default();
                let final_msa = muscle3_run(
                    &mut m3,
                    ap,
                    input_seqs,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input_seqs, input_seq_weights, guide_tree| {
                        p_prog3_run(
                            pp,
                            input_seqs,
                            input_seq_weights,
                            guide_tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                );
                (final_msa, m3.guide_tree)
            },
            |ap, input_seqs| {
                let mut m3 = Muscle3 {
                    ap: Some(ap.clone()),
                    ..Muscle3::default()
                };
                muscle3_run_ro(
                    &mut m3,
                    ap,
                    input_seqs,
                    |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                    |prof_a,
                     weight_a,
                     prof_b,
                     weight_b,
                     subst_mx_letter,
                     gap_open,
                     path,
                     prof_out| {
                        *prof_out = align_two_profs_given_path(
                            prof_a,
                            weight_a,
                            prof_b,
                            weight_b,
                            subst_mx_letter,
                            gap_open,
                            path,
                        );
                    },
                )
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.batch {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            cli.blosumparamset,
            cli.perturb,
            cli.linkage.as_deref(),
            cli.kmerdist.as_deref(),
            cli.treeiters,
        );
        let _files = cmd_batch(
            input_file_name,
            cli.indir.as_deref().unwrap_or(""),
            cli.outdir.as_deref().unwrap_or(""),
            &ap,
            |m3, ap, input_seqs| {
                set_global_input_ms(input_seqs);
                muscle3_run(
                    m3,
                    ap,
                    input_seqs,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input_seqs, input_seq_weights, guide_tree| {
                        p_prog3_run(
                            pp,
                            input_seqs,
                            input_seq_weights,
                            guide_tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                )
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.bench {
        let mut ap = M3AlnParams::default();
        let _ = m3_aln_params_set_from_cmd_line(
            &mut ap,
            false,
            false,
            cli.substmx.as_deref(),
            cli.gapopen,
            cli.center,
            cli.blosumpct,
            cli.blosumparamset,
            cli.perturb,
            cli.linkage.as_deref(),
            cli.kmerdist.as_deref(),
            cli.treeiters,
        );
        let (_bench, log) = cmd_bench(
            input_file_name,
            cli.refdir.as_deref().unwrap_or(""),
            cli.tsvout.as_deref().unwrap_or(""),
            &ap,
            |ap, ref_name, input, ref_msa, _q2| {
                set_global_input_ms(input);
                let mut m3 = Muscle3::default();
                let final_msa = muscle3_run(
                    &mut m3,
                    ap,
                    input,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input_seqs, input_seq_weights, guide_tree| {
                        p_prog3_run(
                            pp,
                            input_seqs,
                            input_seq_weights,
                            guide_tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                );
                let mut qs = QScorer::default();
                let _ = q_scorer_run_l346(&mut qs, ref_name, &final_msa, ref_msa, false);
                (qs.q as f64, qs.tc as f64)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.bench_blosums {
        let (_bench, log, _tsv) = cmd_bench_blosums(
            input_file_name,
            cli.refdir.as_deref().unwrap_or(""),
            cli.tsvout.as_deref().unwrap_or(""),
            |ap, ref_name, input, ref_msa, _q2| {
                set_global_input_ms(input);
                let mut m3 = Muscle3::default();
                let final_msa = muscle3_run(
                    &mut m3,
                    ap,
                    input,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input_seqs, input_seq_weights, guide_tree| {
                        p_prog3_run(
                            pp,
                            input_seqs,
                            input_seq_weights,
                            guide_tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                );
                let mut qs = QScorer::default();
                let _ = q_scorer_run_l346(&mut qs, ref_name, &final_msa, ref_msa, false);
                (qs.q as f64, qs.tc as f64)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.m3ensemble {
        assert!(cli.n.is_none());
        if cli.output.as_deref().unwrap_or("").is_empty() {
            die("Must set -output");
        }
        cmd_m3ensemble(
            input_file_name,
            cli.output.as_deref().unwrap(),
            cli.replicates,
        )
    } else if let Some(input_file_name) = &cli.m3select {
        cmd_m3select(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.replicates,
        )
    } else if let Some(input_file_name) = &cli.m3refine {
        let (_msa, _labels, _weights, _tree, _ap, _refined, log) = cmd_m3refine(
            input_file_name,
            |u, tree| upgma5_run_l75(u, "biased", tree),
            |ap| {
                let _ = m3_aln_params_set_from_cmd_line(
                    ap,
                    false,
                    false,
                    cli.substmx.as_deref(),
                    cli.gapopen,
                    cli.center,
                    cli.blosumpct,
                    cli.blosumparamset,
                    cli.perturb,
                    cli.linkage.as_deref(),
                    cli.kmerdist.as_deref(),
                    cli.treeiters,
                );
            },
            |msa, ap, weights, refined_msa| {
                m3_refine(msa, ap, weights, refined_msa, |cm, prof_a, prof_b| {
                    nw_small3(cm, prof_a, prof_b).1
                })
            },
        );
        log
    } else if let Some(input_file_name) = &cli.swdistmx {
        // C++ uses `opt(guidetreeout)` (swdistmx.cpp:135), not `-output`.
        cmd_swdistmx(input_file_name, cli.guidetreeout.as_deref().unwrap_or(""));
        String::new()
    } else if let Some(input_file_name) = &cli.uclustpd {
        let max_pd = cli.maxpd.unwrap_or_else(|| {
            die("Must set -maxpd");
        });
        if cli.output.is_some() {
            die("Use -tsvout not -output");
        }
        let (_ud, run_log, stats) = cmd_uclustpd(
            input_file_name,
            cli.tsvout.as_deref().unwrap_or(""),
            max_pd,
            cli.threads.unwrap_or(1),
            |uc, seq_indexi, seq_indexj| {
                u_clust_pd_get_prot_dist_pair(
                    uc,
                    seq_indexi,
                    seq_indexj,
                    None,
                    |seqi, li, seqj, lj| {
                        let mut pi = PathInfo::default();
                        with_dp_mem_l17(|mem| {
                            viterbi_fast_mem(mem, seqi, li, seqj, lj, &mut pi);
                        });
                        pi
                    },
                    |row_x, row_y, col_count| {
                        get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
                    },
                )
            },
        );
        format!("{run_log}{stats}")
    } else if let Some(input_file_name) = &cli.uclustpd2 {
        let max_pd = cli.maxpd.unwrap_or_else(|| {
            die("Must set -maxpd");
        });
        if cli.output.is_some() || cli.tsvout.is_some() {
            die("Use -output1/2");
        }
        let (_ud, _selected, _reordered, run_log1, stats1, run_log2, stats2) = cmd_uclustpd2(
            input_file_name,
            cli.output1.as_deref().unwrap_or(""),
            cli.output2.as_deref().unwrap_or(""),
            cli.centroids.as_deref().unwrap_or(""),
            max_pd,
            cli.threads.unwrap_or(1),
            |uc, seq_indexi, seq_indexj| {
                u_clust_pd_get_prot_dist_pair(
                    uc,
                    seq_indexi,
                    seq_indexj,
                    None,
                    |seqi, li, seqj, lj| {
                        let mut pi = PathInfo::default();
                        with_dp_mem_l17(|mem| {
                            viterbi_fast_mem(mem, seqi, li, seqj, lj, &mut pi);
                        });
                        pi
                    },
                    |row_x, row_y, col_count| {
                        get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
                    },
                )
            },
            |uc, seq_index| {
                u_clust_pd_search_all(uc, seq_index, |seq_indexi, seq_indexj| {
                    u_clust_pd_get_prot_dist_pair(
                        uc,
                        seq_indexi,
                        seq_indexj,
                        None,
                        |seqi, li, seqj, lj| {
                            let mut pi = PathInfo::default();
                            with_dp_mem_l17(|mem| {
                                viterbi_fast_mem(mem, seqi, li, seqj, lj, &mut pi);
                            });
                            pi
                        },
                        |row_x, row_y, col_count| {
                            get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
                        },
                    )
                })
            },
        );
        format!("{run_log1}{stats1}{run_log2}{stats2}")
    } else if let Some(input_file_name) = &cli.uclust {
        let (_u, _centroids) = cmd_uclust(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.minea.unwrap_or(0.9),
            |label1, label2| align_pair_flat(label1, label2),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.super4 {
        let perm = cli.perm.as_deref().map(str_to_treeperm);
        let (_s4, log) = cmd_super4(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            None,
            perm,
            cli.mega,
            |s4, input_seqs, tree_perm| {
                super4_run(
                    s4,
                    input_seqs,
                    tree_perm,
                    |s4| {
                        super4_set_opts(
                            s4,
                            cli.paircount,
                            cli.super4_minea1,
                            cli.super4_minea2,
                            cli.consiters,
                            cli.refineiters,
                        )
                    },
                    |ec, big_mfa, min_ea| {
                        ea_cluster_run(ec, big_mfa, min_ea, |label1, label2| {
                            ea_cluster_align_seq_pair(label1, label2)
                        });
                        ec.cluster_mfas.clone()
                    },
                    |mpc, cluster_mfa| {
                        mpc_flat_run(
                            mpc,
                            cluster_mfa,
                            cli.input_order,
                            |mpc| mpc_flat_calc_posteriors(mpc),
                            |mpc| {
                                mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                                    upgma5_run_l75(u, "biased", guide_tree)
                                })
                            },
                            |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                            |mpc| {
                                mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                                    mpc_flat_align_alns(mpc, msa1, msa2).0
                                })
                            },
                            |mpc| {
                                mpc_flat_refine(
                                    mpc,
                                    || libc_rand(),
                                    |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
                                )
                            },
                        );
                        mpc.msa
                            .as_ref()
                            .expect("cmd_super4 missing cluster MSA")
                            .clone()
                    },
                    |consensus_seqs| {
                        calc_ea_dist_mx(consensus_seqs, None, |label1, label2, path| {
                            let (ea, aln_path) = align_pair_flat(label1, label2);
                            *path = aln_path;
                            (ea, None)
                        })
                        .0
                    },
                    |guide_tree, labels, dist_mx| {
                        let mut u = UPGMA5::default();
                        upgma5_init(&mut u, labels, dist_mx);
                        upgma5_run_l75(&mut u, "biased", guide_tree);
                    },
                    |none, abc, acb, bca, labels_a, labels_b, labels_c| {
                        permute_tree(none, abc, acb, bca, labels_a, labels_b, labels_c)
                    },
                    |pp, tree| {
                        p_prog_run_guide_tree(pp, tree, |pp, index1, index2| {
                            p_prog_align_and_join(
                                pp,
                                index1,
                                index2,
                                |label, msa1, msa2, pair_count, path| {
                                    p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
                                },
                            )
                        });
                        p_prog_get_final_msa(pp).clone()
                    },
                )
            },
        );
        log
    } else if let Some(input_file_name) = &cli.super5 {
        let perm = cli.perm.as_deref().map(str_to_treeperm);
        let (_s5, _files, log) = cmd_super5(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            None,
            perm,
            cli.perturb,
            cli.super5_minea1,
            cli.input_order,
            cli.mega,
            cli.diversified,
            cli.replicates.is_some(),
            cli.stratified,
            |s5, input_seqs, tree_perm, input_order| {
                super5_run(
                    s5,
                    input_seqs,
                    tree_perm,
                    input_order,
                    |d, input_seqs| {
                        derep_run(d, input_seqs, true);
                        let mut unique = MultiSequence::default();
                        derep_get_unique_seqs(d, &mut unique);
                        unique
                    },
                    |u, unique_seqs, min_ea| {
                        let _log = u_clust_run(u, unique_seqs, min_ea, |label1, label2| {
                            align_pair_flat(label1, label2)
                        });
                        let mut centroids = MultiSequence::default();
                        u_clust_get_centroid_seqs(u, &mut centroids);
                        centroids
                    },
                    |s4, input_seqs, tree_perm| {
                        let _log = super4_run(
                            s4,
                            input_seqs,
                            tree_perm,
                            |s4| {
                                super4_set_opts(
                                    s4,
                                    cli.paircount,
                                    cli.super4_minea1,
                                    cli.super4_minea2,
                                    cli.consiters,
                                    cli.refineiters,
                                )
                            },
                            |ec, big_mfa, min_ea| {
                                ea_cluster_run(ec, big_mfa, min_ea, |label1, label2| {
                                    ea_cluster_align_seq_pair(label1, label2)
                                });
                                ec.cluster_mfas.clone()
                            },
                            |mpc, cluster_mfa| {
                                mpc_flat_run(
                                    mpc,
                                    cluster_mfa,
                                    input_order,
                                    |mpc| mpc_flat_calc_posteriors(mpc),
                                    |mpc| {
                                        mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                                            upgma5_run_l75(u, "biased", guide_tree)
                                        })
                                    },
                                    |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                                    |mpc| {
                                        mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                                            mpc_flat_align_alns(mpc, msa1, msa2).0
                                        })
                                    },
                                    |mpc| {
                                        mpc_flat_refine(
                                            mpc,
                                            || libc_rand(),
                                            |mpc, msa1, msa2| {
                                                mpc_flat_align_alns(mpc, msa1, msa2).0
                                            },
                                        )
                                    },
                                );
                                mpc.msa
                                    .as_ref()
                                    .expect("cmd_super5 missing Super4 cluster MSA")
                                    .clone()
                            },
                            |consensus_seqs| {
                                calc_ea_dist_mx(consensus_seqs, None, |label1, label2, path| {
                                    let (ea, aln_path) = align_pair_flat(label1, label2);
                                    *path = aln_path;
                                    (ea, None)
                                })
                                .0
                            },
                            |guide_tree, labels, dist_mx| {
                                let mut u = UPGMA5::default();
                                upgma5_init(&mut u, labels, dist_mx);
                                upgma5_run_l75(&mut u, "biased", guide_tree);
                            },
                            |none, abc, acb, bca, labels_a, labels_b, labels_c| {
                                permute_tree(none, abc, acb, bca, labels_a, labels_b, labels_c)
                            },
                            |pp, tree| {
                                p_prog_run_guide_tree(pp, tree, |pp, index1, index2| {
                                    p_prog_align_and_join(
                                        pp,
                                        index1,
                                        index2,
                                        |label, msa1, msa2, pair_count, path| {
                                            p_prog_align_ms_as_flat(
                                                label, msa1, msa2, pair_count, path,
                                            )
                                        },
                                    )
                                });
                                p_prog_get_final_msa(pp).clone()
                            },
                        );
                    },
                );
                String::new()
            },
        );
        log
    } else if let Some(input_file_name) = &cli.super6 {
        let force_nucleo = if cli.nt {
            Some(true)
        } else if cli.amino {
            Some(false)
        } else {
            None
        };
        let (_s6, log) = cmd_super6(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            force_nucleo,
            cli.super6_maxpd1,
            |s6, input_seqs| {
                super6_run(
                    s6,
                    input_seqs,
                    |uc, input_seqs, seq_indexes, max_pd| {
                        let _run_log = u_clust_pd_run(
                            uc,
                            input_seqs,
                            seq_indexes,
                            max_pd,
                            cli.threads.unwrap_or(1),
                            |uc, seq_index, centroids| {
                                u_clust_pd_search(
                                    uc,
                                    seq_index,
                                    centroids,
                                    |seq_indexi, seq_indexj| {
                                        u_clust_pd_get_prot_dist_pair(
                                            uc,
                                            seq_indexi,
                                            seq_indexj,
                                            None,
                                            |seqi, li, seqj, lj| {
                                                let mut mem = XDPMem::default();
                                                let mut pi = PathInfo::default();
                                                viterbi_fast_mem(
                                                    &mut mem, seqi, li, seqj, lj, &mut pi,
                                                );
                                                pi
                                            },
                                            |row_x, row_y, col_count| {
                                                get_prot_dist_l42(
                                                    row_x.as_bytes(),
                                                    row_y.as_bytes(),
                                                    col_count,
                                                )
                                            },
                                        )
                                    },
                                )
                            },
                        );
                        u_clust_pd_get_cluster_mf_as(uc)
                    },
                    |mfa1, mfa2, target_pair_count| {
                        get_prot_dist_mfa_pair(
                            mfa1,
                            mfa2,
                            target_pair_count,
                            |seqi, li, seqj, lj| {
                                get_prot_dist_seq_pair(
                                    seqi,
                                    li,
                                    seqj,
                                    lj,
                                    None,
                                    |seqi, li, seqj, lj| {
                                        let mut mem = XDPMem::default();
                                        let mut pi = PathInfo::default();
                                        viterbi_fast_mem(&mut mem, seqi, li, seqj, lj, &mut pi);
                                        pi
                                    },
                                    |row_x, row_y, col_count| {
                                        get_prot_dist_l42(
                                            row_x.as_bytes(),
                                            row_y.as_bytes(),
                                            col_count,
                                        )
                                    },
                                )
                            },
                        ) as f32
                    },
                    |guide_tree, labels, dist_mx| {
                        let mut u = UPGMA5::default();
                        upgma5_init(&mut u, labels, dist_mx);
                        upgma5_run_l75(&mut u, "biased", guide_tree);
                    },
                    |pp, tree| {
                        p_prog_run_guide_tree(pp, tree, |pp, index1, index2| {
                            p_prog_align_and_join(
                                pp,
                                index1,
                                index2,
                                |label, msa1, msa2, pair_count, path| {
                                    p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
                                },
                            )
                        });
                    },
                    |mpc, cluster_mfa| {
                        mpc_flat_run(
                            mpc,
                            cluster_mfa,
                            cli.input_order,
                            |mpc| mpc_flat_calc_posteriors(mpc),
                            |mpc| {
                                mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                                    upgma5_run_l75(u, "biased", guide_tree)
                                })
                            },
                            |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                            |mpc| {
                                mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                                    mpc_flat_align_alns(mpc, msa1, msa2).0
                                })
                            },
                            |mpc| {
                                mpc_flat_refine(
                                    mpc,
                                    || libc_rand(),
                                    |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
                                )
                            },
                        );
                        mpc.msa
                            .as_ref()
                            .expect("cmd_super6 missing cluster MSA")
                            .clone()
                    },
                )
            },
        );
        log
    } else if let Some(input_file_name) = &cli.upgma5 {
        let (_u, _tree, log) = cmd_upgma5(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.reseek,
            cli.scaledist,
            cli.eadist,
            cli.linkage.as_deref(),
            |u, linkage| {
                let mut tree = Tree::default();
                upgma5_run_l75(u, linkage, &mut tree);
                tree
            },
        );
        log
    } else if let Some(input_file_name) = &cli.upgma5_msa {
        let (_u, _tree, log) = cmd_upgma5_msa(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.linkage.as_deref(),
            |seqi, seqj, col_count| get_prot_dist_l42(seqi.as_bytes(), seqj.as_bytes(), col_count),
            |u, linkage| {
                let mut tree = Tree::default();
                upgma5_run_l75(u, linkage, &mut tree);
                tree
            },
        );
        log
    } else if let Some(input_file_name) = &cli.guide_tree_join_order {
        cmd_guide_tree_join_order(input_file_name, cli.output.as_deref())
    } else if let Some(input_file_name) = &cli.permute_tree {
        let (_tree_abc, _tree_acb, _tree_bca, _labels_a, _labels_b, _labels_c) =
            cmd_permute_tree(input_file_name, cli.prefix.as_deref());
        String::new()
    } else if let Some(input_file_name) = &cli.tree_subset_nodes {
        cmd_tree_subset_nodes(
            input_file_name,
            cli.nodes.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.right,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.divide_tree {
        let (_subtree, _supertree) = cmd_divide_tree(
            input_file_name,
            cli.label1.as_deref().unwrap_or(""),
            cli.label2.as_deref().unwrap_or(""),
            cli.subtreeout.as_deref().unwrap_or(""),
            cli.supertreeout.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.shrub {
        let (_shrub_lcas, _pt, log) = cmd_shrub(input_file_name, cli.n);
        log
    } else if let Some(input_file_name) = &cli.super7 {
        if cli.mega {
            let (_s7, _guide_tree, log) = cmd_super7_mega(
                input_file_name,
                cli.output.as_deref().unwrap_or(""),
                cli.shrub_size,
                cli.guidetreein.as_deref(),
                cli.distmxin.as_deref(),
            );
            log
        } else {
            let mega_loaded = MEGA_STATE.lock().unwrap().loaded;
            let (_s7, _guide_tree, log) = cmd_super7(
                input_file_name,
                cli.output.as_deref().unwrap_or(""),
                cli.shrub_size,
                cli.guidetreein.as_deref(),
                cli.distmxin.as_deref(),
                mega_loaded,
                |u, guide_tree| upgma5_run_l75(u, "avg", guide_tree),
                |input, guide_tree| {
                    let _ = calc_guide_tree_sw_blosum62(input, guide_tree);
                },
                |s7, input_seqs, guide_tree, shrub_size| {
                    super7_run(
                        s7,
                        input_seqs,
                        guide_tree,
                        shrub_size,
                        |mpc, shrub_input| {
                            mpc_flat_run(
                                mpc,
                                shrub_input,
                                cli.input_order,
                                |mpc| mpc_flat_calc_posteriors(mpc),
                                |mpc| {
                                    mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                                        upgma5_run_l75(u, "biased", guide_tree)
                                    })
                                },
                                |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                                |mpc| {
                                    mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                                        mpc_flat_align_alns(mpc, msa1, msa2).0
                                    })
                                },
                                |mpc| {
                                    mpc_flat_refine(
                                        mpc,
                                        || libc_rand(),
                                        |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
                                    )
                                },
                            );
                            mpc.msa
                                .as_ref()
                                .expect("cmd_super7 missing shrub MSA")
                                .clone()
                        },
                        |pp, shrub_tree| {
                            p_prog_run_guide_tree(pp, shrub_tree, |pp, index1, index2| {
                                p_prog_align_and_join(
                                    pp,
                                    index1,
                                    index2,
                                    |label, msa1, msa2, pair_count, path| {
                                        p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
                                    },
                                )
                            });
                            p_prog_get_final_msa(pp).clone()
                        },
                    )
                },
            );
            log
        }
    } else if let Some(input_file_name) = &cli.sweep {
        let grid_spec = cli.gridspec.as_deref().unwrap_or_else(|| {
            die("Must set -gridspec");
        });
        let (_s, _bench, log, _fev) = cmd_sweep(
            input_file_name,
            cli.refdir.as_deref().unwrap_or(""),
            grid_spec,
            cli.fev.as_deref().unwrap_or(""),
            cli.blosumpct,
            cli.substmx.as_deref(),
            cli.treeiters,
            |ap, ref_name, input, ref_msa, _q2| {
                set_global_input_ms(input);
                let mut m3 = Muscle3::default();
                let final_msa = muscle3_run(
                    &mut m3,
                    ap,
                    input,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input, weights, tree| {
                        p_prog3_run(
                            pp,
                            input,
                            weights,
                            tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                );
                let mut qs = QScorer::default();
                let _ = q_scorer_run_l346(&mut qs, ref_name, &final_msa, ref_msa, false);
                (qs.q as f64, qs.tc as f64)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.spatter {
        let warmup_pct = cli.warmup_pct.unwrap_or_else(|| {
            die("Must set -warmup_pct");
        });
        let maxiters = cli.maxiters.unwrap_or_else(|| {
            die("Must set -maxiters");
        });
        let maxfailiters = cli.maxfailiters.unwrap_or_else(|| {
            die("Must set -maxfailiters");
        });
        let triesperiter = cli.triesperiter.unwrap_or_else(|| {
            die("Must set -triesperiter");
        });
        let shrink = cli.shrink.unwrap_or_else(|| {
            die("Must set -shrink");
        });
        let grid_spec = cli.gridspec.as_deref().unwrap_or_else(|| {
            die("Must set -gridspec");
        });
        let spatter_spec = cli.spatterspec.as_deref().unwrap_or_else(|| {
            die("Must set -spatterspec");
        });
        let (_s1, _s2, _bench, _warmup_bench, log, _output1_fev, _fev) = cmd_spatter(
            input_file_name,
            cli.refdir.as_deref().unwrap_or(""),
            warmup_pct,
            grid_spec,
            spatter_spec,
            cli.output1.as_deref().unwrap_or(""),
            cli.fev.as_deref().unwrap_or(""),
            cli.blosumpct,
            cli.substmx.as_deref(),
            maxiters,
            maxfailiters,
            triesperiter,
            shrink,
            |ap, ref_name, input, ref_msa, _q2| {
                set_global_input_ms(input);
                let mut m3 = Muscle3::default();
                let final_msa = muscle3_run(
                    &mut m3,
                    ap,
                    input,
                    |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
                    |pp, input, weights, tree| {
                        p_prog3_run(
                            pp,
                            input,
                            weights,
                            tree,
                            |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                            |prof_a,
                             weight_a,
                             prof_b,
                             weight_b,
                             subst_mx_letter,
                             gap_open,
                             path| {
                                align_two_profs_given_path(
                                    prof_a,
                                    weight_a,
                                    prof_b,
                                    weight_b,
                                    subst_mx_letter,
                                    gap_open,
                                    path,
                                )
                            },
                        );
                        pp.msa.clone()
                    },
                );
                let mut qs = QScorer::default();
                let _ = q_scorer_run_l346(&mut qs, ref_name, &final_msa, ref_msa, false);
                (qs.q as f64, qs.tc as f64)
            },
        );
        log
    } else if let Some(input_file_name) = &cli.mega2 {
        let (_score, _pi, _out) = cmd_mega2(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.gapopen,
            cli.gapext,
            cli.termgapopen,
            cli.termgapext,
        );
        String::new()
    } else if let Some(input_file_name) = &cli.mega_msas {
        mega_from_file(input_file_name);
        let _outputs = cmd_mega_msas(
            cli.input.as_deref().unwrap_or_else(|| {
                die("Must set -input");
            }),
            cli.output.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.pprog {
        let (_pp, _guide_tree) = cmd_pprog(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.guidetreeout.as_deref(),
            cli.paircount,
            |label, msa1, msa2, pair_count, path| {
                p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.pprog2 {
        let _pp = cmd_pprog2(
            input_file_name,
            cli.joins.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.paircount,
            |label, msa1, msa2, pair_count, path| {
                p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.pprog_tree {
        if cli.guidetreein.is_none() {
            die("Must set -guidetreein");
        }
        let _pp = cmd_pprog_tree(
            input_file_name,
            cli.guidetreein.as_deref().unwrap(),
            cli.output.as_deref().unwrap_or(""),
            cli.paircount,
            |label, msa1, msa2, pair_count, path| {
                p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.pprogt {
        let (_pp, _guide_tree) = cmd_pprogt(
            input_file_name,
            cli.guidetreein.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            cli.guidetreeout.as_deref(),
            cli.paircount,
            |label, msa1, msa2, pair_count, path| {
                p_prog_align_ms_as_flat(label, msa1, msa2, pair_count, path)
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.strip_anchors {
        let _anchor_count = cmd_strip_anchors(input_file_name, cli.output.as_deref().unwrap_or(""));
        String::new()
    } else if let Some(input_file_name) = &cli.strip_gappy {
        let (_discard_col_count, _discard_row_count) = cmd_strip_gappy(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
            cli.max_gap_fract_row.unwrap_or(0.5),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.strip_gappy_cols {
        let _gappy_count = cmd_strip_gappy_cols(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.strip_gappy_rows {
        let _discard_count = cmd_strip_gappy_rows(
            input_file_name,
            cli.output.as_deref().unwrap_or(""),
            cli.max_gap_fract.unwrap_or(0.5),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.trimtoref {
        let _trimmed = cmd_trimtoref(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(input_file_name) = &cli.trimtoref_efa {
        let _trimmed = cmd_trimtoref_efa(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
        );
        String::new()
    } else if let Some(query_file_name) = &cli.usorter {
        cmd_usorter(query_file_name, cli.input.as_deref().unwrap_or(""))
    } else if let Some(input_file_name) = &cli.transaln {
        let (_ta, _extended) = cmd_transaln(
            input_file_name,
            cli.ref_.as_deref().unwrap_or(""),
            cli.output.as_deref().unwrap_or(""),
            |input_label, ref_label, path| {
                let (_ea, aln_path) = align_pair_flat(input_label, ref_label);
                *path = aln_path;
            },
        );
        String::new()
    } else if let Some(input_file_name) = &cli.transalnref {
        if cli.input2.is_none() {
            die("Must set -input2");
        }
        if cli.output.is_none() {
            die("Must set -output");
        }
        if cli.label.is_none() {
            die("Must set -label");
        }
        let (_ta, _extended, log) = cmd_transalnref(
            input_file_name,
            cli.input2.as_deref().unwrap(),
            cli.label.as_deref().unwrap(),
            cli.output.as_deref().unwrap(),
            |seqr, lr, seqa, la| {
                let mut mem = XDPMem::default();
                let mut pi = PathInfo::default();
                viterbi_fast_mem(&mut mem, seqr, lr, seqa, la, &mut pi);
                pi
            },
        );
        log
    } else if let Some(input_file_name) = &cli.hmmdump {
        let _files = cmd_hmmdump(input_file_name, cli.nt, |hp| {
            hmm_params_cmd_line_update(
                hp, cli.s_is, cli.s_il, cli.m_is, cli.m_il, cli.is_is, cli.il_il,
            )
        });
        String::new()
    } else if cli.test_malloc.is_some() {
        cmd_test_malloc()
    } else if cli.swsimple2.is_some() {
        cmd_swsimple2()
    } else if cli.build_guide_tree.is_some() {
        cmd_build_guide_tree()
    } else if cli.test.is_some() {
        cmd_test_l4()
    } else if cli.test_mega.is_some() {
        cmd_test_mega()
    } else if let Some(iter_count) = &cli.test_sw_aa {
        let iters = str_to_uint_l1278(iter_count, false);
        cmd_test_sw_aa(
            iters,
            |a, b, pos_a, pos_b, path| {
                let ps = PathScorerAABLOSUM62 {
                    gap_open: -3.0,
                    gap_ext: -1.0,
                    seq_a: a.to_string(),
                    seq_b: b.to_string(),
                    base: PathScorer {
                        la: a.len() as uint,
                        lb: b.len() as uint,
                        ..PathScorer::default()
                    },
                };
                path_scorer_get_local_score(
                    ps.base.la,
                    ps.base.lb,
                    pos_a,
                    pos_b,
                    path,
                    |from_state, to_state, pa, pb| {
                        path_scorer_get_score(
                            from_state,
                            to_state,
                            pa,
                            pb,
                            |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps, pa, pb),
                            |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps, pa, pb),
                        )
                    },
                )
            },
            |a, b, gap_open, gap_ext| {
                let mut mem = XDPMem::default();
                sw_fast_strings_blosum62(&mut mem, a, b, gap_open, gap_ext)
            },
            |ma, pb, gap_open, gap_ext| {
                let mut mem = XDPMem::default();
                sw_fast_masm_mega_prof(&mut mem, ma, pb, gap_open, gap_ext)
            },
        )
    } else if cli.test_sw_mm.is_some() {
        cmd_test_sw_mm(
            |pos_a, pos_b, path| {
                let rows_a = split("WAQHEAW", '|');
                let ma = make_masm_a_as(&rows_a);
                let pb = make_mega_profile_aa("CTFWH");
                let mut ps = PathScorerMASMMega::default();
                path_scorer_masm_mega_init(&mut ps, &ma, &pb);
                path_scorer_get_local_score(
                    ps.base.la,
                    ps.base.lb,
                    pos_a,
                    pos_b,
                    path,
                    |from_state, to_state, pa, pb| {
                        path_scorer_get_score(
                            from_state,
                            to_state,
                            pa,
                            pb,
                            |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
                            |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
                        )
                    },
                )
            },
            |ma, pb, gap_open, gap_ext| {
                let mut mem = XDPMem::default();
                sw_fast_masm_mega_prof(&mut mem, ma, pb, gap_open, gap_ext)
            },
            |ma, pb, la, lb, mut pos_a, mut pos_b, path| {
                let mut ps = PathScorerMASMMega::default();
                path_scorer_masm_mega_init(&mut ps, ma, pb);
                ps.base.la = la;
                ps.base.lb = lb;
                let mut out = String::new();
                let col_count = path.len() as uint;
                assert!(col_count > 0);
                assert_eq!(path.as_bytes()[0], b'M');
                assert_eq!(path.as_bytes()[col_count as usize - 1], b'M');
                let mut total = 0.0f32;
                let mut last_state = 'M';
                for (col, state_byte) in path.bytes().enumerate() {
                    let state = char::from(state_byte);
                    let score = path_scorer_get_score(
                        last_state,
                        state,
                        pos_a,
                        pos_b,
                        |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
                    );
                    total += score;
                    let mut score_s = format!("{score:.3}");
                    while score_s.contains('.') && score_s.ends_with('0') {
                        score_s.pop();
                    }
                    if score_s.ends_with('.') {
                        score_s.pop();
                    }
                    let mut total_s = format!("{total:.3}");
                    while total_s.contains('.') && total_s.ends_with('0') {
                        total_s.pop();
                    }
                    if total_s.ends_with('.') {
                        total_s.pop();
                    }
                    out.push_str(&format!(
                        "Col {col:3}  PosA {pos_a:3}  PosB {pos_b:3}  {last_state}{state}  {score_s:>10}  {total_s:>10}\n"
                    ));
                    match state {
                        'M' => {
                            pos_a += 1;
                            pos_b += 1;
                        }
                        'D' => pos_a += 1,
                        'I' => pos_b += 1,
                        _ => panic!("invalid path state"),
                    }
                    last_state = state;
                }
                assert!(pos_a <= ps.base.la && pos_b <= ps.base.lb);
                out
            },
        )
    } else if cli.swtest.is_some() {
        cmd_swtest()
    } else if cli.swtestmm.is_some() {
        cmd_swtestmm()
    } else if cmd_count == 1 {
        die("CLI dispatch for this command is not implemented");
    } else {
        String::new()
    };
    if !out.is_empty() {
        print!("{out}");
    }
}
