use clap::{Parser, ValueEnum};
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Ruta base para buscar archivos
    #[arg(long)]
    path: String,

    /// Modo predefinido (js-a-jsx o jsx-a-js)
    #[arg(long, value_enum)]
    modo: Option<Modo>,

    /// Expresión regular personalizada (no usar con --modo)
    #[arg(long)]
    regex: Option<String>,

    /// Nueva extensión (sin punto) (no usar con --modo)
    #[arg(long)]
    extension_nueva: Option<String>,

    /// Solo mostrar qué archivos se modificarían
    #[arg(long, short)]
    dry_run: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Modo {
    JsAJsx,
    JsxAJs,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Validación
    if args.modo.is_some() && (args.regex.is_some() || args.extension_nueva.is_some()) {
        eprintln!("Error: No puedes usar --modo junto con --regex o --extension-nueva.");
        std::process::exit(1);
    }

    let (regex, nueva_ext, require_pattern, filter_ext) = match args.modo {
        Some(Modo::JsAJsx) => (Regex::new(r"/\w*>").unwrap(), "jsx", true, Some("js")),
        Some(Modo::JsxAJs) => (
            Regex::new(".*").unwrap(), // Siempre matchea
            "js",
            false,
            Some("jsx"),
        ),
        None => {
            let re =
                Regex::new(args.regex.as_ref().expect("Falta --regex")).expect("Regex inválida");
            let ext = args
                .extension_nueva
                .as_ref()
                .expect("Falta --extension-nueva");
            (re, ext.as_str(), true, None)
        }
    };

    let mut encontrados = 0;
    let mut renombrados = 0;

    for entry in WalkDir::new(&args.path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            if let Some(filtro_ext) = filter_ext {
                if path.extension().and_then(|e| e.to_str()) != Some(filtro_ext) {
                    continue;
                }
            }

            let contiene = if require_pattern {
                file_contains_pattern(path, &regex)?
            } else {
                true
            };

            if contiene {
                encontrados += 1;
                if change_extension(path, nueva_ext, args.dry_run)? {
                    renombrados += 1;
                }
            }
        }
    }

    println!(
        "\nResumen: {} archivos detectados, {} {}.",
        encontrados,
        renombrados,
        if args.dry_run {
            "simulados"
        } else {
            "renombrados"
        }
    );

    Ok(())
}

fn file_contains_pattern(path: &Path, re: &Regex) -> io::Result<bool> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if re.is_match(&line) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn change_extension(path: &Path, nueva_ext: &str, dry_run: bool) -> io::Result<bool> {
    let mut new_path = PathBuf::from(path);
    new_path.set_extension(nueva_ext);

    if dry_run {
        println!("[Dry Run] {:?} → {:?}", path, new_path);
    } else {
        fs::rename(path, &new_path)?;
        println!("Renombrado: {:?} → {:?}", path, new_path);
    }

    Ok(true)
}
