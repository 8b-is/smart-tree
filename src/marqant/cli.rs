use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use marqant::{mq2_uni_decode, mq2_uni_encode, read_mq_metadata, Marqant, MQ2_UNI_DICT_ID};

pub fn run_cli() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return print_help();
    };

    match cmd.as_str() {
        "dict-id" => {
            // Print dict_id derived from ~T/~S if present; or --uni prints built-in UNI dict id
            let mut input: Option<PathBuf> = None;
            let mut uni = false;
            for a in args.by_ref() {
                match a.as_str() {
                    "--uni" => uni = true,
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            if uni {
                println!("{}", MQ2_UNI_DICT_ID);
                return Ok(());
            }
            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let info = read_mq_metadata(&mq)?;
            if let Some(id) = info.dict_id {
                println!("{}", id);
            } else {
                return Err(anyhow!(
                    "no ~T/~S maps present; cannot derive dict_id (use --uni for built-in)"
                ));
            }
        }
        "uni-encode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let bytes = match input {
                Some(path) => {
                    fs::read(&path).with_context(|| format!("failed reading {}", path.display()))?
                }
                None => {
                    let mut buf = Vec::new();
                    io::stdin().read_to_end(&mut buf)?;
                    buf
                }
            };
            let enc = mq2_uni_encode(&bytes)?;
            match output {
                Some(path) => fs::write(&path, enc)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(&enc)?;
                }
            }
        }
        "uni-decode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let bytes = match input {
                Some(path) => {
                    fs::read(&path).with_context(|| format!("failed reading {}", path.display()))?
                }
                None => {
                    let mut buf = Vec::new();
                    io::stdin().read_to_end(&mut buf)?;
                    buf
                }
            };
            let dec = mq2_uni_decode(&bytes)?;
            match output {
                Some(path) => fs::write(&path, dec)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(&dec)?;
                }
            }
        }
        "compress" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut use_zlib = false;
            let mut use_semantic = false;
            let mut std_id: Option<String> = None;

            let iter = args.by_ref();
            while let Some(a) = iter.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = iter.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    "--binary" => {
                        use_zlib = true;
                    }
                    "--semantic" => {
                        use_semantic = true;
                    }
                    "--std" => {
                        let Some(id) = iter.next() else {
                            return Err(anyhow!("missing value for --std"));
                        };
                        std_id = Some(id);
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }

            let content = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };

            let mut flags = String::new();
            if use_zlib {
                flags.push_str("-zlib");
            }
            if use_semantic {
                if !flags.is_empty() {
                    flags.push(' ');
                }
                flags.push_str("-semantic");
            }
            if let Some(id) = &std_id {
                if !flags.is_empty() {
                    flags.push(' ');
                }
                flags.push_str("-std:");
                flags.push_str(id);
            }
            let flags_opt = if flags.is_empty() {
                None
            } else {
                Some(flags.as_str())
            };

            let mq = Marqant::compress_markdown_with_flags(&content, flags_opt)?;

            match output {
                Some(path) => fs::write(&path, mq)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(mq.as_bytes())?;
                }
            }
        }
        "decompress" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }

            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let md = Marqant::decompress_marqant(&mq)?;
            match output {
                Some(path) => fs::write(&path, md)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(md.as_bytes())?;
                }
            }
        }
        "analyze" => {
            // very simple analysis: show token count and size effect
            let mut input: Option<PathBuf> = None;
            for a in args.by_ref() {
                match a.as_str() {
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let content = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let (tokens, tokenized) = Marqant::tokenize_content(&content);
            let savings = content.len() as isize - tokenized.len() as isize;
            println!("tokens: {}\nsavings: {} bytes", tokens.len(), savings);
        }
        "inspect" => {
            let mut input: Option<PathBuf> = None;
            let mut show_tokens = false;
            for a in args {
                match a.as_str() {
                    "--show-tokens" => {
                        show_tokens = true;
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let info = read_mq_metadata(&mq)?;
            println!("kind: {}", info.kind);
            if let Some(v) = info.variant.as_deref() {
                println!("variant: {}", v);
            }
            if let Some(ts) = info.timestamp.as_deref() {
                println!("timestamp: {}", ts);
            }
            if let Some(o) = info.original_size {
                println!("original_size: {}", o);
            }
            if let Some(c) = info.compressed_size {
                println!("compressed_size: {}", c);
            }
            if let Some(t) = info.token_count {
                println!("token_count: {}", t);
            }
            if let Some(l) = info.level.as_deref() {
                println!("level/format: {}", l);
            }
            if let Some(id) = info.dict_id.as_deref() {
                println!("dict_id: {}", id);
            }
            if show_tokens {
                if let Some(t) = info.dict_t.as_deref() {
                    println!("~T{}", t);
                }
                if let Some(s) = info.dict_s.as_deref() {
                    println!("~S{}", s);
                }
            }
        }
        _ => return print_help(),
    }

    Ok(())
}

fn print_help() -> Result<()> {
    let help = "mq - Marqant CLI\n\n\
Usage:\n\
  mq dict-id [<file.mq>|stdin] [--uni]\n\
  mq uni-encode <input> [-o <output>]\n\
  mq uni-decode <input> [-o <output>]\n\
  mq compress <input.md> [-o <output.mq>] [--binary] [--semantic] [--std <id>]\n\
  mq decompress <input.mq> [-o <output.md>]\n\
  mq analyze <input.md>\n\
  mq inspect <input.mq> [--show-tokens]\n\n\
If <input> omitted, reads stdin. Writes to stdout if -o omitted.";
    println!("{}", help);
    Ok(())
}
