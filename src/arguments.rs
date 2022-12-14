use clap::Parser;

/// simple tool to separate a methylome by position within a gene
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of directory containing the methlyome files from which to extract the CG-sites
    #[arg(short, long)]
    pub methylome: String,

    /// Path of the annotation file containing information about beginning and end of gbM-genes
    #[arg(short, long)]
    pub genome: String,

    /// Size of the window in percent of the gbM-gene length or in basepair number if --absolute is supplied
    #[arg(short, long, default_value_t = 5)]
    pub window_size: i32,

    /// Size of the step between the start of each window. Default value is window-size, so no overlapp happens
    #[arg(long, short('s'), default_value_t = 0)]
    pub window_step: i32,

    /// Path of the directory where extracted segments shall be stored
    #[arg(short, long)]
    pub output_dir: String,

    /// Use absolute length in base-pairs for window size instead of percentage of gene length
    #[arg(short, long, default_value_t = false)]
    pub absolute: bool,

    /// Number of basepairs to include upstream and downstream of gene
    #[arg(short, long, default_value_t = 2048)]
    pub cutoff: i32,

    /// Invert strands, to switch from 5' to 3' and vice versa
    #[arg(short, long, default_value_t = false)]
    pub invert: bool,
}
