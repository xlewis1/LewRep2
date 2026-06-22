use lewcolour::{Colour, Style, Coloured};
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, stdin, BufRead, BufReader, IsTerminal, Write, Read};
use std::path::{Path, PathBuf};
use timo::TimoDateTime;

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
    word_regexp: bool,
    tree_view: bool,
    delete_colour: bool,
    hide_all: bool,
    vscode_include: bool,
    json_mode: bool,
    rust_only: bool,
    c_only: bool,
    cpp_only: bool,
    cat_mode: bool,
    show_time: bool,
}

const HELP_TEXT: &str = r#"
================================================================================
                                 LEWREP2 HELP
================================================================================
Usage:
    lewrep2 [FLAGS] [PATTERN] [PATH...]

A parallel grep utility designed to quickly scan directories using regular expressions.

FLAGS:
    -h, --help              Display this quick-reference summary.
    --manpage               Display the comprehensive manual document layout.
    -i, --ignore-case       Perform case-insensitive text evaluation.
    -n, --line-number       Prefix each match line with its 1-based sequential line index.
    -v, --invert-match      Invert query matching selection properties.
    -l, --files-with-matches Print only names of target files matching specifications.
    -c, --count             Print exclusively total matching record line metrics per file.
    -w, --word-regexp       Bound regular expression to validate complete word sequences.
    -T, --tree              Format match visual layout structural mappings hierarchically.
    -d, --delete-colour     Strip text styling components before output execution.
    -j, --json              Stream structural data components formatted as raw JSON.
    -X, --explain           Interactively clarify capture group structural properties.
    -A <NUM>                Append context detailing trailing data lines.
    -R                      Limit target evaluation queries exclusively to Rust files.
    -C                      Limit target evaluation queries exclusively to C files.
    -CPP                    Limit target evaluation queries exclusively to C++ files.
    --Hide                  Omit structured path matching binary formatting properties.
    --vscode                Override defaults to scan structural .vscode tracking areas.
    -u, -uu and -uuu        Shows .gitignore, hidden files and binaries.
    --Hide                  Does the opposite to "-u" and hides hidden files, binaries and .gitignore.
    --cat                   prints cat-style text prints.
    --time                  shows the time via my Timo date/time library crate.

Examples:
    lewrep2 "struct Config" .
    lewrep2 -in "TODO" src/
================================================================================
"#;

const MANPAGE: &str = r#"
LEWREP2(1)                User Commands               LEWREP2(1)

NAME
       lewrep2 - A high-performance parallel grep-class utility.

SYNOPSIS
       lewrep2 [FLAGS] [PATTERN] [PATH...]

DESCRIPTION
       lewrep2 searches for PATTERN in each PATH. It automatically
       respects .gitignore rules, avoids hidden directories, and
       utilizes multi-threaded directory traversal.

FLAGS
       -j, --json
              Enable JSON streaming mode. Outputs matches as clean,
              un-styled JSON objects for tool interoperability.

       -i, --ignore-case
              Perform case-insensitive matching.

       -n, --line-number
              Prefix each line of output with its 1-based line number.

       -v, --invert-match
              Invert matching: select non-matching lines.

       -l, --files-with-matches
              Only print the name of each file that contains matches.
        
       -u --Unrestricted mode 
              include hidden files.

       -uu --include hidden
              includes hidden directories.
        
        -uuu --show everything
              include binary files and all normally excluded content.

       -c, --count
              Only print a count of matching lines per file.

       -w, --word-regexp
              Match only whole words matching PATTERN.

       -T, --tree
              Display results in a structured hierarchical visual tree.

       -d, --delete-colour
              Strip all color output styling.

       -X, --explain
              Explain regular expression match captures interactively.

       -A <NUM>
              Print NUM lines of trailing context after matching lines.

       -R --Rust only
              Exclusively searches Rust files only.

       -CPP -- C++ only
              Exclusively searches C++ files only.
        
       -C --C only
              Exclusively searches C files only.

       --Hide
              Hide structural paths matching binary format signatures.

       --vscode
              Explicitly allow indexing of hidden .vscode directories.

       --manpage
              Display this manual page and exit.

