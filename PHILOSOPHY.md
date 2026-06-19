# **The LewRep2 Philosophy**

LewRep2 is built on a simple idea:

> A search tool should be fast, predictable, and stay out of your way.

It follows the long tradition of grep‑style tools — the ones developers reach for when they just want to search text quickly and reliably. LewRep2 keeps that spirit, but uses modern Rust and parallelism to push the performance further without changing the fundamentals.

---

# **1. Grep Comes First**

LewRep2 sits in the same family as:

- grep  
- ag  
- ripgrep  
- ugrep  

That means:

- familiar flags  
- familiar output  
- familiar behaviour  

If something breaks the expectations of a grep‑style tool, it doesn’t belong here.

---

# **2. Speed Is a Design Rule**

LewRep2 should *feel* fast every time you run it.

That means:

- minimal allocations  
- tight loops  
- efficient traversal  
- no unnecessary abstractions  

If a change makes the tool slower or heavier, it’s a step in the wrong direction.

---

# **3. Zero‑Waste Execution**

Every part of the pipeline should earn its keep.

LewRep2 avoids:

- redundant syscalls  
- hidden buffering  
- heavyweight crates  
- clever abstractions that hide real cost  

You should be able to read the code and understand exactly what it’s doing.

---

# **4. Predictable Behaviour**

LewRep2 should never surprise you.

No silent filtering.  
No reordering.  
No “smart” behaviour that changes output.  

If grep wouldn’t do it, LewRep2 probably shouldn’t either.

---

# **5. Parallelism With Restraint**

LewRep2 uses parallelism where it actually helps:

- directory traversal  
- file scanning  

But it must not:

- break determinism  
- reorder results  
- complicate the codebase  

Parallelism is a tool, not a gimmick.

---

# **6. Simplicity Matters**

LewRep2 values:

- small, readable functions  
- explicit logic  
- minimal dependencies  
- straightforward control flow  

A grep‑style tool should be understandable, not a puzzle.

---

# **7. Output Should Be Grep‑Friendly**

LewRep2 prints plain text by default.

- no metadata unless asked  
- no JSON unless explicitly requested  
- no surprises in formatting  

It should drop cleanly into pipes and scripts.

---

# **8. Stay Within Scope**

LewRep2 does one job: **search text**.

It doesn’t edit files, rename them, index them, or try to be a general‑purpose tool.  
That’s intentional.

---

# **9. Keep It Small**

LewRep2 should remain:

- lightweight  
- portable  
- dependency‑minimal  
- easy to compile  
- easy to audit  

Bloat kills good tools.

---

# **10. Identity Over Feature Lists**

LewRep2 doesn’t chase trends or bolt on features because other tools have them.

It has a clear identity:  
a fast, predictable, grep‑style search tool with modern performance.

Everything else is optional.

---

# **Conclusion**

LewRep2 is built with intent:

- fast  
- predictable  
- grep‑faithful  
- simple  
- disciplined  

This philosophy guides every decision in the project.
