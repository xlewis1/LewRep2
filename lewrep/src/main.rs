use lewcolour::{Colour, Style, Coloured};
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, stdin, BufRead, BufReader, IsTerminal, Write};
use std::path::{Path, PathBuf};

struct Config {
    pattern: String,
    ignore_case: bool,
    line_numbers: bool,
    invert_match: bool,
    filenames_only: bool,
    explicit_ignore: bool,
    context_after: usize,
    count_only: bool,
    unrestricted_level: usize,
    explain_mode: bool,
    no_filename: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if !stdin().is_terminal() {
        if args.len() < 2 {
          eprintln!("Error: Pattern required for standard input piping.");
          std::process::exit(1); 
        }
        let pattern = args[1].clone();
        process_stdin(&pattern);
        return;
    }

    if args.len() < 2 {
       eprintln!("Usage: lewrep2 [FLAGS] <PATTERN> [PATHS...]");
       std::process::exit(1); 
    }

    let mut pattern = String::new();
    let mut paths = Vec::new();
    let mut ignore_case = false;
    let mut line_numbers = false;
    let mut invert_match = false;
    let mut filenames_only = false;
    let mut explicit_ignore = false;
    let mut context_after = 0;
    let mut count_only = false;
    let mut unrestricted_level = 0;
    let mut explain_mode = false;
    let mut no_filename = false;

    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        if arg.starts_with('-') && arg.len() > 1 {
            if arg == "-A" {
                if let Some(num_str) = args_iter.next() {
                    context_after = num_str.parse::<usize>().unwrap_or(0);
                }
                continue;
            }

            for c in arg.chars().skip(1) {
                match c {
                    'i' => ignore_case = true,
                    'n' => line_numbers = true,
                    'v' => invert_match = true,
                    'l' => filenames_only = true,
                    'I' => explicit_ignore = true,
                    'c' => count_only = true,
                    'u' => unrestricted_level += 1,
                    'X' => explain_mode = true,
                    'h' => no_filename = true,
                    _ => {
                       eprintln!("Error: Unknown flag '-{}'", c);
                       std::process::exit(1); 
                    }
                }
            }
        } else if pattern.is_empty() {
           pattern = arg.clone(); 
        } else {
           paths.push(PathBuf::from(arg));  
        }
    }

    if pattern.is_empty() {
       eprintln!("Error: Missing search pattern target.");
       std::process::exit(1); 
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let config = Config {
        pattern,
        ignore_case,
        line_numbers,
        invert_match,
        filenames_only,
        explicit_ignore,
        context_after,
        count_only,
        unrestricted_level,
        explain_mode,
        no_filename,
    };

    let mut target_files = Vec::new();
    for path in paths {
       let mut walker_builder = WalkBuilder::new(path);
       
       if config.unrestricted_level >= 1 {
           walker_builder.hidden(false);
        } else {
           walker_builder.hidden(true);
        }

        if config.unrestricted_level >= 2 || config.explicit_ignore {
            walker_builder.git_ignore(false);
        } else {
            walker_builder.git_ignore(true);
        }


        let walker = walker_builder.build();

        for entry in walker.flatten() {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                target_files.push(entry.into_path());
            }
        }
    }

    target_files.par_iter().for_each(|file_path| {
        let _ = search_in_file(file_path, &config);
    });
}

struct CustomSink<F> where F: for<'a> Fn(&'a str) -> Coloured<'a> {
    file_name: String,
    filenames_only: bool,
    show_line_numbers: bool,
    orange_formatter: F,
    count_only: bool,
    match_count: usize,
    explain_mode: bool,
    pattern: String,
    ignore_case: bool,
    no_filename: bool,
}

impl<F> CustomSink<F> where F: for<'a> Fn(&'a str) -> Coloured<'a> {
    fn execute_explanation(&self, matched_bytes: &[u8]) {
        let line_text = String::from_utf8_lossy(matched_bytes);

        let compiled_regex = match regex::RegexBuilder::new(&self.pattern)
            .case_insensitive(self.ignore_case)
            .build()

        {
            Ok(re) => re,
            Err(_) => return,
        };

        if let Some(captures) = compiled_regex.captures(&line_text) {
            let mut out = io::stdout().lock();

            if write!(out, "  ").is_ok() {
                Coloured::with_style("[EXPLAIN]", Colour::Cyan, Style::bold()).write_to(&mut out).ok();
                let _ = write!(out, "\n");
            }

            if write!(out, "    Full Match: '").is_ok() {
                let full_match = captures.get(0).map_or("", |m| m.as_str());
                Coloured::new(full_match, Colour::Yellow).write_to(&mut out).ok();
                let _ = write!(out, "'\n");
            }

            for i in 1..captures.len() {
                if let Some(group_match) = captures.get(i) {
                    let group_name = compiled_regex.capture_names()
                        .nth(i)
                        .flatten()
                        .map(|name| format!(" ({})", name))
                        .unwrap_or_default();

                    if write!(out, "    => Group ").is_ok() {
                       Coloured::new(&i.to_string(), Colour::Green).write_to(&mut out).ok();
                       Coloured::new(&group_name, Colour::Blue).write_to(&mut out).ok();
                       let _ = write!(out, ": '");
                       Coloured::new(group_match.as_str(), Colour::Rgb(255, 135, 0)).write_to(&mut out).ok();
                       let _ = writeln!(out, "' (at bytes {}-{})", group_match.start(), group_match.end()); 
                    }
                }
            }
        }
    }
}

