use crate::{arguments::Args, error::Error};

use files::*;
use methylation_site::*;
use rayon::prelude::*;
use setup::set_up_output_dir;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
};
use structs::*;
use windows::*;

pub mod arguments;
mod error;
mod files;
mod methylation_site;
mod setup;
mod structs;
mod windows;

pub fn extract(args: Args) -> Result<()> {
    let start = std::time::Instant::now();
    let mut args = args;

    // Adj ust window_step to default value
    if args.window_step == 0 {
        args.window_step = args.window_size;
    }

    let methylome_files = load_methylome(&args.methylome)?;
    let annotation_lines = lines_from_file(&args.genome)?;

    let mut genes: Vec<Gene> = Vec::new();

    // Parse annotation file to extract genes
    for line in annotation_lines {
        let line = line?;
        let gene = Gene::from_annotation_file_line(&line, args.invert);
        if let Some(gene) = gene {
            genes.push(gene)
        }
    }

    // number of different chromosomes assuming they are named from 1 to highest
    let chromosome_count = genes
        .iter()
        .max_by_key(|g| g.chromosome)
        .unwrap()
        .chromosome;

    genes.sort_by_key(|g| g.start); // Sort genes by start bp (propably already the case), needed for binary search

    // Structure genes first by chromosome, then by + and - strand => [Chromosome_1(+ Strand, - Strand), Chromosome_2(+,-), ..]
    let mut structured_genes: Vec<GenesByStrand> = vec![
        GenesByStrand {
            sense: Vec::new(),
            antisense: Vec::new()
        };
        chromosome_count.into()
    ];
    // Put genes into their correct bucket
    let mut gene_length_sum = 0;
    let mut sense_gene_count = 0;
    genes.iter().for_each(|g| {
        let chromosome = &mut structured_genes[(g.chromosome - 1) as usize];
        let strand = match &g.strand {
            Strand::Sense => &mut chromosome.sense,
            Strand::Antisense => &mut chromosome.antisense,
        };
        if g.strand == Strand::Sense {
            sense_gene_count += 1;
        }
        gene_length_sum += g.end - g.start;
        strand.push(g.to_owned());
    });
    let average_gene_length = gene_length_sum / genes.len() as i32;
    println!(
        "Average gene length: {} bp, {} genes, of which {} are on the sense strand and {} on the antisense strand",
        average_gene_length,
        genes.len(),
        sense_gene_count,
        genes.len() - sense_gene_count
    );

    // Determine the maximum gene length by iterating over all genes
    let mut max_gene_length: i32 = 100; // if not using absolute window sizes, the maximum gene length will be 100%
    if args.absolute {
        for gene in &genes {
            let length = gene.end - gene.start;
            if length > max_gene_length {
                max_gene_length = length
            }
        }
        println!("The maximum gene length is {} bp", max_gene_length);
    }

    set_up_output_dir(max_gene_length, args.clone())?;

    methylome_files.par_iter().try_for_each_with(
        structured_genes,
        |genome, (path, filename)| -> Result<()> {
            let file = open_file(path, filename)?;
            let mut windows =
                extract_windows(file, genome.to_vec(), max_gene_length as i32, args.clone())?;
            if args.invert {
                windows = windows.inverse();
            }
            windows.save(
                &args.output_dir,
                filename,
                args.window_step as usize,
                args.invert,
            )?;
            let distribution = windows.distribution();
            let path = format!(
                "{}/{}_distribution.txt",
                &args.output_dir,
                filename.to_str().unwrap()
            );
            fs::write(path, distribution)?;
            Ok(())
        },
    )?;

    println!("Done in: {:?}", start.elapsed());
    Ok(())
}
