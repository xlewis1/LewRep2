use grep_regex::RegexMatcherBuilder;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use std::sync::atomic::{AtomicU64, Ordering};
use ignore::WalkBuilder;
use lewcolour::{Colour, Coloured, Style};
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, stdin, BufRead, BufReader, IsTerminal, Read, Write};
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
    context_before: usize,
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
    extensions: Vec<String>,
    cat_mode: bool,
    show_time: bool,
    only_matching: bool,
}

const HELP_TEXT: &str = r#"
================================================================================
LEWREP2 HELP
================================================================================
Usage:
lewrep2 [FLAGS] [PATTERN] [PATH...]

A parallel grep utility designed to quickly scan directories using regular expressions.

FLAGS:
    --help                      Display the help manual document layout.
    --manpage                   Display the comprehensive manual document layout.
    -i, --ignore-case           Perform case-insensitive text evaluation.
    -n, --line-number           Prefix each match line with its 1-based sequential line index.
    -v, --invert-match          Invert query matching selection properties.
    -l, --files-with-matches    Print only names of target files matching specifications.
    -c, --count                 Print exclusively total matching record line metrics per file.
    -h                          doesn't print filenames, just the files text.
    -o                          matches only the specific pattern you searched.
    -w, --word-regexp           Bound regular expression to validate complete word sequences.
    -T, --tree                  Format match visual layout structural mappings hierarchically.
    -d, --delete-colour         Strip text styling components before output execution.
    -j, --json                  Stream structural data components formatted as raw JSON.
    -X, --explain               Interactively clarify capture group structural properties.
    -A <NUM>                    Append context detailing trailing data lines.
    -B <NUM>                    Print Num lines of trailing context before matching lines.
    -C <NUM>                    Print Num lines of trailing context before and after matching lines.
    -x <EXT|EXT|...>            Limit target evaluation queries to a pipe-separated list of
                                file extensions ( -x "rs", -x "c|h", -x "cpp|hpp|cc|cxx").
    --Hide                      Omit structured path matching binary formatting properties.
    --vscode                    Override defaults to scan structural .vscode tracking areas.
    -u, -uu and -uuu            Shows .gitignore, hidden files and binaries.
    --Hide                      Does the opposite to "-u" and hides hidden files,
                                binaries and .gitignore.
    --cat                       prints cat-style text prints.
    --time                      shows the time via my Timo date/time library crate.

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

       -u, --Unrestricted mode
             include hidden files.

       -uu, --include hidden
             includes hidden directories.

       -uuu, --show everything
             include binary files and all normally excluded content.

       -c, --count
             Only print a count of matching lines per file.

       -h, --no filename
              removes filename and just returns the files text.

       -w, --word-regexp
             Match only whole words matching PATTERN.

       -T, --tree
             Display results in a structured hierarchical visual tree.

       -d, --delete-colour
             Strip all color output styling.

       -X, --explain
             Explain regular expression match captures interactively.
        
       -o, --Matching only
             Matches only the pattern you searched not the full text.

       -A <NUM>
             Print NUM lines of trailing context after matching lines.

       -B <NUM>
             Print Num lines of trailing context before matching lines.

       -C <NUM>
             Print Num lines of trailing context before and after matching lines.

       -x <EXT|EXT|...>
             Limit search to files matching a pipe-separated list of
             extensions. Examples: -x "rs", -x "c|h", -x "cpp|hpp|cc|cxx".

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

    let total_bytes_scanned = std::sync::atomic::AtomicU64::new(0);
    let total_files_scanned = std::sync::atomic::AtomicU64::new(0);

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
            lewcolour::Coloured::with_style(
                "TIMO RUNTIME",
                lewcolour::Colour::Cyan,
                lewcolour::Style::bold(),
            )
            .write_to(&mut out)
            .ok();
            let _ = write!(out, "] ");
            lewcolour::Coloured::new(
                &timo_now.status_summary(),
                lewcolour::Colour::Rgb(255, 135, 0),
            )
            .write_to(&mut out)
            .ok();
            let _ = writeln!(
                out,
                "\n──────────────────────────────────────────────────────────"
            );
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
    let mut context_before = 0;
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
    let mut cat_mode = false;
    let mut show_time = false;
    let mut only_matching = false;
    let mut extensions: Vec<String> = Vec::new();

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

        // -x "c|rs"
        if arg == "-x" {
            if let Some(ext_str) = args_iter.next() {
                extensions = ext_str
                    .split('|')
                    .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
            } else {
                eprintln!("Error: -x requires a pipe-separated extension list (-x \"rs\" or -x \"c|cpp\")");
                std::process::exit(1);
            }
            continue;
        }

        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 1 {
            if arg == "-A" {
                if let Some(num_str) = args_iter.next() {
                    context_after = num_str.parse::<usize>().unwrap_or(0);
                }
                continue;
            }
            if arg == "-B" {
                if let Some(num_str) = args_iter.next() {
                    context_before = num_str.parse::<usize>().unwrap_or(0);
                }
                continue;
            }
            if arg == "-C" {
                if let Some(num_str) = args_iter.next() {
                    let num = num_str.parse::<usize>().unwrap_or(0);
                    context_after = num;
                    context_before = num;
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
                    't' => show_time = true,
                    'o' => only_matching = true,
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
        context_before,
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
        extensions,
        cat_mode,
        show_time,
        only_matching,
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
                if entry.depth() > 0
                    && entry
                        .path()
                        .components()
                        .any(|c| c.as_os_str() == ".vscode")
                {
                    return true;
                }
                !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|s| s.starts_with('.'))
            });
        }

        let walker = walker_builder.build();

        for entry in walker.flatten() {
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                if !config.extensions.is_empty() {
                    let matches_ext = entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| {
                            config.extensions.iter().any(|e| e == &ext.to_lowercase())
                        });

                    if !matches_ext {
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
                
                if let Ok(metadata) = entry.metadata() {
                    total_bytes_scanned.fetch_add(metadata.len(), std::sync::atomic::Ordering::Relaxed);
                    total_files_scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                target_files.push(entry.into_path());
            }
        }
    }

    target_files.par_iter().for_each(|file_path| {
        let _ = search_in_file(file_path, &config, &total_bytes_scanned, &total_files_scanned);
    });
}

