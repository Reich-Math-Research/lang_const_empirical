// main.rs: the "workhorse" file that searches one linear form
// with a range of different parameters. it can be run with
// manager.py in the python_helpers directory or independently
// for one linear form. it generates two pngs and three jsons
// for data analysis, and sends them to the lang_results directory

use chrono::Local;
use clap::Parser;
use plotters::prelude::*;
use rayon::prelude::*;
use rug::{float::Round, Assign, Float, Integer};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "lang_conjecture_search")]
#[command(about = "Investigate the Constant C(epsilon) in Lang's Conjecture")]
struct Args {
    #[arg(long)]
    a1: Option<String>,
    #[arg(long)]
    a2: Option<String>,
    #[arg(long)]
    b1_min: Option<i64>,
    #[arg(long)]
    b1_max: Option<i64>,
    #[arg(long, default_value_t = 2000)]
    prec: u32,
    #[arg(long, default_value_t = -1.0)]
    eps_min: f64,
    #[arg(long, default_value_t = 2.0)]
    eps_max: f64,
    #[arg(long, default_value_t = 2000)]
    eps_steps: usize,
    #[arg(long)]
    resume: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Point {
    b1: i64,
    b2: i64,
    #[serde(rename = "B")]
    b_max: i64,
    abs_lambda: f64,
    log_abs_lambda: f64,
}

// 1(C) Regime Structure
// 1(C) Regime Structure
#[derive(Serialize, Deserialize)] // Added Deserialize
struct Regime {
    b1: i64,
    b2: i64,
    #[serde(rename = "B")]
    b_max: i64,
    eps_range: (f64, f64),
    slope: f64,
    roth_exponent: String,
}

// 4. Final Output Structure
#[derive(Serialize, Deserialize)] // Added Deserialize
struct FinalOutput {
    args: Args,
    pareto_points: Vec<Point>,
    near_extremal_points: Vec<Point>,
    eps_vals: Vec<f64>,
    c_vals: Vec<f64>,
    regimes: Vec<Regime>,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize)] // Added Deserialize
struct Metadata {
    timestamp: String,
    alpha1: f64,
    alpha2: f64,
    num_pareto: usize,
    num_regimes: usize,
    b_range: (i64, i64),
    eps_inputs: (f64, f64, usize),
    convergent_data: Vec<String>,
}

struct Scratchpad {
    b1f: Float,
    base: Float,
    exact_b2: Float,
    lambda_floor: Float,
    lambda_ceil: Float,
    b2f: Float,
}

impl Scratchpad {
    fn new(prec: u32) -> Self {
        Self {
            b1f: Float::new(prec),
            base: Float::new(prec),
            exact_b2: Float::new(prec),
            lambda_floor: Float::new(prec),
            lambda_ceil: Float::new(prec),
            b2f: Float::new(prec),
        }
    }
}

impl Args {
    fn from_cli_or_prompt() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut args = Args::parse();
        let no_flags = std::env::args_os().len() == 1;
        if no_flags && args.resume.is_none() {
            args.a1 = Some(prompt_string("Enter a1 expression (example: (sqrt(3)-1)/2): ")?);
            args.a2 = Some(prompt_string("Enter a2 expression (example: (sqrt(2)+1)/3): ")?);
            args.b1_min = Some(prompt_parse::<i64>("Enter b1_min: ")?);
            args.b1_max = Some(prompt_parse::<i64>("Enter b1_max: ")?);
            args.prec = prompt_parse_or_default("Enter precision in bits [2000]: ", args.prec)?;
            args.eps_min = prompt_parse_or_default("Enter eps_min [-1.0]: ", args.eps_min)?;
            args.eps_max = prompt_parse_or_default("Enter eps_max [2.0]: ", args.eps_max)?;
            args.eps_steps = prompt_parse_or_default("Enter eps_steps [2000]: ", args.eps_steps)?;
        }
        Ok(args)
    }
}

fn prompt_string(label: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    loop {
        print!("{label}");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim().to_string();
        if !value.is_empty() { return Ok(value); }
        eprintln!("Input cannot be empty.");
    }
}

