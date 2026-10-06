use super::*;
#[test]
fn editor_round_trips_bom_crlf_and_refuses_stale_failed_and_unsupported_saves() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("你好.txt");
    std::fs::write(&path, b"\xef\xbb\xbfhello\r\nworld\r\n").unwrap();
    let f = EditorFolder::new(d.path()).unwrap();
    let s = f.snapshot("你好.txt").unwrap();
    assert!(s.bom);
    assert_eq!(s.newline, "crlf");
    assert_eq!(s.text, "hello\nworld\n");
    let next = f
        .save(&s.path, &s.revision, "你好\nworld\n", s.bom, &s.newline)
        .unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        "\u{feff}你好\r\nworld\r\n".as_bytes()
    );
    std::fs::write(&path, "external").unwrap();
    assert!(f
        .save(&s.path, &next.revision, "mine", true, "crlf")
        .is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "external");
    std::fs::remove_file(&path).unwrap();
    assert!(f
        .save(&s.path, &next.revision, "mine", true, "crlf")
        .is_err());
    for (name, bytes) in [
        ("mixed", b"a\r\nb\n".to_vec()),
        ("binary", vec![0, 255]),
        ("long", vec![b'x'; 8193]),
        ("large", vec![b'x'; EDITOR_BYTES + 1]),
    ] {
        std::fs::write(d.path().join(name), &bytes).unwrap();
        let s = f.snapshot(name).unwrap();
        assert!(s.readonly);
        assert!(s.text.len() <= 64 * 1024 * 3);
        assert_eq!(std::fs::read(d.path().join(name)).unwrap(), bytes);
    }
    assert!(f.create("../escape", "", false, "lf").is_err());
    assert!(f.create(".env", "", false, "lf").is_err());
}
#[test]
fn twenty_thousand_entries_are_paged_without_recursive_scanning() {
    let d = tempfile::tempdir().unwrap();
    for n in 0..20_000 {
        std::fs::File::create(d.path().join(format!("f{n:05}.txt"))).unwrap();
    }
    std::fs::create_dir(d.path().join("nested")).unwrap();
    std::fs::write(d.path().join("nested/child"), "child").unwrap();
    let f = EditorFolder::new(d.path()).unwrap();
    let first = f.tree(".", 0).unwrap();
    assert_eq!(first["entries"].as_array().unwrap().len(), 200);
    assert!(first["cursor"].as_u64().unwrap() <= 1000);
    let second = f
        .tree(".", first["cursor"].as_u64().unwrap() as usize)
        .unwrap();
    assert_eq!(second["entries"].as_array().unwrap().len(), 200);
    assert_ne!(first["entries"][0]["path"], second["entries"][0]["path"]);
    assert_eq!(
        f.tree("nested", 0).unwrap()["entries"][0]["path"],
        "nested/child"
    );
}
#[test]
fn create_rename_delete_do_not_replace_an_existing_target() {
    let d = tempfile::tempdir().unwrap();
    let f = EditorFolder::new(d.path()).unwrap();
    let a = f.create("a", "one", false, "lf").unwrap();
    assert!(f.create("a", "two", false, "lf").is_err());
    f.create("b", "two", false, "lf").unwrap();
    assert!(f.rename("a", "b", &a.revision).is_err());
    let c = f.rename("a", "c", &a.revision).unwrap();
    assert!(!d.path().join("a").exists());
    assert!(f.delete("c", "stale").is_err());
    f.delete("c", &c.revision).unwrap();
    assert!(!d.path().join("c").exists());
}