struct CustomSink<F>
where
    F: for<'a> Fn(&'a str) -> Coloured<'a>,
{
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
    only_matching: bool,
    buffered_matches: Vec<(usize, String)>,
}

impl<F> CustomSink<F>
where
    F: for<'a> Fn(&'a str) -> Coloured<'a>,
{
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
                Coloured::with_style("[EXPLAIN]", Colour::Cyan, Style::bold())
                    .write_to(&mut out)
                    .ok();
                let _ = writeln!(out);
            }

            if write!(out, "    Full Match: '").is_ok() {
                let full_match = captures.get(0).map_or("", |m| m.as_str());
                Coloured::new(full_match, Colour::Yellow)
                    .write_to(&mut out)
                    .ok();
                let _ = writeln!(out);
            }

            for i in 1..captures.len() {
                if let Some(group_match) = captures.get(i) {
                    let group_name = compiled_regex
                        .capture_names()
                        .nth(i)
                        .flatten()
                        .map(|name| format!(" ({})", name))
                        .unwrap_or_default();

                    if write!(out, "    => Group ").is_ok() {
                        Coloured::new(&i.to_string(), Colour::Green)
                            .write_to(&mut out)
                            .ok();
                        Coloured::new(&group_name, Colour::Blue)
                            .write_to(&mut out)
                            .ok();
                        let _ = write!(out, ": '");
                        Coloured::new(group_match.as_str(), Colour::Rgb(255, 135, 0))
                            .write_to(&mut out)
                            .ok();
                        let _ = writeln!(
                            out,
                            "' (at bytes {}-{})",
                            group_match.start(),
                            group_match.end()
                        );
                    }
                }
            }
        }
    }
}