fn prompt_parse<T: std::str::FromStr>(label: &str) -> Result<T, Box<dyn Error + Send + Sync>>
where T::Err: Error + Send + Sync + 'static {
    loop {
        let s = prompt_string(label)?;
        match s.parse::<T>() {
            Ok(v) => return Ok(v),
            Err(e) => eprintln!("Invalid input: {e}"),
        }
    }
}

fn prompt_parse_or_default<T: std::str::FromStr + Clone>(label: &str, default: T) -> Result<T, Box<dyn Error + Send + Sync>>
where T::Err: Error + Send + Sync + 'static {
    loop {
        print!("{label}");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed.is_empty() { return Ok(default.clone()); }
        match trimmed.parse::<T>() {
            Ok(v) => return Ok(v),
            Err(e) => eprintln!("Invalid input: {e}"),
        }
    }
}

fn sanitize_for_filename(s: &str) -> String {
    s.chars().map(|c| if c == '/' || c == '\\' { '∕' } else if c.is_whitespace() { '_' } else { c }).collect()
}

#[inline]
fn parse_alpha(expr: &str, prec: u32) -> Result<Float, Box<dyn Error + Send + Sync>> {
    let value = meval::eval_str(expr.trim()).map_err(|e| e.to_string())?;
    if !value.is_finite() { return Err("expression evaluated to a non-finite value".into()); }
    Ok(Float::with_val(prec, value))
}

#[inline]
fn prepare_constants(a1: &str, a2: &str, prec: u32) -> Result<(Float, Float, Float, Float, Float), Box<dyn Error + Send + Sync>> {
    let alpha1 = parse_alpha(a1, prec)?;
    let alpha2 = parse_alpha(a2, prec)?;
    let log_alpha1 = Float::with_val(prec, alpha1.ln_ref());
    let log_alpha2 = Float::with_val(prec, alpha2.ln_ref());
    let ratio = -Float::with_val(prec, &log_alpha1 / &log_alpha2);
    Ok((alpha1, alpha2, log_alpha1, log_alpha2, ratio))
}

#[inline]
fn compute_convergents(ratio: &Float, num_convergents: usize, prec: u32) -> Vec<(Integer, Integer)> {
    let mut convergents = Vec::with_capacity(num_convergents);
    let mut x = ratio.clone().abs();
    let mut p_m2 = Integer::from(0); let mut q_m2 = Integer::from(1);
    let mut p_m1 = Integer::from(1); let mut q_m1 = Integer::from(0);
    for _ in 0..num_convergents {
        let (a, _) = x.to_integer_round(Round::Down).unwrap();
        let p = Integer::from(&a * &p_m1) + &p_m2;
        let q = Integer::from(&a * &q_m1) + &q_m2;
        convergents.push((p.clone(), q.clone()));
        let a_f = Float::with_val(prec, &a);
        let x_minus_a = Float::with_val(prec, &x - &a_f);
        if x_minus_a.is_zero() { break; }
        x = Float::with_val(prec, 1.0 / &x_minus_a);
        p_m2 = p_m1; p_m1 = p; q_m2 = q_m1; q_m1 = q;
    }
    convergents
}