AUTHOR
       Written by xlewis1.
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--help") {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let colored_help = Coloured::new(HELP_TEXT.trim(), Colour::Green);
        let _ = colored_help.write_to(&mut handle);

        let _ = writeln!(handle);
        std::process::exit(0);
    }

    if args.iter().any(|arg| arg == "--manpage") {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let colored_manpage = Coloured::new(MANPAGE.trim(), Colour::Cyan);
        let _ = colored_manpage.write_to(&mut handle);

        let _ = writeln!(handle);
        std::process::exit(0);
    }

    if args.len() < 3 && args.iter().any(|arg| arg == "--time" || arg == "-t") {
        let mut out = io::stdout().lock();
        if let Ok(timo_now) = TimoDateTime::now("Europe/London") {
            let _ = write!(out, "[");
            lewcolour::Coloured::with_style("TIMO RUNTIME", lewcolour::Colour::Cyan, lewcolour::Style::bold()).write_to(&mut out).ok();
            let _ = write!(out, "] ");
            lewcolour::Coloured::new(&timo_now.status_summary(), lewcolour::Colour::Rgb(255, 135, 0)).write_to(&mut out).ok();
            let _ = writeln!(out, "\n──────────────────────────────────────────────────────────");
        }
        std::process::exit(1);
    }

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
    let mut word_regexp = false;
    let mut tree_view = false;
    let mut delete_colour = false;
    let mut hide_all = false;
    let mut vscode_include = false;
    let mut json_mode = false;
    let mut rust_only = false;
    let mut c_only = false;
    let mut cpp_only = false;
    let mut cat_mode = false;
    let mut show_time = false;

    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        if arg == "--cat" {
            cat_mode = true;
            continue;
        }

        if arg == "--time" {
            show_time = true;
            continue;
        }

        if arg == "--Hide" {
            hide_all = true;
            continue;
        }

        if arg == "--vscode" {
            vscode_include = true;
            continue;
        }

        if arg == "-CPP" {
            cpp_only = true;
            continue;
        }

        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 1 {
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
                    'w' => word_regexp = true,
                    'T' => tree_view = true,
                    'd' => delete_colour = true,
                    'j' => json_mode = true,
                    'R' => rust_only = true,
                    'C' => c_only = true,
                    't' => show_time = true,
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
        word_regexp,
        tree_view,
        delete_colour,
        hide_all,
        vscode_include,
        json_mode,
        rust_only,
        c_only,
        cpp_only,
        cat_mode,
        show_time,
    };

    let mut target_files = Vec::new();
    for path in paths {
       let mut walker_builder = WalkBuilder::new(path);
       
       if config.hide_all {
          walker_builder.hidden(true);
          walker_builder.git_ignore(true);
          walker_builder.parents(true);
       } else {
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
        }

        if config.vscode_include && !config.hide_all {
            walker_builder.filter_entry(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    if name == ".vscode" {
                        return true; 
                    }
                }
                if entry.depth() > 0 && entry.path().components().any(|c| c.as_os_str() == ".vscode") {
                    return true;
                }
                !entry.file_name().to_str().map_or(false, |s| s.starts_with('.'))
            });
        }

        let walker = walker_builder.build();

        for entry in walker.flatten() {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if config.rust_only && entry.path().extension().map_or(true, |ext| ext != "rs") {
                    continue;
                }

                if config.c_only {
                    if let Some(ext) = entry.path().extension() {
                        if ext != "c" && ext != "h" {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if config.cpp_only {
                    if let Some(ext) = entry.path().extension() {
                       if ext != "cpp" && ext != "hpp" && ext != "cc" && ext != "cxx" {
                           continue;
                        } 
                    } else {
                        continue;
                    }
                }

                if config.hide_all {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "bin" || ext == "exe" || ext == "o" || ext == "a" {
                            continue;
                        }
                    }
                }
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
    tree_view: bool,
    delete_colour: bool,
    json_mode: bool,
    buffered_matches: Vec<(usize, String)>,
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

        if self.json_mode {
            let mut out = io::stdout().lock();
            let line_num = mat.line_number().unwrap_or(0);

            let escaped_line = clean_line.replace('\\', "\\\\").replace('"', "\\\"");

            writeln!(
                out,
                r#"{{"type":"match","path":"{}","line_number":{},"text":"{}"}}"#,
                self.file_name.replace('\\', "\\\\").replace('"', "\\\""),
                line_num,
                escaped_line
            )?;

            if self.explain_mode {
                self.execute_explanation(mat.bytes());
            }
            return Ok(true);
        }

        if self.tree_view {
            let line_num = mat.line_number().unwrap_or(0) as usize;
            self.buffered_matches.push((line_num, clean_line));

            if self.explain_mode {
                self.execute_explanation(mat.bytes());
            }
            return Ok(true);
        }

        let file_color = if self.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Purple };
        let line_color = if self.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Magenta };

        let colored_line = if self.delete_colour {
            Coloured::new(&clean_line, Colour::Rgb(255, 255, 255))
        } else {
            (self.orange_formatter)(&clean_line) 
        };

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

        if self.json_mode {
            let mut out = io::stdout().lock();
            let line_num = mat.line_number().unwrap_or(0);
            let escaped_line = clean_line.replace('\\', "\\\\").replace('"', "\\\"");
            
            writeln!(
                out,
                r#"{{"type":"context","path":"{}","line_number":{},"text":"{}"}}"#,
                self.file_name.replace('\\', "\\\\").replace('"', "\\\""),
                line_num,
                escaped_line
            )?;
            return Ok(true);
        }

        let file_color = if self.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Purple };
        let line_color = if self.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Magenta };

        let colored_line = if self.delete_colour {
            Coloured::new(&clean_line, Colour::Rgb(255, 255, 255))
        } else {
            (self.orange_formatter)(&clean_line)
        };

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

    if config.hide_all {
        if let Ok(mut file) = File::open(path) {
            let mut buffer = [0; 1024];

            if let Ok(bytes_read) = file.take(1024).read(&mut buffer) {

                #[cfg(target_os = "windows")]
                {
                    if bytes_read >= 2 {
                        let has_utf16_bom = (buffer[0] == 0xFF && buffer[1] == 0xFE)  
                                         || (buffer[0] == 0xFE && buffer[1] == 0xFF);
                        
                        if !has_utf16_bom {
                            let mut looks_like_utf16_le = true;
                            let mut null_count = 0;

                            for i in (0..bytes_read.saturating_sub(1)).step_by(2) {
                                let char_byte = buffer[i];
                                let null_byte = buffer[i + 1];

                                if null_byte == 0 && (char_byte.is_ascii_alphanumeric() || char_byte.is_ascii_whitespace() || char_byte == 0) {
                                    if null_byte == 0 { null_count += 1; }
                                } else {
                                    looks_like_utf16_le = false;
                                    break;
                                }
                            }

                            if !looks_like_utf16_le && buffer[..bytes_read].iter().any(|&b| b == 0) {
                                return Ok(());
                            }

                            if !looks_like_utf16_le && buffer[..bytes_read].iter().any(|&b| b == 0) {
                                return Ok(());
                            }
                        }
                    } else if buffer[..bytes_read].iter().any(|&b| b == 0) {
                        return Ok(());
                    }
                }

                #[cfg(not(target_os = "windows"))]
                if buffer[..bytes_read].iter().any(|&b| b == 0) {
                    return Ok(());
                }
            }
        }
    }

    if config.cat_mode {
        let file = File::open(path)?;
        let mut out = io::stdout().lock();
        let reader = BufReader::new(file);

        if !config.no_filename {
          Coloured::with_style("📂 Cat View: ", Colour::Cyan, Style::bold()).write_to(&mut out).ok();
          Coloured::with_style(&path.to_string_lossy(), Colour::Purple, Style::bold()).write_to(&mut out).ok();
          let _ = writeln!(out, "\n──────────────────────────────────────────────────────────");  
        }

        for (idx, line_result) in reader.lines().enumerate() {
            let line = line_result?;

            if config.line_numbers {
               Coloured::with_style(&(idx + 1).to_string(), Colour::Magenta, Style::bold()).write_to(&mut out).ok();
               let _ = write!(out, ": ");
            }

            if config.delete_colour {
                let _ = writeln!(out, "{}", line);
            } else {
                let _ = writeln!(out, "{}", line);
            }
        }
        return Ok(());
    }


    if config.show_time {
        let mut out = io::stdout().lock();
        if let Ok(timo_now) = TimoDateTime::now("Europe/London") {
            if !config.delete_colour {
                let _ = write!(out, "[");
                Coloured::with_style("TIMO RUNTIME", Colour::Cyan, Style::bold()).write_to(&mut out).ok();
                let _ = write!(out, "] ");
                Coloured::new(&timo_now.status_summary(), Colour::Rgb(255, 135, 0)).write_to(&mut out).ok();
                let _ = writeln!(out, "\n──────────────────────────────────────────────────────────");
            } else {
                let _ = writeln!(out, "[TIMO RUNTIME] {}", timo_now.status_summary());
                let _ = writeln!(out, "──────────────────────────────────────────────────────────");
            }
        }
    }



    let file = File::open(path)?;

    let final_pattern = if config.word_regexp {
        format!(r"\b{}\b", config.pattern)
    } else {
        config.pattern.clone()
    };

    let mut matcher_builder = RegexMatcherBuilder::new();
    matcher_builder.case_insensitive(config.ignore_case);
  

    let matcher = match matcher_builder.build(&final_pattern) {
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
        tree_view: config.tree_view,
        delete_colour: config.delete_colour,
        json_mode: config.json_mode,
        buffered_matches: Vec::new(),
    };

    searcher.search_file(&matcher, &file, &mut sink)?;

    if config.tree_view && !sink.buffered_matches.is_empty() {
        let mut out = io::stdout().lock();
        
        let file_tree_color = if config.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Purple };
        let line_tree_color = if config.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Magenta };
        let leaf_color = if config.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Orange };

        if !config.delete_colour {
            Coloured::new("📄 ", Colour::Cyan).write_to(&mut out)?;
        }
        Coloured::with_style(&sink.file_name, file_tree_color, Style::bold()).write_to(&mut out)?;
        let _ = write!(out, "\n");
     
        for (i, (line_num, line_text)) in sink.buffered_matches.iter().enumerate() {
            let is_last = i == sink.buffered_matches.len() - 1;
            let branch = if is_last { "└── " } else { "├── " };
            
            let _ = write!(out, "{}", branch);
            
            if config.line_numbers && *line_num > 0 {
                let _ = write!(out, "[");
                Coloured::with_style(&line_num.to_string(), Colour::Magenta, Style::bold()).write_to(&mut out)?;
                let _ = write!(out, "] ");
            }

            let colored_line = (orange_formatter)(line_text);
            colored_line.write_to(&mut out)?;
            let _ = write!(out, "\n");
        }
        let _ = write!(out, "\n"); // Extra padding between files
    }

    if config.count_only && sink.match_count > 0 {
        let mut out = io::stdout().lock();
        let count_file_color = if config.delete_colour { Colour::Rgb(255, 255, 255) } else { Colour::Purple };

        if !config.no_filename {
            Coloured::new(&sink.file_name, count_file_color).write_to(&mut out)?;
            write!(out, ":")?;
        }
        let _ = writeln!(out, "{}", sink.match_count);
    }

    Ok(())
}