impl<F> Sink for CustomSink<F>
where
    F: for<'a> Fn(&'a str) -> Coloured<'a>,
{
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        if self.count_only {
            self.match_count += 1;
            return Ok(true);
        }

        if self.filenames_only {
            println!("{}", self.file_name);
            return Ok(false);
        }

        let mut out = io::stdout().lock();

        let clean_line = String::from_utf8_lossy(mat.bytes())
            .trim_end_matches(['\r', '\n'])
            .to_string();

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

        let file_color = if self.delete_colour {
            Colour::Reset
        } else {
            Colour::Purple
        };
        let line_color = if self.delete_colour {
            Colour::Reset
        } else {
            Colour::Magenta
        };

        let colored_line = if self.delete_colour {
            Coloured::new(&clean_line, Colour::Reset)
        } else {
            (self.orange_formatter)(&clean_line)
        };

        if self.only_matching {
            if let Some(idx) = clean_line.find(&self.pattern) {
                let remainder = &clean_line[idx..];

                let mat_str = remainder
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != ':');

                let colored_match = if self.delete_colour {
                   Coloured::new(mat_str, Colour::Reset)
                } else {
                   (self.orange_formatter)(mat_str)
                };

                if self.show_line_numbers {
                    if let Some(line_num) = mat.line_number() {
                        if !self.no_filename {
                            Coloured::new(&self.file_name, file_color).write_to(&mut out)?;
                            write!(out, ":")?;
                        }
                        Coloured::with_style(&line_num.to_string(), line_color, Style::bold())
                            .write_to(&mut out)?;
                        write!(out, ": ")?;
                    }
                } else if !self.no_filename {
                    Coloured::new(&self.file_name, file_color).write_to(&mut out)?;
                    write!(out, ": ")?;
                }

                colored_match.write_to(&mut out)?;
                writeln!(out)?;
            }

            return Ok(true);
        } else {
            if self.show_line_numbers {
                if let Some(line_num) = mat.line_number() {
                    if !self.no_filename {
                        Coloured::new(&self.file_name, file_color).write_to(&mut out)?;
                        let _ = write!(out, ":");
                    }
                    Coloured::with_style(&line_num.to_string(), line_color, Style::bold())
                        .write_to(&mut out)?;
                    write!(out, ": ")?;
                    colored_line.write_to(&mut out)?;
                    writeln!(out)?;
                }
            } else {
                if !self.no_filename {
                    Coloured::new(&self.file_name, file_color).write_to(&mut out)?;
                    write!(out, ": ")?;
                }
                colored_line.write_to(&mut out)?;
                writeln!(out)?;
            }
        }


        if self.explain_mode {
            self.execute_explanation(mat.bytes());
        }

        Ok(true)
    }

    fn context(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &grep_searcher::SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
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

        let _file_color = if self.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Purple
        };
        let _line_color = if self.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Magenta
        };

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
                Coloured::with_style(&line_num.to_string(), Colour::Magenta, Style::bold())
                    .write_to(&mut out)?;
                write!(out, "- ")?;
                colored_line.write_to(&mut out)?;
                writeln!(out)?;
            }
        } else {
            if !self.no_filename {
                Coloured::new(&self.file_name, Colour::Purple).write_to(&mut out)?;
                write!(out, "- ")?;
            }
            colored_line.write_to(&mut out)?;
            writeln!(out)?;
        }
        Ok(true)
    }
}

fn process_stdin(pattern: &str) {
    // Collect the raw arguments to check for flags
    let args_list: Vec<String> = std::env::args().collect();
    let only_matching = args_list.iter().any(|arg| arg == "-o" || arg == "--only-matching");
    let line_numbers = args_list.iter().any(|arg| arg == "-n" || arg == "--line-number");

    let reader = BufReader::new(stdin());
    let mut out = io::stdout().lock();

    for (idx, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            if line.contains(pattern) {
                if only_matching {
                    if let Some(idx_match) = line.find(pattern) {
                        let remainder = &line[idx_match..];
                        let mat_str = remainder
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim_matches(|c: char| !c.is_alphanumeric() && c != ':');

                        if line_numbers {
                            Coloured::with_style(&(idx + 1).to_string(), Colour::Magenta, Style::bold())
                                .write_to(&mut out)
                                .ok();
                            let _ = write!(out, ": ");
                        }
                        Coloured::new(mat_str, Colour::Rgb(255, 135, 0))
                            .write_to(&mut out)
                            .ok();
                        let _ = writeln!(out);
                    }
                } else {
                    if line_numbers {
                        Coloured::with_style(&(idx + 1).to_string(), Colour::Magenta, Style::bold())
                            .write_to(&mut out)
                            .ok();
                        let _ = write!(out, ": ");
                    }
                    Coloured::new(&line, Colour::Rgb(255, 135, 0))
                        .write_to(&mut out)
                        .ok();
                    let _ = writeln!(out);
                }
            }
        }
    }
}