#[inline]
fn compute_point_fast(scratch: &mut Scratchpad, log_alpha1: &Float, log_alpha2: &Float, ratio: &Float, b1: i64) -> Point {
    scratch.b1f.assign(b1 as f64);
    scratch.base.assign(&scratch.b1f * log_alpha1);
    scratch.exact_b2.assign(&scratch.b1f * ratio);
    let b2_floor_i = scratch.exact_b2.to_integer_round(Round::Down).unwrap().0;
    let b2_ceil_i = scratch.exact_b2.to_integer_round(Round::Up).unwrap().0;
    let b2_floor = b2_floor_i.to_i64().unwrap_or(0);
    let b2_ceil = b2_ceil_i.to_i64().unwrap_or(0);

    scratch.b2f.assign(&b2_floor_i);
    scratch.lambda_floor.assign(&scratch.b2f * log_alpha2);
    scratch.lambda_floor += &scratch.base;
    scratch.lambda_floor.abs_mut();

    scratch.b2f.assign(&b2_ceil_i);
    scratch.lambda_ceil.assign(&scratch.b2f * log_alpha2);
    scratch.lambda_ceil += &scratch.base;
    scratch.lambda_ceil.abs_mut();

    let (b2, abs_lambda) = if scratch.lambda_floor <= scratch.lambda_ceil {
        (b2_floor, scratch.lambda_floor.to_f64())
    } else {
        (b2_ceil, scratch.lambda_ceil.to_f64())
    };

    Point {
        b1, b2,
        b_max: b1.wrapping_abs().max(b2.wrapping_abs()),
        abs_lambda,
        log_abs_lambda: if abs_lambda > 0.0 { abs_lambda.ln() } else { f64::NEG_INFINITY },
    }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut args = Args::from_cli_or_prompt()?;
    let mut global_candidates = Vec::new();
    let mut completed_b1s = HashSet::new();

    if let Some(resume_file) = &args.resume {
        println!("Resuming from file: {}", resume_file);
        let file = File::open(resume_file)?;
        let reader = BufReader::new(file);
        let payload: FinalOutput = serde_json::from_reader(reader)?;
        if args.a1.is_none() { args.a1 = payload.args.a1.clone(); }
        if args.a2.is_none() { args.a2 = payload.args.a2.clone(); }
        if args.b1_min.is_none() { args.b1_min = payload.args.b1_min; }
        if args.b1_max.is_none() { args.b1_max = payload.args.b1_max; }
        for pt in payload.pareto_points {
            completed_b1s.insert(pt.b1);
            global_candidates.push(pt);
        }
    }

    let a1_str = args.a1.clone().unwrap_or_default();
    let a2_str = args.a2.clone().unwrap_or_default();
    let b1_min = args.b1_min.unwrap();
    let b1_max = args.b1_max.unwrap();

    let sa1 = sanitize_for_filename(&a1_str);
    let sa2 = sanitize_for_filename(&a2_str);
    let base_name = format!("a1_{}_a2_{}_b1_{}_to_{}", sa1, sa2, b1_min, b1_max);
    let run_dir = PathBuf::from("lang_results").join(&base_name);
    create_dir_all(&run_dir)?;

    let json_file = run_dir.join(format!("{}.json", base_name));
    let plot_file = run_dir.join(format!("{}.png", base_name));
    let worst_plot_file = run_dir.join(format!("{}_worst_case.png", base_name));

    println!("Beginning search in range [{}, {}]...", b1_min, b1_max);

    let (alpha1, alpha2, log_alpha1, log_alpha2, ratio) = prepare_constants(&a1_str, &a2_str, args.prec)?;
    let ratio_abs = ratio.clone().abs();

    let b1_values: Vec<i64> = (b1_min..=b1_max)
        .filter(|&b1| b1 != 0 && !completed_b1s.contains(&b1))
        .collect();

    let chunk_size = 10_000;

    // Process in chunks to prevent memory explosion
    for chunk in b1_values.chunks(chunk_size) {
        let mut local_results: Vec<Point> = chunk
            .par_iter()
            .map_init(
                || Scratchpad::new(args.prec),
                |scratch, &b1| compute_point_fast(scratch, &log_alpha1, &log_alpha2, &ratio, b1),
            )
            .collect();

        // 5. The deeper principle: Only keep points that have a chance at Pareto frontier in this chunk
        local_results.sort_unstable_by(|a, b| {
            a.b_max.cmp(&b.b_max).then(a.abs_lambda.partial_cmp(&b.abs_lambda).unwrap_or(Ordering::Equal))
        });

        let mut current_min = f64::INFINITY;
        for pt in local_results {
            if pt.abs_lambda < current_min {
                global_candidates.push(pt);
                current_min = pt.abs_lambda;
            }
        }
    }

    // Now reduce all collected chunk candidates to the true global Pareto Frontier
    global_candidates.sort_unstable_by(|a, b| {
        a.b_max.cmp(&b.b_max).then(a.abs_lambda.partial_cmp(&b.abs_lambda).unwrap_or(Ordering::Equal))
    });

    let mut pareto_points = Vec::new();
    let mut current_min_lambda = f64::INFINITY;
    for pt in global_candidates.clone() {
        if pt.abs_lambda < current_min_lambda {
            pareto_points.push(pt);
            current_min_lambda = pt.abs_lambda;
        }
    }

    // 3. Safety net buffer (top 100 overall smallest abs_lambda)
    let mut near_extremal_points = global_candidates.clone();
    near_extremal_points.sort_unstable_by(|a, b| a.abs_lambda.partial_cmp(&b.abs_lambda).unwrap_or(Ordering::Equal));
    near_extremal_points.truncate(100);

    let convergents = compute_convergents(&ratio, 30, args.prec);

    let ln_abs_lambdas: Vec<f64> = pareto_points.iter().map(|&r| if r.abs_lambda > 0.0 { r.abs_lambda.ln() } else { f64::NEG_INFINITY }).collect();
    let ln_bmax: Vec<f64> = pareto_points.iter().map(|r| (r.b_max as f64).ln()).collect();

    let eps_vals: Vec<f64> = if args.eps_steps <= 1 { vec![args.eps_min] } else {
        (0..args.eps_steps).map(|i| args.eps_min + (args.eps_max - args.eps_min) * (i as f64) / ((args.eps_steps - 1) as f64)).collect()
    };

    let eps_scan: Vec<(f64, usize)> = eps_vals.par_iter().map(|&eps| {
        let exp_factor = 1.0 + eps;
        let mut min_score = f64::INFINITY; let mut min_idx = 0usize;
        for i in 0..pareto_points.len() {
            let score = ln_abs_lambdas[i] + exp_factor * ln_bmax[i];
            if score < min_score { min_score = score; min_idx = i; }
        }
        (min_score.exp(), min_idx)
    }).collect();

    let c_vals: Vec<f64> = eps_scan.iter().map(|t| t.0).collect();

    let mut worst_offenders: HashMap<(i64, i64), (f64, f64, Point)> = HashMap::new();
    for (&eps, &(_, idx)) in eps_vals.iter().zip(&eps_scan) {
        let pt = pareto_points[idx];
        worst_offenders.entry((pt.b1, pt.b2)).and_modify(|range| range.1 = eps).or_insert((eps, eps, pt));
    }

    let mut offenders: Vec<_> = worst_offenders.into_iter().collect();
    offenders.sort_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap_or(Ordering::Equal));

    let mut regimes = Vec::new();
    println!("\n--- Points that dictated the boundary of C(eps) ---");

    for ((b1, b2), (eps_start, eps_end, match_pt)) in &offenders {
        let b1_f = Float::with_val(args.prec, *b1);
        let b2_f = Float::with_val(args.prec, *b2);
        let approx = Float::with_val(args.prec, &b2_f / &b1_f);
        let dist = Float::with_val(args.prec, &approx - &ratio_abs).abs();

        let b1_f64 = (*b1 as f64).abs();
        let mu = if b1_f64 <= 1.0 { 0.0 } else if dist.is_zero() { f64::INFINITY } else { -dist.ln().to_f64() / b1_f64.ln() };

        let mu_str = if mu.is_infinite() { "inf".to_string() } else if mu.is_nan() { "nan".to_string() } else { format!("{:.4}", mu) };

        regimes.push(Regime {
            b1: *b1,
            b2: *b2,
            b_max: match_pt.b_max,
            eps_range: (*eps_start, *eps_end),
            slope: (match_pt.b_max as f64).ln(),
            roth_exponent: mu_str.clone(),
        });

        println!("Point (b1={}, b2={}) -> Dominated eps: [{:.3}, {:.3}] | Max B: {} | Roth mu: {}", b1, b2, eps_start, eps_end, match_pt.b_max, mu_str);
    }

    // Charting
    let max_c = c_vals.iter().copied().fold(0.0_f64, f64::max);
    let y_max = max_c.max(1e-10) * 1.1;
    let root = BitMapBackend::new(&plot_file, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Empirical C(ε) | α1: {}, α2: {} | b-range: [{}, {}]", a1_str, a2_str, b1_min, b1_max), ("sans-serif", 20).into_font().style(FontStyle::Bold))
        .margin(20).x_label_area_size(45).y_label_area_size(65)
        .build_cartesian_2d(args.eps_min..args.eps_max, 0.0..y_max)?;

    chart.configure_mesh().x_desc("epsilon").y_desc("Constant C(epsilon)").draw()?;
    chart.draw_series(LineSeries::new(eps_vals.iter().copied().zip(c_vals.iter().copied()), &RED))?
        .label("Empirical C(epsilon)").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart.configure_series_labels().position(SeriesLabelPosition::UpperLeft).border_style(BLACK).draw()?;
    root.present()?;

    // Worst-case Charting
    let worst_pt = offenders.first().map(|o| o.1.2).unwrap_or(Point { b1: 0, b2: 0, b_max: 1, abs_lambda: 0.0, log_abs_lambda: f64::NEG_INFINITY });
    let mut wc_x = Vec::new(); let mut wc_y = Vec::new();
    for &eps in &eps_vals { wc_x.push(eps); wc_y.push(worst_pt.abs_lambda * (worst_pt.b_max as f64).powf(1.0 + eps)); }

    let wc_min_raw = wc_y.iter().copied().fold(f64::INFINITY, f64::min);
    let wc_max_raw = wc_y.iter().copied().fold(0.0_f64, f64::max);
    let wc_min = if wc_min_raw.is_finite() { (wc_min_raw * 0.95).max(0.0) } else { 0.0 };
    let wc_max = if wc_max_raw > 0.0 { wc_max_raw * 1.05 } else { 1.0 };

    let wc_root = BitMapBackend::new(&worst_plot_file, (1280, 720)).into_drawing_area();
    wc_root.fill(&WHITE)?;
    let mut wc_chart = ChartBuilder::on(&wc_root)
        .caption(format!("Worst-Case Regime (Smallest ε) C(ε) | b-range: [{}, {}]", b1_min, b1_max), ("sans-serif", 20).into_font().style(FontStyle::Bold))
        .margin(20).x_label_area_size(45).y_label_area_size(65)
        .build_cartesian_2d(args.eps_min..args.eps_max, wc_min..wc_max)?;

    wc_chart.configure_mesh().x_desc("epsilon").y_desc("Constant C(epsilon)").draw()?;
    wc_chart.draw_series(LineSeries::new(wc_x.iter().copied().zip(wc_y.iter().copied()), &RED))?
        .label("Worst-Case Constant").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    wc_chart.configure_series_labels().position(SeriesLabelPosition::UpperLeft).border_style(BLACK).draw()?;
    wc_root.present()?;

    // Build the final output payload
    let metadata = Metadata {
        timestamp: Local::now().to_rfc3339(),
        alpha1: alpha1.to_f64(),
        alpha2: alpha2.to_f64(),
        num_pareto: pareto_points.len(),
        num_regimes: regimes.len(),
        b_range: (b1_min, b1_max),
        eps_inputs: (args.eps_min, args.eps_max, args.eps_steps),
        convergent_data: convergents.iter().map(|(p, q)| format!("{}/{}", p, q)).collect(),
    };

    let final_output = FinalOutput {
        args: args.clone(),
        pareto_points,
        near_extremal_points,
        eps_vals,
        c_vals,
        regimes,
        metadata,
    };

    // 4. Output the exact small JSON payload
    serde_json::to_writer_pretty(File::create(&json_file)?, &final_output)?;

    println!("\nOutputs completely generated in: {}", run_dir.display());
    println!("Consolidated JSON: {}", json_file.display());
    println!("Plot: {}", plot_file.display());
    println!("Worst-case plot: {}", worst_plot_file.display());

    Ok(())
}
