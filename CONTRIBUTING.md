# **Contributing to LewRep2**

Thanks for taking the time to contribute. LewRep2 is a performance‑first Grep‑style search tool, and every improvement helps keep it fast, sharp, and predictable.

This guide explains how to work on the project, inspect the code, and submit changes without breaking the pipeline.

---

# **📦 Getting Started**

Clone the repository:

\`\`\`bash
git clone https://github.com/xlewis1/LewRep2.git
cd LewRep2
\`\`\`

Inspect the codebase (LewRep2 is intentionally small and readable):

\`\`\`bash
cat src/main.rs
\`\`\`

Build and run:

\`\`\`bash
cargo build --release
./target/release/lewrep2
\`\`\`

---

# **🧠 Code Philosophy**

LewRep2 follows the same principles as classic UNIX & Other Grep clone tools:

- **Simple interface, deep internals**
- **Zero‑waste pipeline**
- **Predictable behaviour**
- **No magic, no hidden work**
- **Performance over features**
- **Parallel where it matters**

If a change slows the tool down or adds unnecessary abstraction, it probably doesn’t belong here.

---

# **🛠️ What You Can Contribute**

LewRep2 welcomes improvements in:

- **Performance** (hot‑path optimisations, IO improvements, mmap tuning)
- **New flags** (as long as they don’t bloat the core)
- **Traversal logic** (ignore rules, smarter walking)
- **Documentation** (README, examples, architecture notes)
- **Bug fixes** (edge cases, platform quirks, error handling)
- **new library crate imports** (ones that are for command line tools only and don't mess with speed)
- **P.S** (it must stay in the syntax of a grep-style clone)

If you’re unsure whether an idea fits the project’s philosophy, open an Issue first.

---

# **🔧 Development Workflow**

1. **Fork** the repository  
2. **Create a branch** for your feature or fix  
3. **Write clean, explicit Rust**  
4. **Benchmark your change**  
   - LewRep2 must remain fast  
   - Use Unix's default time command or your own timing tools  
5. **Commit with clear messages**  
6. **Open a Pull Request**  
   - Explain what changed  
   - Explain *why* it changed  
   - Include benchmarks if performance is affected  

LewRep2 is a performance‑driven project — numbers matter.

---

# **🧪 Testing & Benchmarks**

Before submitting a PR, test your changes on:

- small files  
- large files  
- nested directories  
- ignored directories  
- literal vs regex patterns
- make sure it prints like standard grep, ag, ripgrep, ugrep or other grep clones  

Compare against:

\`\`\`bash
rg
grep
\`\`\`

LewRep2 should never regress without a very good reason.

---

# **📜 Code Style**

- Keep functions small and focused  
- Avoid unnecessary abstractions  
- Prefer explicit over clever  
- Don’t allocate in the hot path  
- Don’t introduce blocking calls
- Don't update versioning in Cargo.Toml
- Respect the UNIX philosophy: **do one thing, do it well**

If your code looks like it belongs in a grep clones internals, you’re on the right track.

---

# **📮 Submitting a Pull Request**

When opening a PR:

- Describe the change  
- Explain the motivation  
- Include benchmarks if relevant  
- Keep the tone human — this is an engineer‑to‑engineer project  

LewRep2 is built with intent, and contributions should reflect that.

---

# **🦀 Thank You**

Every contribution — big or small — helps LewRep2 evolve into a sharper, faster, more reliable tool.