fn search_in_file(path: &Path, config: &Config, total_bytes_scanned: &std::sync::atomic::AtomicU64, total_files_scanned: &std::sync::atomic::AtomicU64) -> io::Result<()> {
    if config.hide_all {
        if let Ok(file) = File::open(path) {
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

                                if null_byte == 0
                                    && (char_byte.is_ascii_alphanumeric()
                                        || char_byte.is_ascii_whitespace()
                                        || char_byte == 0)
                                {
                                    if null_byte == 0 {
                                        null_count += 1;
                                    }
                                } else {
                                    looks_like_utf16_le = false;
                                    break;
                                }
                            }

                            if !looks_like_utf16_le && buffer[..bytes_read].iter().any(|&b| b == 0)
                            {
                                return Ok(());
                            }

                            if !looks_like_utf16_le && buffer[..bytes_read].iter().any(|&b| b == 0)
                            {
                                return Ok(());
                            }
                        }
                    } else if buffer[..bytes_read].iter().any(|&b| b == 0) {
                        return Ok(());
                    }
                }

                #[cfg(not(target_os = "windows"))]
                if buffer[..bytes_read].contains(&0) {
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
            Coloured::with_style("📂 Cat View: ", Colour::Cyan, Style::bold())
                .write_to(&mut out)
                .ok();
            Coloured::with_style(&path.to_string_lossy(), Colour::Purple, Style::bold())
                .write_to(&mut out)
                .ok();
            let _ = writeln!(
                out,
                "\n──────────────────────────────────────────────────────────"
            );
        }

        for (idx, line_result) in reader.lines().enumerate() {
            let line = line_result?;

            if config.line_numbers {
                Coloured::with_style(&(idx + 1).to_string(), Colour::Magenta, Style::bold())
                    .write_to(&mut out)
                    .ok();
                let _ = write!(out, ": ");
            }

            if config.delete_colour {
                let _ = writeln!(out, "{}", line);
            }
        }
        return Ok(());
    }

    if config.show_time {
        let mut out = io::stdout().lock();
        if let Ok(timo_now) = TimoDateTime::now("Europe/London") {
            let bytes = total_bytes_scanned.load(std::sync::atomic::Ordering::Relaxed);
            let files = total_files_scanned.load(std::sync::atomic::Ordering::Relaxed);

            let formatted_total_size = fsize_core::format_size(bytes as u128, None, false);

            let avg_bytes = if files > 0 { bytes / files } else { 0 };
            let formatted_avg_size = fsize_core::format_size(avg_bytes as u128, None, false);

            if !config.delete_colour {
                let _ = write!(out, "[");
                Coloured::with_style("TIMO RUNTIME", Colour::Cyan, Style::bold())
                    .write_to(&mut out)
                    .ok();
                let _ = write!(out, "] ");
                Coloured::new(&timo_now.status_summary(), Colour::Rgb(255, 135, 0))
                    .write_to(&mut out)
                    .ok();
                let _ = write!(out, "\n[");
                Coloured::with_style("STORAGE INSIGHTS", Colour::Green, Style::bold())
                    .write_to(&mut out)
                    .ok();
                let _ = write!(out, "] Scanned Files: ");
                Coloured::new(&files.to_string(), Colour::Yellow).write_to(&mut out).ok();
                let _ = write!(out, " | Aggregate Volume: ");
                Coloured::new(&formatted_total_size, Colour::Yellow).write_to(&mut out).ok();
                let _ = write!(out, " | Average Size: ");
                Coloured::new(&formatted_avg_size, Colour::Yellow).write_to(&mut out).ok();

                let _ = writeln!(
                    out,
                    "\n──────────────────────────────────────────────────────────"
                );
            } else {
                let _ = writeln!(out, "[TIMO RUNTIME] {}", timo_now.status_summary());
                let _ = writeln!(
                    out,
                    "──────────────────────────────────────────────────────────"
                );
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
        .before_context(config.context_before)
        .after_context(config.context_after)
        .invert_match(config.invert_match)
        .build();

    let orange_formatter: for<'a> fn(&'a str) -> Coloured<'a> =
        |line| Coloured::new(line, Colour::Orange);

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
        only_matching: config.only_matching,
        buffered_matches: Vec::new(),
    };

    searcher.search_file(&matcher, &file, &mut sink)?;

    if config.tree_view && !sink.buffered_matches.is_empty() {
        let mut out = io::stdout().lock();

        let file_tree_color = if config.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Purple
        };
        let _line_tree_color = if config.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Magenta
        };
        let _leaf_color = if config.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Orange
        };

        if !config.delete_colour {
            Coloured::new("📄 ", Colour::Cyan).write_to(&mut out)?;
        }
        Coloured::with_style(&sink.file_name, file_tree_color, Style::bold()).write_to(&mut out)?;
        let _ = writeln!(out);

        for (i, (line_num, line_text)) in sink.buffered_matches.iter().enumerate() {
            let is_last = i == sink.buffered_matches.len() - 1;
            let branch = if is_last { "└── " } else { "├── " };

            let _ = write!(out, "{}", branch);

            if config.line_numbers && *line_num > 0 {
                let _ = write!(out, "[");
                Coloured::with_style(&line_num.to_string(), Colour::Magenta, Style::bold())
                    .write_to(&mut out)?;
                let _ = write!(out, "] ");
            }

            let colored_line = (orange_formatter)(line_text);
            colored_line.write_to(&mut out)?;
            let _ = writeln!(out);
        }
        let _ = writeln!(out); // Extra padding between files
    }

    if config.count_only && sink.match_count > 0 {
        let mut out = io::stdout().lock();
        let count_file_color = if config.delete_colour {
            Colour::Rgb(255, 255, 255)
        } else {
            Colour::Purple
        };

        if !config.no_filename {
            Coloured::new(&sink.file_name, count_file_color).write_to(&mut out)?;
            write!(out, ":")?;
        }
        let _ = writeln!(out, "{}", sink.match_count);
    }

    Ok(())
}