impl<F> Sink for CustomSink<F> where F: for<'a> Fn(&'a str) -> Coloured<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &grep_searcher::Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        if self.count_only {
            self.match_count += 1;
            return Ok(true);
        }

        if self.filenames_only {
            println!("{}", self.file_name);
            return Ok(false);
        }

        let mut out = io::stdout().lock();

        let clean_line = String::from_utf8_lossy(mat.bytes()).trim_end_matches(['\r', '\n']).to_string();
        let colored_line = (self.orange_formatter)(&clean_line);

        if self.show_line_numbers {
            if let Some(line_num) = mat.line_number() {
                if !self.no_filename {
                   Coloured::new(&self.file_name, Colour::Purple).write_to(&mut out)?;
                   write!(out, ":")?; 
                }
                Coloured::with_style(&line_num.to_string(), Colour::Magenta, Style::bold()).write_to(&mut out)?;
                write!(out, ": ")?;
                colored_line.write_to(&mut out)?;
                write!(out, "\n")?;
            }
        } else {
           if !self.no_filename {
              Coloured::new(&self.file_name, Colour::Purple).write_to(&mut out)?;
              write!(out, ": ")?;
            }
            colored_line.write_to(&mut out)?;
            write!(out, "\n")?;      
        }
        
        if self.explain_mode {
            self.execute_explanation(mat.bytes());
        }

        Ok(true)
    }

    fn context(&mut self, _searcher: &grep_searcher::Searcher, mat: &grep_searcher::SinkContext<'_>) -> Result<bool, Self::Error> {
        if self.count_only {
            return Ok(true);
        }
        
        let clean_line = String::from_utf8_lossy(mat.bytes())
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        let colored_line = (self.orange_formatter)(&clean_line);
        let mut out = io::stdout().lock();

        if self.show_line_numbers {
            if let Some(line_num) = mat.line_number() {
                if !self.no_filename {
                   Coloured::new(&self.file_name, Colour::Purple).write_to(&mut out)?;
                   write!(out, "-")?; 
                }
                Coloured::with_style(&line_num.to_string(), Colour::Magenta, Style::bold()).write_to(&mut out)?;
                write!(out, "- ")?;
                colored_line.write_to(&mut out)?;
                write!(out, "\n")?;
            }
        } else {
            if !self.no_filename {
               Coloured::new(&self.file_name, Colour::Purple).write_to(&mut out)?;
               write!(out, "- ")?; 
            }
            colored_line.write_to(&mut out)?;
            write!(out, "\n")?;
        }
        Ok(true)
    }
}

fn process_stdin(pattern: &str) {
    let reader = BufReader::new(stdin());
    let mut out = io::stdout().lock();

    for (idx, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            if line.contains(pattern) {
               Coloured::with_style(&(idx + 1).to_string(), Colour::Magenta, Style::bold()).write_to(&mut out).ok();
               if write!(out, ": ").is_ok() {
                   Coloured::new(&line, Colour::Rgb(255, 135, 0)).write_to(&mut out).ok();
                   let _ = write!(out, "\n");
                }
            }
        }
    }
}

fn search_in_file(path: &Path, config: &Config) -> io::Result<()> {
    let file = File::open(path)?;

    let matcher = match RegexMatcherBuilder::new()
        .case_insensitive(config.ignore_case)
        .build(&config.pattern)
    {
        Ok(m) => m,
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
    };

    let mut searcher = SearcherBuilder::new()
        .after_context(config.context_after)
        .invert_match(config.invert_match)
        .build();

    let orange_formatter: for<'a> fn(&'a str) -> Coloured<'a> = |line| {
        Coloured::new(line, Colour::Orange)
    };

    let mut sink = CustomSink {
        file_name: path.to_string_lossy().into_owned(),
        filenames_only: config.filenames_only,
        show_line_numbers: config.line_numbers,
        orange_formatter,
        count_only: config.count_only,
        match_count: 0,
        explain_mode: config.explain_mode,
        pattern: config.pattern.clone(),
        ignore_case: config.ignore_case,
        no_filename: config.no_filename,
    };

    searcher.search_file(&matcher, &file, &mut sink)?;

    if config.count_only && sink.match_count > 0 {
        let mut out = io::stdout().lock();
        if !config.no_filename {
            Coloured::new(&sink.file_name, Colour::Purple).write_to(&mut out)?;
            write!(out, ":")?;
        }
        let _ = writeln!(out, "{}", sink.match_count);
    }

    Ok(())
}