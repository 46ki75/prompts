//! Hermetic integration tests — no external dependencies, deterministic.

use std::fs;

#[test]
fn builds_resources_and_index() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().join("repo");
    let out = tmp.path().join("dist");

    fs::create_dir_all(root.join("alpha")).expect("mkdir alpha");
    fs::write(root.join("alpha/prompt.md"), "# Alpha Prompt\n\nbody\n").expect("write alpha");
    fs::create_dir_all(root.join("beta")).expect("mkdir beta");
    fs::write(root.join("beta/prompt.md"), "no heading here\n").expect("write beta");
    // Directories without prompt.md and hidden directories are skipped.
    fs::create_dir_all(root.join("no-prompt")).expect("mkdir no-prompt");
    fs::create_dir_all(root.join(".hidden")).expect("mkdir .hidden");
    fs::write(root.join(".hidden/prompt.md"), "# Hidden\n").expect("write hidden");

    let index = builder::build(&root, &out).expect("build succeeds");

    // Index: sorted, titled, hidden/non-prompt dirs excluded.
    assert_eq!(index.len(), 2);
    assert_eq!(index[0].name, "alpha");
    assert_eq!(index[0].title, "Alpha Prompt");
    assert_eq!(index[0].path, "alpha.md");
    assert_eq!(index[1].name, "beta");
    assert_eq!(index[1].title, "beta"); // falls back to directory name

    // Prompt files are copied verbatim.
    let alpha = fs::read_to_string(out.join("resources/alpha.md")).expect("read alpha.md");
    assert_eq!(alpha, "# Alpha Prompt\n\nbody\n");
    assert!(!out.join("resources/no-prompt.md").exists());

    // list.json round-trips as a JSON array with the expected shape.
    let list = fs::read_to_string(out.join("resources/list.json")).expect("read list.json");
    let parsed: serde_json::Value = serde_json::from_str(&list).expect("valid JSON");
    let entries = parsed.as_array().expect("top-level array");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["name"], "alpha");
    assert_eq!(entries[0]["title"], "Alpha Prompt");
    assert_eq!(entries[0]["path"], "alpha.md");
}

#[test]
fn empty_root_produces_empty_index() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().join("repo");
    let out = tmp.path().join("dist");
    fs::create_dir_all(&root).expect("mkdir root");

    let index = builder::build(&root, &out).expect("build succeeds");

    assert!(index.is_empty());
    let list = fs::read_to_string(out.join("resources/list.json")).expect("read list.json");
    assert_eq!(list, "[]\n");
}
